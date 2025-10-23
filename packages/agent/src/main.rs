use agent::config::load_config;
use agent::grpc_client::run_client_loop;
use agent::health_server::run_health_server;
use futures_util::future::TryFutureExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;

    let health_server_future = run_health_server().map_err(|e| {
        eprintln!("Health Server crashed: {}", e);
        e
    });

    let client_future = run_client_loop(config.clone()).map_err(|e| {
        eprintln!("gRPC Client loop crashed: {}", e);
        e
    });

    println!("start grpc client and health server...");
    tokio::try_join!(health_server_future, client_future)?;

    println!("Worker shut down.");
    Ok(())
}
