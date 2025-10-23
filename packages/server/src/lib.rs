use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use std::sync::Arc;
use tokio::sync::mpsc::{Sender, UnboundedSender};
use uuid::Uuid;

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
}
pub type WsClientTx = UnboundedSender<WsServerMessage>;
pub type WsClientMap = Arc<DashMap<Uuid, WsClientTx>>;

pub type LiveAgentMap = Arc<
    DashMap<String, Sender<StdResult<crate::grpc_server::runner::ServerCommand, tonic::Status>>>,
>;

pub mod db;
pub mod error;
pub mod grpc_server;
pub mod http_server;
pub mod models;
pub mod state;
pub mod tasks;

pub use error::{AppError, Result};
