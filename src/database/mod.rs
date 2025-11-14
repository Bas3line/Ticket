use anyhow::Result;
use redis::aio::ConnectionManager;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub mod ticket;
pub mod tag;
pub mod ignore;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
    #[allow(dead_code)]
    pub redis: ConnectionManager,
}

impl Database {
    pub async fn new(database_url: &str, redis_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        crate::logging::database::log_postgres_connection().await;

        let redis_client = redis::Client::open(redis_url)?;
        let redis = ConnectionManager::new(redis_client).await?;

        crate::logging::database::log_redis_connection().await;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool, redis })
    }
}
