use axum::{response::Html, routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;

async fn health_check_handler() -> Html<&'static str> {
    Html("OK")
}

pub async fn run_health_server() -> Result<(), Box<dyn std::error::Error>> {
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], 3002));
    let rest_app = Router::new().route("/", get(health_check_handler));

    println!("worker online...");
    let listener = TcpListener::bind(rest_addr).await?;
    axum::serve(listener, rest_app.into_make_service()).await?;
    Ok(())
}
