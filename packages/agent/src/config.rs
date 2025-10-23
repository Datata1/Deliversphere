use std::env;
use std::time::Duration;
use tonic::transport::Endpoint;

#[derive(Clone, Debug)]
pub struct AgentConfig {
    pub agent_id: String,
    pub hostname: String,
    pub server_endpoint: Endpoint,
}

pub fn load_config() -> Result<AgentConfig, Box<dyn std::error::Error>> {
    println!("Lade Agenten-Konfiguration...");

    let server_name =
        env::var("CS_SERVER").map_err(|e| format!("Env-Var CS_SERVER nicht gefunden: {}", e))?;
    let replica_name =
        env::var("CS_REPLICA").map_err(|e| format!("Env-Var CS_REPLICA nicht gefunden: {}", e))?;

    let agent_id = format!("{}_{}", server_name, replica_name);

    let hostname = match hostname::get() {
        Ok(os_string) => os_string
            .into_string()
            .unwrap_or_else(|_| "invalid_hostname".to_string()),
        Err(_) => "unknown_hostname".to_string(),
    };

    let server_addr = "http://ws-server-71635-server.workspaces:3001";
    println!("Server Adresse: {}", server_addr);

    let server_endpoint =
        Endpoint::from_static(server_addr).http2_keep_alive_interval(Duration::from_secs(10));

    let config = AgentConfig {
        agent_id,
        hostname,
        server_endpoint,
    };

    println!(
        "Konfiguration geladen: ID={}, Hostname={}",
        config.agent_id, config.hostname
    );
    Ok(config)
}
