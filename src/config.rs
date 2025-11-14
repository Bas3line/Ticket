use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub discord_token: String,
    #[allow(dead_code)]
    pub application_id: u64,
    pub database_url: String,
    pub backup_database_url: Option<String>,
    pub redis_url: String,
    #[allow(dead_code)]
    pub owner_id: u64,
    pub guild_webhook: Option<String>,
    pub command_webhook: Option<String>,
    pub interaction_webhook: Option<String>,
    pub postgres_webhook: Option<String>,
    pub redis_webhook: Option<String>,
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
            backup_database_url: env::var("BACKUP_DATABASE_URL").ok(),
            redis_url: env::var("REDIS_URL")
                .context("REDIS_URL must be set")?,
            owner_id: env::var("OWNER_ID")
                .context("OWNER_ID must be set")?
                .parse()
                .context("OWNER_ID must be a valid u64")?,
            guild_webhook: env::var("GUILD_WEBHOOK").ok(),
            command_webhook: env::var("COMMAND_WEBHOOK").ok(),
            interaction_webhook: env::var("INTERACTION_WEBHOOK").ok(),
            postgres_webhook: env::var("POSTGRES_WEBHOOK").ok(),
            redis_webhook: env::var("REDIS_WEBHOOK").ok(),
        })
    }
}
