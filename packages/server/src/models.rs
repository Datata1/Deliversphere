use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Repräsentiert einen Agenten, wie er in der `agents`-Tabelle gespeichert ist.
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub id: String,
    pub hostname: String,
    pub status: String,
    pub last_heartbeat: i64,
}

/// Repräsentiert einen Job, wie er in der `jobs`-Tabelle gespeichert ist.
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Job {
    pub id: String,
    pub agent_id: Option<String>, 
    pub status: String,
    pub repository_url: String,
    #[sqlx(json)]
    pub commands: Vec<String>,
    pub created_at: i64,
}