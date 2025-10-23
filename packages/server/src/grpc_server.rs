pub mod runner {
    tonic::include_proto!("runner");
}

use crate::{db::DbPool, models::Agent, LiveAgentMap, WsClientMap, WsServerMessage};

use std::pin::Pin;
use std::time::SystemTime;
use tokio::sync::mpsc;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

pub use runner::runner_service_server::RunnerServiceServer;
use runner::{
    agent_request::Payload, runner_service_server::RunnerService, AgentRequest, ServerCommand,
};

pub struct MyRunnerService {
    pub db_pool: DbPool,
    pub live_agents: LiveAgentMap,
    pub ws_clients: WsClientMap,
}

async fn broadcast_ws_message(clients: &WsClientMap, message: &WsServerMessage) {
    for entry in clients.iter() {
        let tx = entry.value();
        if tx.send(message.clone()).is_err() {
            println!(
                "Failed to send WS message to client {}, will be cleaned up on next disconnect.",
                entry.key()
            );
        }
    }
}

#[tonic::async_trait]
impl RunnerService for MyRunnerService {
    type CommunicateStream =
        Pin<Box<dyn Stream<Item = Result<ServerCommand, Status>> + Send + 'static>>;

    async fn communicate(
        &self,
        request_stream: Request<Streaming<AgentRequest>>,
    ) -> Result<Response<Self::CommunicateStream>, Status> {
        let mut inbound = request_stream.into_inner();
        let (tx, rx) = mpsc::channel(128);

        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        let response = Response::new(Box::pin(output_stream) as Self::CommunicateStream);

        let db_pool = self.db_pool.clone();
        let live_agents = self.live_agents.clone();
        let ws_clients = self.ws_clients.clone();

        tokio::spawn(async move {
            let agent_id: Option<String>;

            if let Some(Ok(first_msg)) = inbound.next().await {
                if let Some(Payload::Register(reg)) = first_msg.payload {
                    println!("Agent '{}' registriert sich...", &reg.agent_id);
                    agent_id = Some(reg.agent_id.clone());

                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64;

                    let query = sqlx::query(
                        r#"
                        INSERT INTO agents (id, hostname, status, last_heartbeat)
                        VALUES (?, ?, 'online', ?)
                        ON CONFLICT(id) DO UPDATE SET
                            hostname = excluded.hostname, status = 'online', last_heartbeat = excluded.last_heartbeat
                        "#,
                    )
                    .bind(&reg.agent_id)
                    .bind(&reg.hostname)
                    .bind(now)
                    .execute(&db_pool)
                    .await;

                    if let Err(e) = query {
                        eprintln!("DB-Fehler bei Agent-Registrierung: {}", e);
                        return;
                    }

                    let agent_result =
                        sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?")
                            .bind(&reg.agent_id)
                            .fetch_optional(&db_pool)
                            .await;

                    if let Ok(Some(agent)) = agent_result {
                        let update_msg = WsServerMessage::AgentUpdate { agent };
                        println!("Broadcasting WS AgentUpdate: {:?}", update_msg);
                        broadcast_ws_message(&ws_clients, &update_msg).await;
                    } else {
                        eprintln!(
                            "Konnte Agent {} nach Registrierung nicht aus DB laden.",
                            reg.agent_id
                        );
                    }

                    live_agents.insert(reg.agent_id, tx.clone());
                    println!(
                        "Agent '{}' ist jetzt online (Registrierung abgeschlossen).",
                        agent_id.as_ref().unwrap()
                    );
                } else {
                    eprintln!("Fehler: Erste Nachricht war nicht 'RegisterAgent'");
                    return;
                }
            } else {
                eprintln!("Agent hat Verbindung vor Registrierung getrennt");
                return;
            }

            let current_agent_id = match agent_id {
                Some(id) => id,
                None => return,
            };

            while let Some(result) = inbound.next().await {
                if let Ok(request) = result {
                    if let Some(Payload::Heartbeat(_)) = request.payload {
                        let now = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        let _ = sqlx::query("UPDATE agents SET last_heartbeat = ? WHERE id = ?")
                            .bind(now)
                            .bind(&current_agent_id)
                            .execute(&db_pool)
                            .await;
                    }
                } else {
                    eprintln!("Fehler beim Empfangen von Agent '{}'", current_agent_id);
                    break;
                }
            }

            println!("Agent '{}' hat die Verbindung getrennt.", current_agent_id);
            live_agents.remove(&current_agent_id);
            let _ = sqlx::query("UPDATE agents SET status = 'offline' WHERE id = ?")
                .bind(&current_agent_id)
                .execute(&db_pool)
                .await;

            let agent_result = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?")
                .bind(&current_agent_id)
                .fetch_optional(&db_pool)
                .await;

            if let Ok(Some(agent)) = agent_result {
                let update_msg = WsServerMessage::AgentUpdate { agent };
                println!("Broadcasting WS AgentUpdate (disconnect): {:?}", update_msg);
                broadcast_ws_message(&ws_clients, &update_msg).await;
            }
        });

        Ok(response)
    }
}
