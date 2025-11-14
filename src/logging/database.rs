use serde_json::json;
use chrono::Utc;

pub async fn log_postgres_connection() {
    if let Some(webhook_url) = crate::logging::webhooks::get_postgres_webhook() {
        let embed = json!({
            "title": "PostgreSQL Connection Established",
            "color": 0x336791,
            "description": "Successfully connected to PostgreSQL database",
            "timestamp": Utc::now().to_rfc3339()
        });

        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}

pub async fn log_postgres_query(query_type: &str, table: &str, duration_ms: u64) {
    if let Some(webhook_url) = crate::logging::webhooks::get_postgres_webhook() {
        let embed = json!({
            "title": "PostgreSQL Query",
            "color": 0x336791,
            "fields": [
                {
                    "name": "Query Type",
                    "value": query_type,
                    "inline": true
                },
                {
                    "name": "Table",
                    "value": table,
                    "inline": true
                },
                {
                    "name": "Duration",
                    "value": format!("{}ms", duration_ms),
                    "inline": true
                }
            ],
            "timestamp": Utc::now().to_rfc3339()
        });

        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}

pub async fn log_postgres_error(error: &str, query: &str) {
    if let Some(webhook_url) = crate::logging::webhooks::get_postgres_webhook() {
        let embed = json!({
            "title": "PostgreSQL Error",
            "color": 0xED4245,
            "fields": [
                {
                    "name": "Error",
                    "value": error,
                    "inline": false
                },
                {
                    "name": "Query",
                    "value": format!("`{}`", query),
                    "inline": false
                }
            ],
            "timestamp": Utc::now().to_rfc3339()
        });

        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}

pub async fn log_redis_connection() {
    if let Some(webhook_url) = crate::logging::webhooks::get_redis_webhook() {
        let embed = json!({
            "title": "Redis Connection Established",
            "color": 0xDC382D,
            "description": "Successfully connected to Redis",
            "timestamp": Utc::now().to_rfc3339()
        });

        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}

pub async fn log_redis_operation(operation: &str, key: &str, duration_ms: u64) {
    if let Some(webhook_url) = crate::logging::webhooks::get_redis_webhook() {
        let embed = json!({
            "title": "Redis Operation",
            "color": 0xDC382D,
            "fields": [
                {
                    "name": "Operation",
                    "value": operation,
                    "inline": true
                },
                {
                    "name": "Key",
                    "value": format!("`{}`", key),
                    "inline": true
                },
                {
                    "name": "Duration",
                    "value": format!("{}ms", duration_ms),
                    "inline": true
                }
            ],
            "timestamp": Utc::now().to_rfc3339()
        });

        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}

pub async fn log_redis_error(error: &str, operation: &str) {
    if let Some(webhook_url) = crate::logging::webhooks::get_redis_webhook() {
        let embed = json!({
            "title": "Redis Error",
            "color": 0xED4245,
            "fields": [
                {
                    "name": "Error",
                    "value": error,
                    "inline": false
                },
                {
                    "name": "Operation",
                    "value": format!("`{}`", operation),
                    "inline": false
                }
            ],
            "timestamp": Utc::now().to_rfc3339()
        });

        crate::logging::webhooks::send_webhook(webhook_url, vec![embed]).await;
    }
}
