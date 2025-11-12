use anyhow::Result;
use redis::aio::ConnectionManager;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub mod ticket;

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

        let redis_client = redis::Client::open(redis_url)?;
        let redis = ConnectionManager::new(redis_client).await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool, redis })
    }
}
