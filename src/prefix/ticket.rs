use anyhow::Result;
use serenity::all::{Context, Message, Permissions};
use std::sync::Arc;
use crate::database::Database;
use crate::database::ticket as db_ticket;
use crate::utils::{create_success_embed, create_error_embed};
use crate::utils::transcript::{generate_transcript, save_transcript};

pub async fn close(ctx: &Context, msg: &Message, db: &Arc<Database>) -> Result<()> {
    let channel_id = msg.channel_id.get() as i64;

    let ticket = match db_ticket::get_ticket_by_channel(&db.pool, channel_id).await? {
        Some(t) => t,
        None => {
            let embed = create_error_embed("Not a Ticket", "This command can only be used in ticket channels");
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
            return Ok(());
        }
    };

    if !can_manage_ticket(ctx, msg, &ticket, db).await? {
        let embed = create_error_embed("Permission Denied", "You don't have permission to close this ticket");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    let messages = db_ticket::get_ticket_messages(&db.pool, ticket.id).await?;

    let owner_name = format!("<@{}>", ticket.owner_id);
    let claimed_by = ticket.claimed_by.map(|id| format!("<@{}>", id));

    let html_content = generate_transcript(
        ticket.ticket_number,
        owner_name,
        ticket.created_at,
        ticket.closed_at,
        claimed_by,
        messages,
    ).await?;
    let file_path = save_transcript(ticket.guild_id, ticket.ticket_number, html_content).await?;

    db_ticket::delete_ticket_messages(&db.pool, ticket.id).await?;

    let mut redis_conn = db.redis.clone();
    let _ = db_ticket::cleanup_priority_ping(&mut redis_conn, ticket.id).await;

    let guild = db_ticket::get_or_create_guild(&db.pool, ticket.guild_id).await?;

    if let Some(transcript_channel_id) = guild.transcript_channel_id {
        let transcript_channel = serenity::all::ChannelId::new(transcript_channel_id as u64);

        let embed = crate::utils::create_embed(
            format!("Ticket #{} Transcript", ticket.ticket_number),
            format!("Ticket Owner: <@{}>\nClosed by: <@{}>", ticket.owner_id, msg.author.id.get()),
        );

        transcript_channel.send_message(&ctx.http,
            serenity::all::CreateMessage::new()
                .embed(embed)
                .add_file(serenity::all::CreateAttachment::path(&file_path).await?)
        ).await?;

        let _ = crate::utils::transcript::delete_transcript(&file_path).await;
    }

    // Send log
    let log_embed = crate::utils::create_embed(
        "Ticket Closed",
        format!("Ticket #{}\nOwner: <@{}>\nClosed by: <@{}>\nClosed at: <t:{}:F>",
            ticket.ticket_number, ticket.owner_id, msg.author.id.get(), chrono::Utc::now().timestamp())
    );
    let _ = crate::utils::send_log(ctx, guild.log_channel_id, log_embed).await;

    db_ticket::close_ticket(&db.pool, ticket.id).await?;

    let embed = create_success_embed("Ticket Closed", "This ticket will be deleted in 5 seconds");
    msg.channel_id.send_message(&ctx.http,
        serenity::all::CreateMessage::new().embed(embed)
    ).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    msg.channel_id.delete(&ctx.http).await?;

    Ok(())
}

pub async fn claim(ctx: &Context, msg: &Message, db: &Arc<Database>) -> Result<()> {
    let channel_id = msg.channel_id.get() as i64;

    let ticket = match db_ticket::get_ticket_by_channel(&db.pool, channel_id).await? {
        Some(t) => t,
        None => {
            let embed = create_error_embed("Not a Ticket", "This command can only be used in ticket channels");
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
            return Ok(());
        }
    };

    if !is_support_staff(ctx, msg, ticket.guild_id, db).await? {
        let embed = create_error_embed("Permission Denied", "Only support staff can claim tickets");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    if ticket.is_claimed() {
        let embed = create_error_embed("Already Claimed", format!("This ticket is already claimed by <@{}>", ticket.claimed_by.unwrap()));
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    db_ticket::claim_ticket(&db.pool, ticket.id, msg.author.id.get() as i64).await?;

    // Send log
    let guild = db_ticket::get_or_create_guild(&db.pool, ticket.guild_id).await?;
    let log_embed = crate::utils::create_embed(
        "Ticket Claimed",
        format!("Ticket #{}\nClaimed by: <@{}>\nOwner: <@{}>", ticket.ticket_number, msg.author.id.get(), ticket.owner_id)
    );
    let _ = crate::utils::send_log(ctx, guild.log_channel_id, log_embed).await;

    let embed = create_success_embed("Ticket Claimed", format!("<@{}> has claimed this ticket", msg.author.id.get()));
    msg.channel_id.send_message(&ctx.http,
        serenity::all::CreateMessage::new().embed(embed)
    ).await?;

    Ok(())
}

pub async fn transcript(ctx: &Context, msg: &Message, db: &Arc<Database>) -> Result<()> {
    let channel_id = msg.channel_id.get() as i64;

    let ticket = match db_ticket::get_ticket_by_channel(&db.pool, channel_id).await? {
        Some(t) => t,
        None => {
            let embed = create_error_embed("Not a Ticket", "This command can only be used in ticket channels");
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
            return Ok(());
        }
    };

    if !can_manage_ticket(ctx, msg, &ticket, db).await? {
        let embed = create_error_embed("Permission Denied", "You don't have permission to generate transcripts");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    let messages = db_ticket::get_ticket_messages(&db.pool, ticket.id).await?;

    let owner_name = format!("<@{}>", ticket.owner_id);
    let claimed_by = ticket.claimed_by.map(|id| format!("<@{}>", id));

    let html_content = generate_transcript(
        ticket.ticket_number,
        owner_name,
        ticket.created_at,
        ticket.closed_at,
        claimed_by,
        messages,
    ).await?;
    let file_path = save_transcript(ticket.guild_id, ticket.ticket_number, html_content).await?;

    let embed = create_success_embed("Transcript Generated", format!("Ticket #{} transcript", ticket.ticket_number));
    msg.channel_id.send_message(&ctx.http,
        serenity::all::CreateMessage::new()
            .embed(embed)
            .add_file(serenity::all::CreateAttachment::path(&file_path).await?)
    ).await?;

    let _ = crate::utils::transcript::delete_transcript(&file_path).await;

    Ok(())
}

async fn can_manage_ticket(ctx: &Context, msg: &Message, ticket: &crate::models::Ticket, db: &Arc<Database>) -> Result<bool> {
    if msg.author.id.get() as i64 == ticket.owner_id {
        return Ok(true);
    }

    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(false),
    };

    let member = guild_id.member(&ctx.http, msg.author.id).await?;
    let permissions = {
        let guild_obj = ctx.cache.guild(guild_id).ok_or_else(|| anyhow::anyhow!("Guild not found"))?;
        guild_obj.member_permissions(&member)
    };

    if permissions.contains(Permissions::ADMINISTRATOR) {
        return Ok(true);
    }

    let support_roles = db_ticket::get_support_roles(&db.pool, ticket.guild_id).await?;

    for role in support_roles {
        if member.roles.contains(&serenity::all::RoleId::new(role.role_id as u64)) {
            return Ok(true);
        }
    }

    Ok(false)
}

async fn is_support_staff(ctx: &Context, msg: &Message, guild_id: i64, db: &Arc<Database>) -> Result<bool> {
    let guild_id_obj = match msg.guild_id {
        Some(id) => id,
        None => return Ok(false),
    };

    let member = guild_id_obj.member(&ctx.http, msg.author.id).await?;
    let permissions = {
        let guild_obj = ctx.cache.guild(guild_id_obj).ok_or_else(|| anyhow::anyhow!("Guild not found"))?;
        guild_obj.member_permissions(&member)
    };

    if permissions.contains(Permissions::ADMINISTRATOR) {
        return Ok(true);
    }

    let support_roles = db_ticket::get_support_roles(&db.pool, guild_id).await?;

    for role in support_roles {
        if member.roles.contains(&serenity::all::RoleId::new(role.role_id as u64)) {
            return Ok(true);
        }
    }

    Ok(false)
}
