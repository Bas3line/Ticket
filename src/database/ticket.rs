use crate::models::{Guild, SupportRole, Ticket, TicketCategory, TicketMessage, TicketPanel};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_or_create_guild(pool: &PgPool, guild_id: i64) -> Result<Guild> {
    let guild = sqlx::query_as::<_, Guild>(
        "INSERT INTO guilds (guild_id) VALUES ($1)
         ON CONFLICT (guild_id) DO UPDATE SET guild_id = guilds.guild_id
         RETURNING *"
    )
    .bind(guild_id)
    .fetch_one(pool)
    .await?;

    Ok(guild)
}

pub async fn update_guild_category(pool: &PgPool, guild_id: i64, category_id: i64) -> Result<()> {
    sqlx::query("UPDATE guilds SET ticket_category_id = $1 WHERE guild_id = $2")
        .bind(category_id)
        .bind(guild_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_guild_log_channel(pool: &PgPool, guild_id: i64, channel_id: i64) -> Result<()> {
    sqlx::query("UPDATE guilds SET log_channel_id = $1 WHERE guild_id = $2")
        .bind(channel_id)
        .bind(guild_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_guild_transcript_channel(pool: &PgPool, guild_id: i64, channel_id: i64) -> Result<()> {
    sqlx::query("UPDATE guilds SET transcript_channel_id = $1 WHERE guild_id = $2")
        .bind(channel_id)
        .bind(guild_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_guild_prefix(pool: &PgPool, guild_id: i64) -> Result<String> {
    let result: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT prefix FROM guilds WHERE guild_id = $1"
    )
    .bind(guild_id)
    .fetch_optional(pool)
    .await?;

    Ok(result.and_then(|(prefix,)| prefix).unwrap_or_else(|| "!".to_string()))
}

pub async fn set_guild_prefix(pool: &PgPool, guild_id: i64, prefix: String) -> Result<()> {
    sqlx::query("UPDATE guilds SET prefix = $1 WHERE guild_id = $2")
        .bind(prefix)
        .bind(guild_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn create_ticket_category(
    pool: &PgPool,
    guild_id: i64,
    name: String,
    description: Option<String>,
    emoji: Option<String>,
) -> Result<TicketCategory> {
    let category = sqlx::query_as::<_, TicketCategory>(
        "INSERT INTO ticket_categories (guild_id, name, description, emoji)
         VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(guild_id)
    .bind(name)
    .bind(description)
    .bind(emoji)
    .fetch_one(pool)
    .await?;

    Ok(category)
}

pub async fn get_ticket_categories(pool: &PgPool, guild_id: i64) -> Result<Vec<TicketCategory>> {
    let categories = sqlx::query_as::<_, TicketCategory>(
        "SELECT * FROM ticket_categories WHERE guild_id = $1 ORDER BY created_at ASC"
    )
    .bind(guild_id)
    .fetch_all(pool)
    .await?;

    Ok(categories)
}

#[allow(dead_code)]
pub async fn delete_ticket_category(pool: &PgPool, category_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM ticket_categories WHERE id = $1")
        .bind(category_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn add_support_role(pool: &PgPool, guild_id: i64, role_id: i64) -> Result<SupportRole> {
    let role = sqlx::query_as::<_, SupportRole>(
        "INSERT INTO support_roles (guild_id, role_id) VALUES ($1, $2)
         ON CONFLICT (guild_id, role_id) DO UPDATE SET role_id = support_roles.role_id
         RETURNING *"
    )
    .bind(guild_id)
    .bind(role_id)
    .fetch_one(pool)
    .await?;

    Ok(role)
}

pub async fn remove_support_role(pool: &PgPool, guild_id: i64, role_id: i64) -> Result<()> {
    sqlx::query("DELETE FROM support_roles WHERE guild_id = $1 AND role_id = $2")
        .bind(guild_id)
        .bind(role_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_support_roles(pool: &PgPool, guild_id: i64) -> Result<Vec<SupportRole>> {
    let roles = sqlx::query_as::<_, SupportRole>(
        "SELECT * FROM support_roles WHERE guild_id = $1"
    )
    .bind(guild_id)
    .fetch_all(pool)
    .await?;

    Ok(roles)
}

pub async fn create_ticket(
    pool: &PgPool,
    guild_id: i64,
    channel_id: i64,
    owner_id: i64,
    category_id: Option<Uuid>,
) -> Result<Ticket> {
    let ticket_number = get_next_ticket_number(pool, guild_id).await?;

    let ticket = sqlx::query_as::<_, Ticket>(
        "INSERT INTO tickets (guild_id, channel_id, ticket_number, owner_id, category_id)
         VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(guild_id)
    .bind(channel_id)
    .bind(ticket_number)
    .bind(owner_id)
    .bind(category_id)
    .fetch_one(pool)
    .await?;

    Ok(ticket)
}

async fn get_next_ticket_number(pool: &PgPool, guild_id: i64) -> Result<i32> {
    let result: Option<(Option<i32>,)> = sqlx::query_as(
        "SELECT MAX(ticket_number) FROM tickets WHERE guild_id = $1"
    )
    .bind(guild_id)
    .fetch_optional(pool)
    .await?;

    Ok(result.and_then(|(num,)| num).unwrap_or(0) + 1)
}

pub async fn get_ticket_by_channel(pool: &PgPool, channel_id: i64) -> Result<Option<Ticket>> {
    let ticket = sqlx::query_as::<_, Ticket>(
        "SELECT * FROM tickets WHERE channel_id = $1"
    )
    .bind(channel_id)
    .fetch_optional(pool)
    .await?;

    Ok(ticket)
}

#[allow(dead_code)]
pub async fn get_ticket_by_id(pool: &PgPool, ticket_id: Uuid) -> Result<Option<Ticket>> {
    let ticket = sqlx::query_as::<_, Ticket>(
        "SELECT * FROM tickets WHERE id = $1"
    )
    .bind(ticket_id)
    .fetch_optional(pool)
    .await?;

    Ok(ticket)
}

pub async fn get_user_tickets(pool: &PgPool, guild_id: i64, user_id: i64) -> Result<Vec<Ticket>> {
    let tickets = sqlx::query_as::<_, Ticket>(
        "SELECT * FROM tickets WHERE guild_id = $1 AND owner_id = $2 AND status = 'open'"
    )
    .bind(guild_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(tickets)
}

pub async fn claim_ticket(pool: &PgPool, ticket_id: Uuid, claimer_id: i64) -> Result<()> {
    sqlx::query("UPDATE tickets SET claimed_by = $1 WHERE id = $2")
        .bind(claimer_id)
        .bind(ticket_id)
        .execute(pool)
        .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn unclaim_ticket(pool: &PgPool, ticket_id: Uuid) -> Result<()> {
    sqlx::query("UPDATE tickets SET claimed_by = NULL WHERE id = $1")
        .bind(ticket_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn close_ticket(pool: &PgPool, ticket_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM tickets WHERE id = $1")
        .bind(ticket_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn cleanup_priority_ping(redis: &mut redis::aio::ConnectionManager, ticket_id: Uuid) -> Result<()> {
    let redis_key = format!("priority_ping:{}", ticket_id);
    let _: () = redis::cmd("DEL")
        .arg(&redis_key)
        .query_async(redis)
        .await
        .unwrap_or(());

    Ok(())
}

pub async fn add_ticket_message(
    pool: &PgPool,
    ticket_id: Uuid,
    message_id: i64,
    author_id: i64,
    author_name: String,
    author_discriminator: Option<String>,
    author_avatar_url: Option<String>,
    content: String,
    attachments: serde_json::Value,
) -> Result<TicketMessage> {
    let message = sqlx::query_as::<_, TicketMessage>(
        "INSERT INTO ticket_messages
         (ticket_id, message_id, author_id, author_name, author_discriminator, author_avatar_url, content, attachments)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"
    )
    .bind(ticket_id)
    .bind(message_id)
    .bind(author_id)
    .bind(author_name)
    .bind(author_discriminator)
    .bind(author_avatar_url)
    .bind(content)
    .bind(attachments)
    .fetch_one(pool)
    .await?;

    Ok(message)
}

pub async fn get_ticket_messages(pool: &PgPool, ticket_id: Uuid) -> Result<Vec<TicketMessage>> {
    let messages = sqlx::query_as::<_, TicketMessage>(
        "SELECT * FROM ticket_messages WHERE ticket_id = $1 ORDER BY created_at ASC"
    )
    .bind(ticket_id)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

pub async fn create_ticket_panel(
    pool: &PgPool,
    guild_id: i64,
    channel_id: i64,
    message_id: i64,
    title: String,
    description: Option<String>,
) -> Result<TicketPanel> {
    let panel = sqlx::query_as::<_, TicketPanel>(
        "INSERT INTO ticket_panel (guild_id, channel_id, message_id, title, description)
         VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(guild_id)
    .bind(channel_id)
    .bind(message_id)
    .bind(title)
    .bind(description)
    .fetch_one(pool)
    .await?;

    Ok(panel)
}

#[allow(dead_code)]
pub async fn get_ticket_panel(pool: &PgPool, message_id: i64) -> Result<Option<TicketPanel>> {
    let panel = sqlx::query_as::<_, TicketPanel>(
        "SELECT * FROM ticket_panel WHERE message_id = $1"
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    Ok(panel)
}

pub async fn get_panel_count(pool: &PgPool, guild_id: i64) -> Result<i64> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM ticket_panel WHERE guild_id = $1"
    )
    .bind(guild_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

pub async fn add_premium(
    pool: &PgPool,
    guild_id: i64,
    max_servers: i32,
    duration_days: i32,
    created_by: i64,
) -> Result<crate::models::Premium> {
    let expires_at = chrono::Utc::now() + chrono::Duration::days(duration_days as i64);

    let premium = sqlx::query_as::<_, crate::models::Premium>(
        "INSERT INTO premium (guild_id, max_servers, expires_at, created_by)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (guild_id) DO UPDATE
         SET max_servers = $2, expires_at = $3
         RETURNING *"
    )
    .bind(guild_id)
    .bind(max_servers)
    .bind(expires_at)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    Ok(premium)
}

pub async fn get_premium(pool: &PgPool, guild_id: i64) -> Result<Option<crate::models::Premium>> {
    let premium = sqlx::query_as::<_, crate::models::Premium>(
        "SELECT * FROM premium WHERE guild_id = $1 AND expires_at > NOW()"
    )
    .bind(guild_id)
    .fetch_optional(pool)
    .await?;

    Ok(premium)
}

pub async fn remove_premium(pool: &PgPool, guild_id: i64) -> Result<()> {
    sqlx::query("DELETE FROM premium WHERE guild_id = $1")
        .bind(guild_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn is_premium(pool: &PgPool, guild_id: i64) -> Result<bool> {
    let premium = get_premium(pool, guild_id).await?;
    Ok(premium.is_some())
}

pub async fn update_guild_settings(
    pool: &PgPool,
    guild_id: i64,
    claim_buttons: Option<bool>,
    auto_close: Option<i32>,
    ticket_limit: Option<i32>,
    cooldown: Option<i32>,
    dm_on_create: Option<bool>,
) -> Result<()> {
    let mut query = String::from("UPDATE guilds SET ");
    let mut updates = Vec::new();
    let mut bind_count = 1;

    if claim_buttons.is_some() {
        updates.push(format!("claim_buttons_enabled = ${}", bind_count));
        bind_count += 1;
    }
    if auto_close.is_some() {
        updates.push(format!("auto_close_hours = ${}", bind_count));
        bind_count += 1;
    }
    if ticket_limit.is_some() {
        updates.push(format!("ticket_limit_per_user = ${}", bind_count));
        bind_count += 1;
    }
    if cooldown.is_some() {
        updates.push(format!("ticket_cooldown_seconds = ${}", bind_count));
        bind_count += 1;
    }
    if dm_on_create.is_some() {
        updates.push(format!("dm_on_create = ${}", bind_count));
        bind_count += 1;
    }

    if updates.is_empty() {
        return Ok(());
    }

    query.push_str(&updates.join(", "));
    query.push_str(&format!(" WHERE guild_id = ${}", bind_count));

    let mut q = sqlx::query(&query);

    if let Some(val) = claim_buttons {
        q = q.bind(val);
    }
    if let Some(val) = auto_close {
        q = q.bind(val);
    }
    if let Some(val) = ticket_limit {
        q = q.bind(val);
    }
    if let Some(val) = cooldown {
        q = q.bind(val);
    }
    if let Some(val) = dm_on_create {
        q = q.bind(val);
    }

    q = q.bind(guild_id);
    q.execute(pool).await?;

    Ok(())
}

pub async fn update_embed_settings(
    pool: &PgPool,
    guild_id: i64,
    color: Option<i32>,
    title: Option<String>,
    description: Option<String>,
    footer: Option<String>,
) -> Result<()> {
    let mut query = String::from("UPDATE guilds SET ");
    let mut updates = Vec::new();
    let mut bind_count = 1;

    if color.is_some() {
        updates.push(format!("embed_color = ${}", bind_count));
        bind_count += 1;
    }
    if title.is_some() {
        updates.push(format!("embed_title = ${}", bind_count));
        bind_count += 1;
    }
    if description.is_some() {
        updates.push(format!("embed_description = ${}", bind_count));
        bind_count += 1;
    }
    if footer.is_some() {
        updates.push(format!("embed_footer = ${}", bind_count));
        bind_count += 1;
    }

    if updates.is_empty() {
        return Ok(());
    }

    query.push_str(&updates.join(", "));
    query.push_str(&format!(" WHERE guild_id = ${}", bind_count));

    let mut q = sqlx::query(&query);

    if let Some(val) = color {
        q = q.bind(val);
    }
    if let Some(val) = title {
        q = q.bind(val);
    }
    if let Some(val) = description {
        q = q.bind(val);
    }
    if let Some(val) = footer {
        q = q.bind(val);
    }

    q = q.bind(guild_id);
    q.execute(pool).await?;

    Ok(())
}

pub async fn add_blacklist(
    pool: &PgPool,
    target_id: i64,
    target_type: &str,
    reason: Option<String>,
    blacklisted_by: i64,
) -> Result<crate::models::Blacklist> {
    let blacklist = sqlx::query_as::<_, crate::models::Blacklist>(
        "INSERT INTO blacklist (target_id, target_type, reason, blacklisted_by)
         VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(target_id)
    .bind(target_type)
    .bind(reason)
    .bind(blacklisted_by)
    .fetch_one(pool)
    .await?;

    Ok(blacklist)
}

pub async fn remove_blacklist(pool: &PgPool, target_id: i64) -> Result<()> {
    sqlx::query("DELETE FROM blacklist WHERE target_id = $1")
        .bind(target_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn is_blacklisted(pool: &PgPool, target_id: i64, target_type: &str) -> Result<bool> {
    let result: Option<(i64,)> = sqlx::query_as(
        "SELECT target_id FROM blacklist WHERE target_id = $1 AND target_type = $2"
    )
    .bind(target_id)
    .bind(target_type)
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

pub async fn get_all_blacklists(pool: &PgPool, target_type: Option<&str>) -> Result<Vec<crate::models::Blacklist>> {
    let blacklists = if let Some(t_type) = target_type {
        sqlx::query_as::<_, crate::models::Blacklist>(
            "SELECT * FROM blacklist WHERE target_type = $1 ORDER BY created_at DESC"
        )
        .bind(t_type)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, crate::models::Blacklist>(
            "SELECT * FROM blacklist ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?
    };

    Ok(blacklists)
}

pub async fn delete_ticket_messages(pool: &PgPool, ticket_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM ticket_messages WHERE ticket_id = $1")
        .bind(ticket_id)
        .execute(pool)
        .await?;

    Ok(())
}
