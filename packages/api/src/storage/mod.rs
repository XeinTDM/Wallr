pub mod auth;
pub mod cache;
pub mod collections;
pub mod comments;
pub mod files;
pub mod notifications;
pub mod users;
pub mod wallpapers;

pub use auth::*;
pub use cache::*;
pub use collections::*;
pub use comments::*;
pub use files::*;
pub use notifications::*;
pub use users::*;
pub use wallpapers::*;

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::str::FromStr;
use std::sync::OnceLock;

pub static DB_POOL: OnceLock<PgPool> = OnceLock::new();

pub async fn init_db(database_url: &str) -> Result<PgPool, sqlx::Error> {
    use sqlx::ConnectOptions;
    let options =
        sqlx::postgres::PgConnectOptions::from_str(database_url)?.disable_statement_logging();

    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect_with(options)
        .await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    DB_POOL.set(pool.clone()).ok();
    Ok(pool)
}

pub fn get_pool() -> anyhow::Result<&'static PgPool> {
    DB_POOL
        .get()
        .ok_or_else(|| anyhow::anyhow!("Database not initialized. Please call init_db first."))
}
