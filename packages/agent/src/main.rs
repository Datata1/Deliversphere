use std::time::Duration;
use std::env;
use std::net::SocketAddr;

use tokio::sync::mpsc;
use tokio::time::timeout; 
use tokio::net::TcpListener;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use tonic::transport::Endpoint;

use axum::{response::Html, routing::get, Router};
use futures_util::future::TryFutureExt;

use hostname;

pub mod runner {
    tonic::include_proto!("runner");
}
use runner::{
    agent_request::{Payload},
    runner_service_client::RunnerServiceClient,
    AgentRequest, Heartbeat, RegisterAgent,
};

async fn run_agent_session() -> Result<(), Box<dyn std::error::Error>> {

	let server_name = env::var("CS_SERVER")
        .map_err(|e| format!("Env-Var CS_SERVER nicht gefunden: {}", e))?;
    let replica_name = env::var("CS_REPLICA")
        .map_err(|e| format!("Env-Var CS_REPLICA nicht gefunden: {}", e))?;
    
    let agent_id = format!("{}_{}", server_name, replica_name);
    
    let hostname = match hostname::get() { 
        Ok(os_string) => { 
            os_string.into_string()
                .unwrap_or_else(|_| "invalid_hostname".to_string()) 
        }
        Err(_) => { 
            "unknown_hostname".to_string() // Fallback
        }
    };
    
    let endpoint = Endpoint::from_static("http://ws-server-71635-server.workspaces:3001")
        .http2_keep_alive_interval(Duration::from_secs(10));

    // --- NEU: Verbindungsversuch mit 5-Sekunden-Timeout ---
    println!("Versuche, Server zu kontaktieren...");
    let connect_future = RunnerServiceClient::connect(endpoint);
    let connect_timeout = Duration::from_secs(5);

    let mut client = match timeout(connect_timeout, connect_future).await {
        // Erfolg: Timeout nicht ausgelöst, Verbindung erfolgreich
        Ok(Ok(client)) => client,
        // Fehler: Timeout nicht ausgelöst, aber Verbindung fehlgeschlagen (z.B. Refused)
        Ok(Err(e)) => {
            return Err(format!("Verbindungsfehler: {}", e).into());
        }
        // Fehler: Timeout ausgelöst
        Err(_) => {
            return Err("Verbindungs-Timeout nach 5 Sekunden".into());
        }
    };
    // --------------------------------------------------------
    
    println!("Erfolgreich mit gRPC Server verbunden!");

    let (tx, rx) = mpsc::channel(128);
    let outbound = ReceiverStream::new(rx);
    let response = client.communicate(outbound).await?;
    let mut inbound = response.into_inner();

    // 1. Registrierung senden
    tx.send(AgentRequest {
        payload: Some(Payload::Register(RegisterAgent {
            agent_id: agent_id,
            hostname: hostname,
        })),
    })
    .await?;
    println!("Registrierung an Server gesendet.");

    // 2. Heartbeat-Task starten
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            println!("Sende Heartbeat...");
            let heartbeat_msg = AgentRequest {
                payload: Some(Payload::Heartbeat(Heartbeat {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                })),
            };
            if tx_clone.send(heartbeat_msg).await.is_err() {
                eprintln!("Heartbeat fehlgeschlagen, Verbindung getrennt.");
                break;
            }
        }
    });

    // 3. Auf Jobs vom Server lauschen
    println!("Warte auf Jobs...");
    while let Some(result) = inbound.next().await {
        match result {
            Ok(command) => {
                println!("\nBefehl vom Server erhalten: {:?}", command);
                println!("Führe Job aus...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                println!("Job beendet!");
            }
            Err(err) => {
                eprintln!("Fehler bei Verbindung zum Server: {}", err);
                break;
            }
        }
    }

    println!("Verbindung zum Server wurde getrennt.");
    Ok(())
}

async fn health_check_handler() -> Html<&'static str> {
    Html("OK")
}

async fn run_client_loop() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("--- Starte Agenten-Session ---");
        
        if let Err(e) = run_agent_session().await {
            eprintln!("Agenten-Session fehlgeschlagen: {}", e);
        }
        
        println!("Warte 5 Sekunden vor dem nächsten Verbindungsversuch...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}


// --- NEU: Dein Polling-Mechanismus in main ---
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let rest_app = Router::new().route("/", get(health_check_handler));
    let listener = TcpListener::bind(rest_addr).await?;
    let health_server_future = async {
        axum::serve(listener, rest_app.into_make_service())
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    };

    let client_future = run_client_loop();

    println!("Agent Health-Check lauscht auf http://0.0.0.0:3000");
    println!("Agent gRPC-Client startet...");

    tokio::try_join!(
        health_server_future,
        client_future
    )?;

    Ok(())
}