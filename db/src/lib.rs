use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::time::Duration;

pub mod tables;
pub mod queries;

/// Opens a pool against DATABASE_URL. Dev sets this via shell.nix;
/// deployment sets the same var at the container/orchestration layer —
/// this function doesn't need to know or care which.
pub async fn init_pool() -> Result<MySqlPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set — see .env.example");

    MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
}
