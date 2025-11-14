use serde_json::json;
use chrono::Utc;

pub async fn log_button_interaction(
    guild_id: Option<u64>,
    guild_name: Option<&str>,
    user_id: u64,
    username: &str,
    custom_id: &str,
    channel_id: u64,
) {
    let embed = json!({
        "title": "Button Interaction",
        "color": 0xFEE75C,
        "fields": [
            {
                "name": "Custom ID",
                "value": format!("`{}`", custom_id),
                "inline": true
            },
            {
                "name": "Channel",
                "value": channel_id.to_string(),
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

    if let Some(webhook_url) = crate::logging::webhooks::get_interaction_webhook() {
        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}

pub async fn log_select_menu_interaction(
    guild_id: Option<u64>,
    guild_name: Option<&str>,
    user_id: u64,
    username: &str,
    custom_id: &str,
    values: &[String],
    channel_id: u64,
) {
    let embed = json!({
        "title": "Select Menu Interaction",
        "color": 0xFEE75C,
        "fields": [
            {
                "name": "Custom ID",
                "value": format!("`{}`", custom_id),
                "inline": true
            },
            {
                "name": "Selected Values",
                "value": format!("`{}`", values.join(", ")),
                "inline": true
            },
            {
                "name": "Channel",
                "value": channel_id.to_string(),
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

    if let Some(webhook_url) = crate::logging::webhooks::get_interaction_webhook() {
        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}

pub async fn log_modal_interaction(
    guild_id: Option<u64>,
    guild_name: Option<&str>,
    user_id: u64,
    username: &str,
    custom_id: &str,
    channel_id: u64,
) {
    let embed = json!({
        "title": "Modal Interaction",
        "color": 0xFEE75C,
        "fields": [
            {
                "name": "Custom ID",
                "value": format!("`{}`", custom_id),
                "inline": true
            },
            {
                "name": "Channel",
                "value": channel_id.to_string(),
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

    if let Some(webhook_url) = crate::logging::webhooks::get_interaction_webhook() {
        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}
