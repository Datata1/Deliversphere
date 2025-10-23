use crate::db::DbPool;
use crate::WsClientMap;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub ws_clients: WsClientMap,
    // Füge hier zukünftigen Shared State hinzu (z.B. LiveAgentMap für gRPC)
    // pub live_agents: LiveAgentMap, // Wenn benötigt
}
