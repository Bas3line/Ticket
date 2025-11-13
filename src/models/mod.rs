use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Guild {
    #[allow(dead_code)]
    pub guild_id: i64,
    pub ticket_category_id: Option<i64>,
    pub log_channel_id: Option<i64>,
    pub transcript_channel_id: Option<i64>,
    #[allow(dead_code)]
    pub prefix: Option<String>,
    pub claim_buttons_enabled: Option<bool>,
    #[allow(dead_code)]
    pub auto_close_hours: Option<i32>,
    #[allow(dead_code)]
    pub ticket_limit_per_user: Option<i32>,
    #[allow(dead_code)]
    pub ticket_cooldown_seconds: Option<i32>,
    #[allow(dead_code)]
    pub dm_on_create: Option<bool>,
    #[allow(dead_code)]
    pub embed_color: Option<i32>,
    #[allow(dead_code)]
    pub embed_title: Option<String>,
    #[allow(dead_code)]
    pub embed_description: Option<String>,
    #[allow(dead_code)]
    pub embed_footer: Option<String>,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
    #[allow(dead_code)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Premium {
    #[allow(dead_code)]
    pub id: Uuid,
    pub guild_id: i64,
    #[allow(dead_code)]
    pub max_servers: i32,
    pub expires_at: DateTime<Utc>,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
    #[allow(dead_code)]
    pub created_by: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct TicketCategory {
    #[allow(dead_code)]
    pub id: Uuid,
    #[allow(dead_code)]
    pub guild_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
    #[allow(dead_code)]
    pub use_custom_welcome: Option<bool>,
    #[allow(dead_code)]
    pub custom_welcome_message: Option<String>,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SupportRole {
    #[allow(dead_code)]
    pub id: Uuid,
    #[allow(dead_code)]
    pub guild_id: i64,
    pub role_id: i64,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Ticket {
    pub id: Uuid,
    pub guild_id: i64,
    pub channel_id: i64,
    pub ticket_number: i32,
    pub owner_id: i64,
    #[allow(dead_code)]
    pub category_id: Option<Uuid>,
    pub claimed_by: Option<i64>,
    #[allow(dead_code)]
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    pub priority: Option<String>,
    #[allow(dead_code)]
    pub rating: Option<i32>,
    #[allow(dead_code)]
    pub last_activity: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    pub opening_message_id: Option<i64>,
    #[allow(dead_code)]
    pub has_messages: Option<bool>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TicketMessage {
    #[allow(dead_code)]
    pub id: Uuid,
    #[allow(dead_code)]
    pub ticket_id: Uuid,
    #[allow(dead_code)]
    pub message_id: i64,
    #[allow(dead_code)]
    pub author_id: i64,
    pub author_name: String,
    #[allow(dead_code)]
    pub author_discriminator: Option<String>,
    pub author_avatar_url: Option<String>,
    pub content: String,
    pub attachments: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TicketPanel {
    #[allow(dead_code)]
    pub id: Uuid,
    #[allow(dead_code)]
    pub guild_id: i64,
    #[allow(dead_code)]
    pub channel_id: i64,
    #[allow(dead_code)]
    pub message_id: i64,
    #[allow(dead_code)]
    pub title: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub url: String,
    pub size: u64,
}

impl Ticket {
    #[allow(dead_code)]
    pub fn is_open(&self) -> bool {
        self.status == "open"
    }

    pub fn is_claimed(&self) -> bool {
        self.claimed_by.is_some()
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Blacklist {
    #[allow(dead_code)]
    pub id: Uuid,
    pub target_id: i64,
    pub target_type: String,
    pub reason: Option<String>,
    #[allow(dead_code)]
    pub blacklisted_by: i64,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Reminder {
    pub id: Uuid,
    pub user_id: i64,
    pub channel_id: i64,
    pub guild_id: Option<i64>,
    pub message_id: Option<i64>,
    pub reason: String,
    pub remind_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub completed: bool,
}
