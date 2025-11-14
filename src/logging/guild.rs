use serde_json::json;
use chrono::Utc;

pub async fn log_guild_join(guild_id: u64, guild_name: &str, member_count: u64, owner_id: u64) {
    if let Some(webhook_url) = crate::logging::webhooks::get_guild_webhook() {
        let embed = json!({
            "title": "Guild Joined",
            "color": 0x57F287,
            "fields": [
                {
                    "name": "Guild Name",
                    "value": guild_name,
                    "inline": true
                },
                {
                    "name": "Guild ID",
                    "value": guild_id.to_string(),
                    "inline": true
                },
                {
                    "name": "Members",
                    "value": member_count.to_string(),
                    "inline": true
                },
                {
                    "name": "Owner ID",
                    "value": owner_id.to_string(),
                    "inline": true
                }
            ],
            "timestamp": Utc::now().to_rfc3339()
        });

        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}

pub async fn log_guild_leave(guild_id: u64, guild_name: &str, member_count: u64) {
    if let Some(webhook_url) = crate::logging::webhooks::get_guild_webhook() {
        let embed = json!({
            "title": "Guild Left",
            "color": 0xED4245,
            "fields": [
                {
                    "name": "Guild Name",
                    "value": guild_name,
                    "inline": true
                },
                {
                    "name": "Guild ID",
                    "value": guild_id.to_string(),
                    "inline": true
                },
                {
                    "name": "Members",
                    "value": member_count.to_string(),
                    "inline": true
                }
            ],
            "timestamp": Utc::now().to_rfc3339()
        });

        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}
