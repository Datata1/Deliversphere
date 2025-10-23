use futures_util::future::TryFutureExt;
use server::{
    db,
    grpc_server::{MyRunnerService, RunnerServiceServer},
    http_server,
    state::AppState,
    tasks, AppError, LiveAgentMap, Result, WsClientMap,
};
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Initialisiere Server...");
    let db_pool = db::init_pool().await?;
    let live_agents = LiveAgentMap::default();
    let ws_clients = WsClientMap::default();
    let app_state = AppState {
        db_pool: db_pool.clone(),
        ws_clients: ws_clients.clone(),
    };

    tasks::spawn_background_tasks(db_pool.clone());

    let grpc_addr = "[::]:3001".parse().map_err(|e| {
        AppError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid gRPC address: {}", e),
        ))
    })?;
    let runner_service = MyRunnerService {
        db_pool,
        live_agents,
        ws_clients,
    };
    let grpc_server_future = Server::builder()
        .add_service(RunnerServiceServer::new(runner_service))
        .serve(grpc_addr)
        .map_err(|e| AppError::from(e));

    let rest_addr: SocketAddr = "[::]:3000".parse().map_err(|e| {
        AppError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid REST address: {}", e),
        ))
    })?;
    let rest_router = http_server::create_router(app_state);
    let listener = TcpListener::bind(rest_addr).await?;
    let rest_server_future = async {
        axum::serve(listener, rest_router.into_make_service())
            .await
            .map_err(|e| AppError::Io(e))
    };

    println!("REST/WebSocket Server lauscht auf {}", rest_addr);
    println!("gRPC Server lauscht auf {}", grpc_addr);
    tokio::try_join!(grpc_server_future, rest_server_future)?;

    println!("Server heruntergefahren.");
    Ok(())
}
