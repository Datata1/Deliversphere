use server::db::{self, DbPool}; // Import db module and DbPool type
use server::models::Agent; // Import Agent model
use server::server::{MyRunnerService, RunnerServiceServer}; // Import gRPC service
use server::{LiveAgentMap, WsClientMap, WsServerMessage, WsClientMessage};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade}, 
        State,
    },
    response::{Html, IntoResponse}, 
    routing::get, 
    Router, 
};

use tokio::net::TcpListener; 
use tokio::sync::mpsc; 


use std::net::SocketAddr;
use std::time::Duration; 

use futures_util::{sink::SinkExt, stream::StreamExt, future::TryFutureExt}; // WS Stream handling + map_err
use serde_json; // For serializing WS messages
use sqlx; // For database queries
use uuid::Uuid; // For generating client IDs

use tonic::transport::Server;

#[derive(Clone)] // Wichtig für .with_state()
struct AppState {
    db_pool: DbPool,
    ws_clients: WsClientMap,
}


async fn websocket_handler(
    ws: WebSocketUpgrade, // Type should now resolve
    State(app_state): State<AppState>,
) -> impl IntoResponse { // Trait should now resolve
    println!("New WebSocket connection attempt");
		ws.on_upgrade(move |socket| handle_socket(socket, app_state.ws_clients, app_state.db_pool))
}

async fn handle_socket(socket: WebSocket, clients: WsClientMap, db_pool: DbPool) {
    let client_id = Uuid::new_v4(); // Generate unique ID for this client
    println!("WebSocket client connected: {}", client_id);

    // Split the socket into a sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Create an unbounded mpsc channel for this client
    // We'll send messages destined FOR this client INTO this channel
    let (tx, mut rx) = mpsc::unbounded_channel::<WsServerMessage>();

    // Store the sender tx in the shared map
    clients.insert(client_id, tx);

    // --- Send Initial State ---
    let initial_agents = sqlx::query_as::<_, Agent>("SELECT * FROM agents")
        .fetch_all(&db_pool)
        .await
        .unwrap_or_else(|e| {
            eprintln!("DB Error fetching initial state for {}: {}", client_id, e);
            vec![] 
        });

    let initial_msg = WsServerMessage::InitialState { agents: initial_agents };
    if let Ok(json_msg) = serde_json::to_string(&initial_msg) {
        // Send initial state immediately
        if sender.send(Message::Text(json_msg.into())).await.is_err() {
            eprintln!("Failed to send initial state to {}", client_id);
            // Don't continue if we can't even send the first message
             clients.remove(&client_id);
             println!("WebSocket client disconnected early: {}", client_id);
             return;
        }
         println!("Sent initial state to {}", client_id);
    }
    // --- End Send Initial State ---


    // This task forwards messages from the client-specific mpsc channel
    // out over the WebSocket connection.
    let send_task = tokio::spawn(async move {
        while let Some(msg_to_send) = rx.recv().await {
            if let Ok(json_msg) = serde_json::to_string(&msg_to_send) {
                if sender.send(Message::Text(json_msg.into())).await.is_err() {
                    // Error sending, client likely disconnected
                    println!("Send failed for {}, breaking send task.", client_id);
                    break;
                }
            }
        }
    });

    // This task handles messages received FROM the client over WebSocket
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    println!("Received text from {}: {}", client_id, text);
                    // Attempt to parse client message (for future actions)
                    if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                         println!("Parsed client message: {:?}", client_msg);
                        // --- TODO: Handle client messages (e.g., reruns) ---
                        // match client_msg {
                        //    WsClientMessage::RequestRerun { job_id } => { /* ... */ }
                        // }
                    } else {
                        eprintln!("Failed to parse client message from {}", client_id);
                    }
                }
                Message::Binary(_) => {
                    println!("Received binary data (unhandled) from {}", client_id);
                }
                Message::Ping(_) | Message::Pong(_) => {
                    // Handled automatically by Axum/Hyper
                }
                Message::Close(_) => {
                    println!("Client {} sent close frame.", client_id);
                    break; // Exit loop on close
                }
            }
        }
        // Loop exited, client likely disconnected
    });

    // Keep the connection alive until one of the tasks finishes (error or close)
    tokio::select! {
        _ = send_task => { /* Send task finished, probably an error */ },
        _ = recv_task => { /* Receive task finished, client closed or error */ },
    }

    // --- Cleanup ---
    println!("WebSocket client disconnected: {}", client_id);
    clients.remove(&client_id); // Remove client from shared map
}

async fn health_check_handler() -> Html<&'static str> {
    Html("OK")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Datenbank-Pool initialisieren
    let db_pool = db::init_pool().await?;
    let live_agents = LiveAgentMap::default();
		let ws_clients = WsClientMap::default();

		let app_state = AppState {
	        db_pool: db_pool.clone(), 
	        ws_clients: ws_clients.clone(), 
	    };
	
    // 3. Health-Check-Task (liest jetzt aus der DB!)
    let db_pool_clone = db_pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            println!("\n--- Agent Health Check ---");

            let cutoff_time = (std::time::SystemTime::now() - Duration::from_secs(60))
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            
            // Setze alle Agents auf 'offline', deren Heartbeat zu alt ist
            let query = sqlx::query(
                "UPDATE agents SET status = 'offline' WHERE last_heartbeat < ? AND status = 'online'"
            )
            .bind(cutoff_time)
            .execute(&db_pool_clone)
            .await;
            
            match query {
                Ok(result) => {
                    if result.rows_affected() > 0 {
                        println!("{} Agents als 'offline' markiert.", result.rows_affected());
                    }
                },
                Err(e) => eprintln!("Fehler beim Health-Check-Update: {}", e)
            }
        }
    });


    // 4. gRPC-Server starten
    let grpc_addr = "[::]:3001".parse()?;
    let runner_service = MyRunnerService {
        db_pool,
        live_agents,
				ws_clients,
    };
    let grpc_server_future = Server::builder()
        .add_service(RunnerServiceServer::new(runner_service))
        .serve(grpc_addr)
				.map_err(|e| Box::new(e) as Box<dyn std::error::Error>);

    // Task B: Der REST-Server auf Port 3000
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let rest_app = Router::new()
			.route("/", get(health_check_handler))
			.route("/api/ws", get(websocket_handler))
			.with_state(app_state);
	
    let listener = TcpListener::bind(rest_addr).await?;
		let rest_server_future = async {
    axum::serve(listener, rest_app.into_make_service())
				.await 
				.map_err(|e| Box::new(e) as Box<dyn std::error::Error>) 
		};

    // --- 3. Beide Server gleichzeitig starten ---
    println!("REST Health-Check lauscht auf http://0.0.0.0:3000");
    println!("gRPC Server lauscht auf [::]:3001");

    tokio::try_join!(
        grpc_server_future,
        rest_server_future
    )?;

    Ok(())
}