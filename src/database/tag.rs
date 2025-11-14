use anyhow::Result;
use sqlx::PgPool;
use crate::models::Tag;

pub async fn create_tag(pool: &PgPool, guild_id: i64, name: &str, content: &str, creator_id: i64) -> Result<Tag> {
    let tag = sqlx::query_as::<_, Tag>(
        "INSERT INTO tags (guild_id, name, content, creator_id) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(guild_id)
    .bind(name)
    .bind(content)
    .bind(creator_id)
    .fetch_one(pool)
    .await?;
    Ok(tag)
}

pub async fn get_tag(pool: &PgPool, guild_id: i64, name: &str) -> Result<Option<Tag>> {
    let tag = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags WHERE guild_id = $1 AND LOWER(name) = LOWER($2)"
    )
    .bind(guild_id)
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(tag)
}

pub async fn update_tag(pool: &PgPool, guild_id: i64, name: &str, content: &str) -> Result<()> {
    sqlx::query(
        "UPDATE tags SET content = $1, updated_at = NOW() WHERE guild_id = $2 AND LOWER(name) = LOWER($3)"
    )
    .bind(content)
    .bind(guild_id)
    .bind(name)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_tag(pool: &PgPool, guild_id: i64, name: &str) -> Result<bool> {
    let result = sqlx::query(
        "DELETE FROM tags WHERE guild_id = $1 AND LOWER(name) = LOWER($2)"
    )
    .bind(guild_id)
    .bind(name)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn increment_tag_uses(pool: &PgPool, guild_id: i64, name: &str) -> Result<()> {
    sqlx::query(
        "UPDATE tags SET uses = uses + 1 WHERE guild_id = $1 AND LOWER(name) = LOWER($2)"
    )
    .bind(guild_id)
    .bind(name)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_tags(pool: &PgPool, guild_id: i64) -> Result<Vec<Tag>> {
    let tags = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags WHERE guild_id = $1 ORDER BY name ASC"
    )
    .bind(guild_id)
    .fetch_all(pool)
    .await?;
    Ok(tags)
}

pub async fn get_tag_info(pool: &PgPool, guild_id: i64, name: &str) -> Result<Option<Tag>> {
    let tag = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags WHERE guild_id = $1 AND LOWER(name) = LOWER($2)"
    )
    .bind(guild_id)
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(tag)
}

pub async fn search_tags(pool: &PgPool, guild_id: i64, query: &str) -> Result<Vec<Tag>> {
    let pattern = format!("%{}%", query);
    let tags = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags WHERE guild_id = $1 AND (LOWER(name) LIKE LOWER($2) OR LOWER(content) LIKE LOWER($2)) ORDER BY name ASC LIMIT 25"
    )
    .bind(guild_id)
    .bind(pattern)
    .fetch_all(pool)
    .await?;
    Ok(tags)
}

pub async fn get_popular_tags(pool: &PgPool, guild_id: i64, limit: i64) -> Result<Vec<Tag>> {
    let tags = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags WHERE guild_id = $1 ORDER BY uses DESC LIMIT $2"
    )
    .bind(guild_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(tags)
}

pub async fn rename_tag(pool: &PgPool, guild_id: i64, old_name: &str, new_name: &str) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE tags SET name = $1, updated_at = NOW() WHERE guild_id = $2 AND LOWER(name) = LOWER($3)"
    )
    .bind(new_name)
    .bind(guild_id)
    .bind(old_name)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
