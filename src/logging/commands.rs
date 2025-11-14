use serde_json::json;
use chrono::Utc;

pub async fn log_prefix_command(
    guild_id: Option<u64>,
    guild_name: Option<&str>,
    user_id: u64,
    username: &str,
    command: &str,
    args: &str,
) {
    let embed = json!({
        "title": "Prefix Command Executed",
        "color": 0x5865F2,
        "fields": [
            {
                "name": "Command",
                "value": format!("`{}`", command),
                "inline": true
            },
            {
                "name": "Arguments",
                "value": if args.is_empty() { "None".to_string() } else { format!("`{}`", args) },
                "inline": true
            },
            {
                "name": "User",
                "value": format!("{} ({})", username, user_id),
                "inline": false
            },
            {
                "name": "Guild",
                "value": match (guild_name, guild_id) {
                    (Some(name), Some(id)) => format!("{} ({})", name, id),
                    _ => "DM".to_string()
                },
                "inline": false
            }
        ],
        "timestamp": Utc::now().to_rfc3339()
    });

    if let Some(webhook_url) = crate::logging::webhooks::get_command_webhook() {
        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}

pub async fn log_slash_command(
    guild_id: Option<u64>,
    guild_name: Option<&str>,
    user_id: u64,
    username: &str,
    command_name: &str,
    options_str: &str,
) {
    let embed = json!({
        "title": "Slash Command Executed",
        "color": 0x5865F2,
        "fields": [
            {
                "name": "Command",
                "value": format!("`/{}`", command_name),
                "inline": true
            },
            {
                "name": "Options",
                "value": if options_str.is_empty() { "None".to_string() } else { format!("`{}`", options_str) },
                "inline": true
            },
            {
                "name": "User",
                "value": format!("{} ({})", username, user_id),
                "inline": false
            },
            {
                "name": "Guild",
                "value": match (guild_name, guild_id) {
                    (Some(name), Some(id)) => format!("{} ({})", name, id),
                    _ => "DM".to_string()
                },
                "inline": false
            }
        ],
        "timestamp": Utc::now().to_rfc3339()
    });

    if let Some(webhook_url) = crate::logging::webhooks::get_command_webhook() {
        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}
