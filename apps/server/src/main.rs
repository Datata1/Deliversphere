use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::{Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};

pub mod runner {
    tonic::include_proto!("runner");
}
use runner::{
    runner_service_server::{RunnerService, RunnerServiceServer},
    AgentRequest, ServerCommand,
};

#[derive(Default)]
pub struct MyRunnerService {}

#[tonic::async_trait]
impl RunnerService for MyRunnerService {
    type CommunicateStream =
        Pin<Box<dyn Stream<Item = Result<ServerCommand, Status>> + Send + 'static>>;

    async fn communicate(
        &self,
        request_stream: Request<Streaming<AgentRequest>>,
    ) -> Result<Response<Self::CommunicateStream>, Status> {
        let mut inbound = request_stream.into_inner();
        println!("Neuer Agent hat sich verbunden! Warte auf Nachrichten...");

        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(result) = inbound.next().await {
                match result {
                    Ok(request) => {
                        println!("Nachricht vom Agenten erhalten: {:?}", request);
                    }
                    Err(err) => {
                        eprintln!("Fehler beim Empfangen vom Agenten: {}", err);
                        break;
                    }
                }
            }
            println!("Agent hat die Verbindung getrennt.");
        });

        tokio::spawn(async move {
            println!("Sende ersten Job...");
            let job1 = ServerCommand {
                payload: Some(runner::server_command::Payload::Job(runner::RunJob {
                    job_id: "123".into(),
                    repository_url: "http://test.com/repo.git".into(),
                    commands: vec!["echo 'Job 1: Hallo Welt'".into()],
                })),
            };
            tx.send(Ok(job1)).await.unwrap();

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            println!("Sende zweiten Job...");
            let job2 = ServerCommand {
                payload: Some(runner::server_command::Payload::Job(runner::RunJob {
                    job_id: "456".into(),
                    repository_url: "http://test.com/repo.git".into(),
                    commands: vec!["echo 'Job 2: Ich bin immer noch da!'".into()],
                })),
            };
            tx.send(Ok(job2)).await.unwrap();
        });
        
        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        Ok(Response::new(
            Box::pin(output_stream) as Self::CommunicateStream
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:3001".parse()?;
    let runner_service = MyRunnerService::default();
    println!("gRPC Server lauscht auf {}", addr);
    Server::builder()
        .add_service(RunnerServiceServer::new(runner_service))
        .serve(addr)
        .await?;
    Ok(())
}