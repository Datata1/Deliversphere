use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::env;

pub type DbPool = Pool<Sqlite>;

pub async fn init_pool() -> Result<DbPool, sqlx::Error> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL muss gesetzt sein");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("Datenbank-Pool erfolgreich initialisiert.");
    Ok(pool)
}