use crate::db::DbPool;
use sqlx;
use std::time::Duration;
use tokio::time::interval;

pub fn spawn_background_tasks(db_pool: DbPool) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            println!("\n--- Agent Health Check ---");

            let cutoff_time = (std::time::SystemTime::now() - Duration::from_secs(60))
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            let query_result = sqlx::query(
                "UPDATE agents SET status = 'offline' WHERE last_heartbeat < ? AND status = 'online'"
            )
            .bind(cutoff_time)
            .execute(&db_pool) // Verwende den übergebenen Pool
            .await;

            match query_result {
                Ok(result) => {
                    if result.rows_affected() > 0 {
                        println!("{} Agents als 'offline' markiert.", result.rows_affected());
                        // TODO: Broadcast StatsUpdate via WsClientMap?
                        // Braucht Zugriff auf WsClientMap oder einen separaten Broadcast-Channel.
                    }
                }
                Err(e) => eprintln!("Fehler beim Health-Check-Update: {}", e),
            }
        }
    });
}
