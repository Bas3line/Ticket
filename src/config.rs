use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub discord_token: String,
    #[allow(dead_code)]
    pub application_id: u64,
    pub database_url: String,
    pub redis_url: String,
    #[allow(dead_code)]
    pub owner_id: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Config {
            discord_token: env::var("DISCORD_TOKEN")
                .context("DISCORD_TOKEN must be set")?,
            application_id: env::var("APPLICATION_ID")
                .context("APPLICATION_ID must be set")?
                .parse()
                .context("APPLICATION_ID must be a valid u64")?,
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            redis_url: env::var("REDIS_URL")
                .context("REDIS_URL must be set")?,
            owner_id: env::var("OWNER_ID")
                .context("OWNER_ID must be set")?
                .parse()
                .context("OWNER_ID must be a valid u64")?,
        })
    }
}
