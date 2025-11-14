pub mod transcript;

use serenity::all::{Colour, CreateEmbed, Context, ChannelId};
use anyhow::Result;

pub fn create_embed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Colour::from_rgb(88, 101, 242))
}

pub fn create_error_embed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Colour::from_rgb(237, 66, 69))
}

pub fn create_success_embed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Colour::from_rgb(87, 242, 135))
}

pub async fn send_log(
    ctx: &Context,
    log_channel_id: Option<i64>,
    embed: CreateEmbed,
) -> Result<()> {
    if let Some(channel_id) = log_channel_id {
        let channel = ChannelId::new(channel_id as u64);
        let _ = channel.send_message(
            &ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await;
    }
    Ok(())
}

pub async fn close_ticket_unified(
    ctx: &Context,
    ticket: crate::models::Ticket,
    closer_user_id: u64,
    db: &crate::database::Database,
) -> Result<()> {
    use serenity::all::UserId;

    let closed_at = chrono::Utc::now();
    let messages = crate::database::ticket::get_ticket_messages(&db.pool, ticket.id).await?;

    let owner = ctx.http.get_user(UserId::new(ticket.owner_id as u64)).await?;
    let claimed_by_name = if let Some(claimer_id) = ticket.claimed_by {
        let claimer = ctx.http.get_user(UserId::new(claimer_id as u64)).await?;
        Some(claimer.name)
    } else {
        None
    };

    let html = transcript::generate_transcript(
        ticket.ticket_number,
        owner.name,
        ticket.created_at,
        Some(closed_at),
        claimed_by_name,
        messages,
    ).await?;

    let filepath = transcript::save_transcript(ticket.guild_id, ticket.ticket_number, html).await?;

    if let Ok(guild) = sqlx::query_as::<_, crate::models::Guild>(
        "SELECT guild_id, ticket_category_id, log_channel_id, transcript_channel_id, prefix,
                claim_buttons_enabled, auto_close_hours, ticket_limit_per_user, ticket_cooldown_seconds,
                dm_on_create, embed_color, embed_title, embed_description, embed_footer,
                autoclose_enabled, autoclose_minutes, created_at, updated_at
         FROM guilds WHERE guild_id = $1"
    )
    .bind(ticket.guild_id)
    .fetch_one(&db.pool)
    .await
    {
        if let Some(transcript_channel_id) = guild.transcript_channel_id {
            let channel = ChannelId::new(transcript_channel_id as u64);
            let file = serenity::all::CreateAttachment::path(&filepath).await?;
            let embed = create_embed(
                format!("Ticket - {} Closed", ticket.owner_id),
                format!(
                    "Owner: <@{}>\nClosed by: <@{}>\nClosed at: <t:{}:F>",
                    ticket.owner_id,
                    closer_user_id,
                    closed_at.timestamp()
                ),
            );
            let _ = channel.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed).add_file(file)).await;
        }

        let owner_user = UserId::new(ticket.owner_id as u64).to_user(&ctx.http).await;
        if let Ok(user) = owner_user {
            if let Ok(dm) = user.create_dm_channel(&ctx.http).await {
                let dm_embed = create_embed(
                    "Ticket Closed - Transcript",
                    format!("Your ticket #{} has been closed. Here's the transcript.", ticket.ticket_number)
                ).color(0x5865F2);
                let dm_file = serenity::all::CreateAttachment::path(&filepath).await?;
                let _ = dm.send_message(&ctx.http,
                    serenity::all::CreateMessage::new()
                        .embed(dm_embed)
                        .add_file(dm_file)
                ).await;
            }
        }
    }

    let _ = transcript::delete_transcript(&filepath).await;

    crate::database::ticket::delete_ticket_messages(&db.pool, ticket.id).await?;

    let mut redis_conn = db.redis.clone();
    let _ = crate::database::ticket::cleanup_priority_ping(&mut redis_conn, ticket.id).await;
    let _ = crate::database::ticket::deactivate_escalation(&db.pool, ticket.id).await;

    if let Ok(guild) = crate::database::ticket::get_or_create_guild(&db.pool, ticket.guild_id).await {
        let log_embed = create_embed(
            "Ticket Closed",
            format!("Ticket: ticket-{}\nOwner: <@{}>\nClosed by: <@{}>\nClosed at: <t:{}:F>",
                ticket.owner_id, ticket.owner_id, closer_user_id, closed_at.timestamp())
        );
        let _ = send_log(ctx, guild.log_channel_id, log_embed).await;
    }

    crate::database::ticket::close_ticket(&db.pool, ticket.id).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    let channel_id = ChannelId::new(ticket.channel_id as u64);
    let _ = channel_id.delete(&ctx.http).await;

    Ok(())
}

pub async fn has_support_role_or_admin(
    ctx: &Context,
    user_id: serenity::all::UserId,
    guild_id: i64,
    db: &crate::database::Database,
) -> Result<bool> {
    use serenity::all::Permissions;

    let guild_id_obj = serenity::all::GuildId::new(guild_id as u64);
    let member = guild_id_obj.member(&ctx.http, user_id).await?;

    if let Some(guild) = ctx.cache.guild(guild_id_obj) {
        let permissions = guild.member_permissions(&member);
        if permissions.contains(Permissions::ADMINISTRATOR) {
            return Ok(true);
        }
    }

    let support_roles = crate::database::ticket::get_support_roles(&db.pool, guild_id).await?;
    let has_support_role = support_roles.iter().any(|role| {
        member.roles.contains(&serenity::all::RoleId::new(role.role_id as u64))
    });

    Ok(has_support_role)
}
