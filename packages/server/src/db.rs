use crate::{AppError, Result};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::env;

pub type DbPool = Pool<Sqlite>;

pub async fn init_pool() -> Result<DbPool> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL")
        .map_err(|_| AppError::MissingEnvVar("DATABASE_URL".to_string()))?;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("Datenbank-Pool erfolgreich initialisiert.");
    Ok(pool)
}
