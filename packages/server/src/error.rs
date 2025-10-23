use serde_json;
use sqlx;
use std::env;
use std::io;
use thiserror::Error;
use tokio;
use tonic;
use uuid;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: Environment variable '{0}' not found")]
    MissingEnvVar(String),

    #[error("Configuration error: {0}")]
    ConfigVar(#[from] env::VarError),

    #[error("Invalid Server URL: {0}")]
    InvalidServerUrl(#[from] tonic::transport::Error),

    #[error("Could not determine hostname: {0}")]
    Hostname(#[from] io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("gRPC error: {0}")]
    GrpcStatus(#[from] tonic::Status),

    #[error("Network error binding server: {0}")]
    ServerBind(io::Error),

    #[error("Error sending over channel: {0}")]
    ChannelSend(String),

    #[error("Error parsing client message: {0}")]
    MessageParse(#[from] serde_json::Error),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Other I/O error: {0}")]
    Io(io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
