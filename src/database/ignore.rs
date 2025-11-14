use anyhow::Result;
use sqlx::PgPool;

pub async fn add_ignored_channel(pool: &PgPool, guild_id: i64, channel_id: i64) -> Result<()> {
    sqlx::query(
        "INSERT INTO ignored_channels (guild_id, channel_id) VALUES ($1, $2) ON CONFLICT (guild_id, channel_id) DO NOTHING"
    )
    .bind(guild_id)
    .bind(channel_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_ignored_channel(pool: &PgPool, guild_id: i64, channel_id: i64) -> Result<()> {
    sqlx::query(
        "DELETE FROM ignored_channels WHERE guild_id = $1 AND channel_id = $2"
    )
    .bind(guild_id)
    .bind(channel_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn is_channel_ignored(pool: &PgPool, guild_id: i64, channel_id: i64) -> Result<bool> {
    let result: Option<(bool,)> = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM ignored_channels WHERE guild_id = $1 AND channel_id = $2)"
    )
    .bind(guild_id)
    .bind(channel_id)
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.0).unwrap_or(false))
}

pub async fn get_ignored_channels(pool: &PgPool, guild_id: i64) -> Result<Vec<i64>> {
    let channels: Vec<(i64,)> = sqlx::query_as(
        "SELECT channel_id FROM ignored_channels WHERE guild_id = $1"
    )
    .bind(guild_id)
    .fetch_all(pool)
    .await?;

    Ok(channels.into_iter().map(|c| c.0).collect())
}
