use crate::config::AgentConfig; // Importiere die Config-Struktur
use crate::runner::{AgentRequest, Heartbeat, Payload, RegisterAgent, RunnerServiceClient};

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

async fn run_agent_session(config: AgentConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Versuche, Server zu kontaktieren...");
    let connect_future = RunnerServiceClient::connect(config.server_endpoint.clone());
    let connect_timeout = Duration::from_secs(5);

    let mut client = match timeout(connect_timeout, connect_future).await {
        Ok(Ok(client)) => client,
        Ok(Err(e)) => return Err(format!("Verbindungsfehler: {}", e).into()),
        Err(_) => return Err("Verbindungs-Timeout nach 5 Sekunden".into()),
    };
    println!("Erfolgreich mit gRPC Server verbunden!");

    let (tx, rx) = mpsc::channel(128);
    let outbound = ReceiverStream::new(rx);
    let response = client.communicate(outbound).await?;
    let mut inbound = response.into_inner();

    tx.send(AgentRequest {
        payload: Some(Payload::Register(RegisterAgent {
            agent_id: config.agent_id.clone(),
            hostname: config.hostname.clone(),
        })),
    })
    .await?;
    println!("Registration sent to Worker {}.", config.agent_id);

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let heartbeat_msg = AgentRequest {
                payload: Some(Payload::Heartbeat(Heartbeat {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                })),
            };
            if tx_clone.send(heartbeat_msg).await.is_err() {
                eprintln!("Heartbeat failed, connection closed.");
                break;
            }
        }
    });

    println!("Worker {} is waiting for a job...", config.agent_id);
    while let Some(result) = inbound.next().await {
        match result {
            Ok(command) => {
                println!("\nGot command: {:?}", command);
                println!("Simulate Job execution...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                println!("Simulation successful!!");
            }
            Err(err) => {
                eprintln!("Connection to Server failed: {}", err);
                break;
            }
        }
    }

    println!("Connection ended.");
    Ok(())
}

pub async fn run_client_loop(config: AgentConfig) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("--- start Worker session for {} ---", config.agent_id);

        if let Err(e) = run_agent_session(config.clone()).await {
            eprintln!("Worker session failed: {}", e);
        }

        println!("wait 5 seconds...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
