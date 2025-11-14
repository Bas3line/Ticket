use reqwest::Client;
use serde_json::json;
use std::sync::OnceLock;

static GUILD_WEBHOOK: OnceLock<String> = OnceLock::new();
static COMMAND_WEBHOOK: OnceLock<String> = OnceLock::new();
static INTERACTION_WEBHOOK: OnceLock<String> = OnceLock::new();
static POSTGRES_WEBHOOK: OnceLock<String> = OnceLock::new();
static REDIS_WEBHOOK: OnceLock<String> = OnceLock::new();

pub fn init_webhooks(config: &crate::config::Config) {
    if let Some(url) = &config.guild_webhook {
        let _ = GUILD_WEBHOOK.set(url.clone());
    }
    if let Some(url) = &config.command_webhook {
        let _ = COMMAND_WEBHOOK.set(url.clone());
    }
    if let Some(url) = &config.interaction_webhook {
        let _ = INTERACTION_WEBHOOK.set(url.clone());
    }
    if let Some(url) = &config.postgres_webhook {
        let _ = POSTGRES_WEBHOOK.set(url.clone());
    }
    if let Some(url) = &config.redis_webhook {
        let _ = REDIS_WEBHOOK.set(url.clone());
    }
}

pub fn get_guild_webhook() -> Option<&'static str> {
    GUILD_WEBHOOK.get().map(|s| s.as_str())
}

pub fn get_command_webhook() -> Option<&'static str> {
    COMMAND_WEBHOOK.get().map(|s| s.as_str())
}

pub fn get_interaction_webhook() -> Option<&'static str> {
    INTERACTION_WEBHOOK.get().map(|s| s.as_str())
}

pub fn get_postgres_webhook() -> Option<&'static str> {
    POSTGRES_WEBHOOK.get().map(|s| s.as_str())
}

pub fn get_redis_webhook() -> Option<&'static str> {
    REDIS_WEBHOOK.get().map(|s| s.as_str())
}

pub async fn send_webhook(webhook_url: &str, embeds: Vec<serde_json::Value>) {
    let webhook_url = webhook_url.to_string();
    tokio::spawn(async move {
        let client = Client::new();
        let payload = json!({
            "embeds": embeds
        });

        let _ = client.post(&webhook_url)
            .json(&payload)
            .send()
            .await;
    });
}
