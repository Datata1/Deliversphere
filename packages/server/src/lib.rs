use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedSender};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum WsServerMessage {
    InitialState { agents: Vec<models::Agent> }, 
    AgentUpdate { agent: models::Agent },      
    StatsUpdate { online: usize, offline: usize }, 
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WsClientMessage {
    RequestRerun { job_id: String },
    // Add other client actions here
}

pub type WsClientTx = UnboundedSender<WsServerMessage>;
pub type WsClientMap = Arc<DashMap<Uuid, WsClientTx>>;
pub type LiveAgentMap = Arc<DashMap<String, tokio::sync::mpsc::Sender<Result<crate::server::runner::ServerCommand, tonic::Status>>>>;
pub mod db;
pub mod models;
pub mod server;