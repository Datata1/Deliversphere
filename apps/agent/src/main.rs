use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

pub mod runner {
    tonic::include_proto!("runner");
}
use runner::{
    agent_request, runner_service_client::RunnerServiceClient, AgentRequest, RegisterAgent,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RunnerServiceClient::connect("http://[::1]:50051").await?;
    println!("Verbunden mit gRPC Server...");

    let (tx, rx) = mpsc::channel(128);

    let outbound = ReceiverStream::new(rx);

    let response = client.communicate(outbound).await?;
    let mut inbound = response.into_inner();

    tx.send(AgentRequest {
        payload: Some(agent_request::Payload::Register(RegisterAgent {
            agent_id: "agent-007".into(),
            hostname: "macbook-pro".into(),
        })),
    })
    .await?;
    println!("Registrierung an Server gesendet. Warte auf Jobs...");

    while let Some(result) = inbound.next().await {
        match result {
            Ok(command) => {
                println!("\nBefehl vom Server erhalten: {:?}", command);
                println!("Führe Job aus...");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; 
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