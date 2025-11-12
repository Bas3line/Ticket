use anyhow::Result;
use serenity::all::{Context, Message};
use std::sync::Arc;
use sysinfo::System;
use crate::database::Database;
use crate::utils::{create_success_embed, create_error_embed};

pub async fn stats(ctx: &Context, msg: &Message, db: &Arc<Database>, owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = sys.total_memory() as f64 / 1_073_741_824.0;
    let used_memory = sys.used_memory() as f64 / 1_073_741_824.0;
    let memory_usage = (used_memory / total_memory) * 100.0;

    let cpu_count = sys.cpus().len();
    let global_cpu_usage = sys.global_cpu_usage();

    let guild_count = ctx.cache.guild_count();

    let online_tickets: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tickets WHERE status = 'open'"
    )
    .fetch_one(&db.pool)
    .await?;

    let total_panels: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM ticket_panel"
    )
    .fetch_one(&db.pool)
    .await?;

    let premium_servers: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM premium WHERE expires_at > NOW()"
    )
    .fetch_one(&db.pool)
    .await?;

    let embed = create_success_embed("Bot Statistics", "")
        .field("Memory Usage", format!("{:.2} GB / {:.2} GB ({:.1}%)", used_memory, total_memory, memory_usage), false)
        .field("CPU Usage", format!("{:.1}% ({} cores)", global_cpu_usage, cpu_count), false)
        .field("Servers", format!("{}", guild_count), true)
        .field("Open Tickets", format!("{}", online_tickets.0), true)
        .field("Total Panels", format!("{}", total_panels.0), true)
        .field("Premium Servers", format!("{}", premium_servers.0), true);

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn add_premium(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if args.len() < 2 {
        let embed = create_error_embed("Invalid Usage", "Usage: `!addprem <guild_id> <days>`");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id: i64 = args[0].parse()
        .map_err(|_| anyhow::anyhow!("Invalid guild ID"))?;

    let days_str = args[1].trim_end_matches('d');
    let days: i32 = days_str.parse()
        .map_err(|_| anyhow::anyhow!("Invalid duration format. Use format like '30d'"))?;

    let max_servers = 1;

    crate::database::ticket::add_premium(&db.pool, guild_id, max_servers, days, msg.author.id.get() as i64).await?;

    let embed = create_success_embed(
        "Premium Added",
        format!("Premium added to guild {} for {} days", guild_id, days),
    );

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn remove_premium(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!removeprem <guild_id>`");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id: i64 = args[0].parse()
        .map_err(|_| anyhow::anyhow!("Invalid guild ID"))?;

    crate::database::ticket::remove_premium(&db.pool, guild_id).await?;

    let embed = create_success_embed(
        "Premium Removed",
        format!("Premium removed from guild {}", guild_id),
    );

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn list_premium(ctx: &Context, msg: &Message, db: &Arc<Database>, owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let premiums: Vec<crate::models::Premium> = sqlx::query_as(
        "SELECT * FROM premium WHERE expires_at > NOW() ORDER BY expires_at DESC"
    )
    .fetch_all(&db.pool)
    .await?;

    if premiums.is_empty() {
        let embed = create_error_embed("No Premium Servers", "There are no active premium servers");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let mut description = String::new();
    for prem in premiums.iter().take(25) {
        description.push_str(&format!(
            "Guild: `{}` | Expires: <t:{}:R>\n",
            prem.guild_id,
            prem.expires_at.timestamp()
        ));
    }

    let mut embed = create_success_embed("Premium Servers", description);
    if premiums.len() > 25 {
        embed = embed.footer(serenity::all::CreateEmbedFooter::new(
            format!("Showing 25 of {} premium servers", premiums.len())
        ));
    }

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn profile(ctx: &Context, msg: &Message, db: &Arc<Database>) -> Result<()> {
    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;

    let is_premium = crate::database::ticket::is_premium(&db.pool, guild_id).await?;

    if !is_premium {
        let embed = crate::utils::create_embed(
            "Server Profile",
            format!("This server is on the **Free Plan**\n\n**Current Limits:**\n• 1 Ticket Panel\n• Basic Features\n\n**Upgrade to Premium for:**\n• 30 Ticket Panels\n• No Limits\n• Advanced Features\n• Priority Support")
        )
        .color(serenity::all::Colour::from_rgb(255, 165, 0));

        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if let Some(premium) = crate::database::ticket::get_premium(&db.pool, guild_id).await? {
        let panel_count = crate::database::ticket::get_panel_count(&db.pool, guild_id).await?;

        let embed = create_success_embed(
            "Server Profile - Premium",
            format!(
                "This server has **Premium** status!\n\n**Benefits:**\n• Up to 30 Ticket Panels\n• No Limits on Features\n• Advanced Customization\n• Priority Support\n\n**Current Usage:**\n• Panels: {}/30\n• Expires: <t:{}:R>",
                panel_count,
                premium.expires_at.timestamp()
            )
        );

        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    }

    Ok(())
}

pub async fn blacklist_user(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!blacklistuser <user_id> [reason]`");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let user_id: i64 = args[0].parse()
        .map_err(|_| anyhow::anyhow!("Invalid user ID"))?;

    let reason = if args.len() > 1 {
        Some(args[1..].join(" "))
    } else {
        None
    };

    crate::database::ticket::add_blacklist(&db.pool, user_id, "user", reason.clone(), msg.author.id.get() as i64).await?;

    let embed = create_success_embed(
        "User Blacklisted",
        format!("User <@{}> has been blacklisted{}", user_id, if let Some(r) = reason { format!("\nReason: {}", r) } else { String::new() }),
    );

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn blacklist_guild(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!blacklistguild <guild_id> [reason]`");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id: i64 = args[0].parse()
        .map_err(|_| anyhow::anyhow!("Invalid guild ID"))?;

    let reason = if args.len() > 1 {
        Some(args[1..].join(" "))
    } else {
        None
    };

    crate::database::ticket::add_blacklist(&db.pool, guild_id, "guild", reason.clone(), msg.author.id.get() as i64).await?;

    if let Ok(guild) = ctx.http.get_guild(serenity::all::GuildId::new(guild_id as u64)).await {
        let _ = guild.id.leave(&ctx.http).await;
    }

    let embed = create_success_embed(
        "Guild Blacklisted",
        format!("Guild {} has been blacklisted and the bot has left{}", guild_id, if let Some(r) = reason { format!("\nReason: {}", r) } else { String::new() }),
    );

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn unblacklist_user(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!unblacklistuser <user_id>`");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let user_id: i64 = args[0].parse()
        .map_err(|_| anyhow::anyhow!("Invalid user ID"))?;

    crate::database::ticket::remove_blacklist(&db.pool, user_id).await?;

    let embed = create_success_embed(
        "User Unblacklisted",
        format!("User <@{}> has been removed from the blacklist", user_id),
    );

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn unblacklist_guild(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!unblacklistguild <guild_id>`");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id: i64 = args[0].parse()
        .map_err(|_| anyhow::anyhow!("Invalid guild ID"))?;

    crate::database::ticket::remove_blacklist(&db.pool, guild_id).await?;

    let embed = create_success_embed(
        "Guild Unblacklisted",
        format!("Guild {} has been removed from the blacklist", guild_id),
    );

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn list_blacklist(ctx: &Context, msg: &Message, db: &Arc<Database>, owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let blacklists = crate::database::ticket::get_all_blacklists(&db.pool, None).await?;

    if blacklists.is_empty() {
        let embed = create_error_embed("No Blacklists", "There are no blacklisted users or guilds");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let mut users_desc = String::new();
    let mut guilds_desc = String::new();

    for bl in blacklists.iter().take(50) {
        let line = format!(
            "{}: `{}` - {}\n",
            if bl.target_type == "user" { "User" } else { "Guild" },
            bl.target_id,
            bl.reason.clone().unwrap_or_else(|| "No reason".to_string())
        );

        if bl.target_type == "user" {
            users_desc.push_str(&line);
        } else {
            guilds_desc.push_str(&line);
        }
    }

    let mut embed = create_success_embed("Blacklist", "");

    if !users_desc.is_empty() {
        embed = embed.field("Blacklisted Users", users_desc, false);
    }
    if !guilds_desc.is_empty() {
        embed = embed.field("Blacklisted Guilds", guilds_desc, false);
    }

    if blacklists.len() > 50 {
        embed = embed.footer(serenity::all::CreateEmbedFooter::new(
            format!("Showing 50 of {} blacklisted entries", blacklists.len())
        ));
    }

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}
