use crate::db::DbPool;
use crate::state::AppState;
use crate::{models::Agent, WsClientMap, WsClientMessage, WsServerMessage};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json;
use sqlx;
use tokio::sync::mpsc;
use uuid::Uuid;

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    println!("New WebSocket connection attempt");
    ws.on_upgrade(move |socket| handle_socket(socket, app_state.ws_clients, app_state.db_pool))
}

async fn handle_socket(socket: WebSocket, clients: WsClientMap, db_pool: DbPool) {
    let client_id = Uuid::new_v4();
    println!("WebSocket client connected: {}", client_id);

    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<WsServerMessage>();

    clients.insert(client_id, tx);

    let initial_agents = sqlx::query_as::<_, Agent>("SELECT * FROM agents")
        .fetch_all(&db_pool)
        .await
        .unwrap_or_else(|e| {
            eprintln!("DB Error fetching initial state for {}: {}", client_id, e);
            vec![]
        });

    let initial_msg = WsServerMessage::InitialState {
        agents: initial_agents,
    };
    if let Ok(json_msg) = serde_json::to_string(&initial_msg) {
        if sender.send(Message::Text(json_msg.into())).await.is_err() {
            eprintln!("Failed to send initial state to {}", client_id);
            clients.remove(&client_id);
            println!("WebSocket client disconnected early: {}", client_id);
            return;
        }
        println!("Sent initial state to {}", client_id);
    }

    let send_task = tokio::spawn(async move {
        while let Some(msg_to_send) = rx.recv().await {
            if let Ok(json_msg) = serde_json::to_string(&msg_to_send) {
                if sender.send(Message::Text(json_msg.into())).await.is_err() {
                    println!("Send failed for {}, breaking send task.", client_id);
                    break;
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    println!("Received text from {}: {}", client_id, text);
                    if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                        println!("Parsed client message: {:?}", client_msg);
                    } else {
                        eprintln!("Failed to parse client message from {}", client_id);
                    }
                }
                Message::Binary(_) => {
                    println!("Received binary data (unhandled) from {}", client_id);
                }
                Message::Ping(_) | Message::Pong(_) => {}
                Message::Close(_) => {
                    println!("Client {} sent close frame.", client_id);
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => { /* Send task finished, probably an error */ },
        _ = recv_task => { /* Receive task finished, client closed or error */ },
    }

    println!("WebSocket client disconnected: {}", client_id);
    clients.remove(&client_id);
}

async fn health_check_handler() -> Html<&'static str> {
    Html("OK")
}

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(health_check_handler))
        .route("/api/ws", get(websocket_handler))
        .with_state(app_state)
}
