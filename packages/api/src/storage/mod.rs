pub mod auth;
pub mod cache;
pub mod collections;
pub mod comments;
pub mod files;
pub mod notifications;
pub mod queue;
pub mod users;
pub mod wallpapers;

pub use auth::*;
pub use cache::*;
pub use collections::*;
pub use comments::*;
pub use files::*;
pub use notifications::*;
pub use queue::*;
#[allow(ambiguous_glob_reexports)]
pub use users::*;
#[allow(ambiguous_glob_reexports)]
pub use wallpapers::*;

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::str::FromStr;
use std::sync::OnceLock;

#[cfg(feature = "server")]
use deadpool_redis::{Config, Runtime};

pub static DB_POOL: OnceLock<PgPool> = OnceLock::new();
#[cfg(feature = "server")]
pub static REDIS_POOL: OnceLock<deadpool_redis::Pool> = OnceLock::new();

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

#[cfg(feature = "server")]
pub fn init_redis(redis_url: &str) -> anyhow::Result<deadpool_redis::Pool> {
    let cfg = Config::from_url(redis_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
    REDIS_POOL.set(pool.clone()).ok();
    Ok(pool)
}

pub fn get_pool() -> anyhow::Result<&'static PgPool> {
    DB_POOL
        .get()
        .ok_or_else(|| anyhow::anyhow!("api_err_db_uninit"))
}

#[cfg(feature = "server")]
pub fn get_redis_pool() -> anyhow::Result<&'static deadpool_redis::Pool> {
    REDIS_POOL
        .get()
        .ok_or_else(|| anyhow::anyhow!("api_err_redis_uninit"))
}
