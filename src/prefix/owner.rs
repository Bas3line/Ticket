use anyhow::Result;
use serenity::all::{Context, Message};
use std::sync::Arc;
use sysinfo::System;
use sqlx::Row;
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
        "SELECT id, guild_id, max_servers, expires_at, created_at, created_by FROM premium WHERE expires_at > NOW() ORDER BY expires_at DESC"
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

pub async fn guilds(ctx: &Context, msg: &Message, _db: &Arc<Database>, owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_count = ctx.cache.guild_count();
    let mut guilds_text = String::new();

    for (i, guild_id) in ctx.cache.guilds().iter().enumerate() {
        if i >= 20 {
            break;
        }
        if let Some(guild) = ctx.cache.guild(*guild_id) {
            guilds_text.push_str(&format!(
                "**{}** (`{}`) - {} members\n",
                guild.name, guild_id.get(), guild.member_count
            ));
        }
    }

    if guilds_text.is_empty() {
        guilds_text = "No guilds found".to_string();
    }

    let mut embed = create_success_embed(
        &format!("Bot Guilds ({})", guild_count),
        guilds_text
    );

    if guild_count > 20 {
        embed = embed.footer(serenity::all::CreateEmbedFooter::new(
            format!("Showing 20 of {} guilds", guild_count)
        ));
    }

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn leave_guild(ctx: &Context, msg: &Message, _db: &Arc<Database>, args: &[&str], owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!leaveguild <guild_id>`");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id: u64 = args[0].parse()
        .map_err(|_| anyhow::anyhow!("Invalid guild ID"))?;

    let guild_name = ctx.cache.guild(serenity::all::GuildId::new(guild_id))
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    serenity::all::GuildId::new(guild_id).leave(&ctx.http).await?;

    let embed = create_success_embed(
        "Left Guild",
        format!("Successfully left guild: {} ({})", guild_name, guild_id),
    );

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn announce(ctx: &Context, msg: &Message, _db: &Arc<Database>, args: &[&str], owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!announce <message>`");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let announcement = args.join(" ");
    let guilds = ctx.cache.guilds();
    let mut success_count = 0;
    let mut failed_count = 0;

    for guild_id in guilds {
        if let Ok(channels) = guild_id.channels(&ctx.http).await {
            if let Some(channel) = channels.values().find(|c| {
                c.kind == serenity::all::ChannelType::Text &&
                (c.name.contains("general") || c.name.contains("announcement"))
            }) {
                let embed = crate::utils::create_embed(
                    "Bot Announcement",
                    &announcement
                )
                .color(serenity::all::Colour::from_rgb(87, 242, 135));

                if channel.id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await.is_ok() {
                    success_count += 1;
                } else {
                    failed_count += 1;
                }
            } else {
                failed_count += 1;
            }
        } else {
            failed_count += 1;
        }
    }

    let embed = create_success_embed(
        "Announcement Sent",
        format!("Success: {} guilds\nFailed: {} guilds", success_count, failed_count),
    );

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn reload_config(ctx: &Context, msg: &Message, _db: &Arc<Database>, owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let embed = create_success_embed(
        "Config Reload",
        "Configuration reloaded successfully\nNote: Restart bot to apply environment variable changes",
    );

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn eval_sql(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!sql <query>`");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let query = args.join(" ");

    if query.to_lowercase().starts_with("select") {
        match sqlx::query(&query).fetch_all(&db.pool).await {
            Ok(rows) => {
                let count = rows.len();
                let embed = create_success_embed(
                    "SQL Query Result",
                    format!("Query executed successfully\nRows returned: {}", count),
                );
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
            }
            Err(e) => {
                let embed = create_error_embed("SQL Error", &format!("Error: {}", e));
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
            }
        }
    } else {
        match sqlx::query(&query).execute(&db.pool).await {
            Ok(result) => {
                let embed = create_success_embed(
                    "SQL Query Result",
                    format!("Query executed successfully\nRows affected: {}", result.rows_affected()),
                );
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
            }
            Err(e) => {
                let embed = create_error_embed("SQL Error", &format!("Error: {}", e));
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
            }
        }
    }

    Ok(())
}

pub async fn ticket_stats(ctx: &Context, msg: &Message, db: &Arc<Database>, owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let total_tickets: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tickets"
    )
    .fetch_one(&db.pool)
    .await?;

    let open_tickets: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tickets WHERE status = 'open'"
    )
    .fetch_one(&db.pool)
    .await?;

    let closed_tickets: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tickets WHERE status = 'closed'"
    )
    .fetch_one(&db.pool)
    .await?;

    let avg_resolution_time: (Option<f64>,) = sqlx::query_as(
        "SELECT AVG(EXTRACT(EPOCH FROM (closed_at - created_at))/3600.0)
         FROM tickets
         WHERE status = 'closed' AND closed_at IS NOT NULL"
    )
    .fetch_one(&db.pool)
    .await?;

    let total_messages: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM ticket_messages"
    )
    .fetch_one(&db.pool)
    .await?;

    let embed = create_success_embed("Global Ticket Statistics", "")
        .field("Total Tickets", format!("{}", total_tickets.0), true)
        .field("Open Tickets", format!("{}", open_tickets.0), true)
        .field("Closed Tickets", format!("{}", closed_tickets.0), true)
        .field("Total Messages", format!("{}", total_messages.0), true)
        .field(
            "Avg Resolution Time",
            format!("{:.1} hours", avg_resolution_time.0.unwrap_or(0.0)),
            true
        );

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn system_info(ctx: &Context, msg: &Message, _db: &Arc<Database>, owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = sys.total_memory() as f64 / 1_073_741_824.0;
    let used_memory = sys.used_memory() as f64 / 1_073_741_824.0;
    let total_swap = sys.total_swap() as f64 / 1_073_741_824.0;
    let used_swap = sys.used_swap() as f64 / 1_073_741_824.0;

    let cpu_count = sys.cpus().len();
    let cpu_brand = sys.cpus().first().map(|c| c.brand()).unwrap_or("Unknown");

    let embed = create_success_embed("System Information", "")
        .field("CPU", format!("{} ({} cores)", cpu_brand, cpu_count), false)
        .field("Memory", format!("{:.2} GB / {:.2} GB", used_memory, total_memory), true)
        .field("Swap", format!("{:.2} GB / {:.2} GB", used_swap, total_swap), true);

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

pub async fn backup_db(ctx: &Context, msg: &Message, db: &Arc<Database>, owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = create_error_embed("Permission Denied", "This command is owner-only");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let backup_url = match std::env::var("BACKUP_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            let embed = create_error_embed(
                "Backup Not Configured",
                "BACKUP_DATABASE_URL environment variable is not set",
            );
            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
            return Ok(());
        }
    };

    let initial_embed = crate::utils::create_embed(
        "Database Backup Started",
        "Starting backup process, this may take a few minutes...",
    )
    .color(serenity::all::Colour::from_rgb(255, 165, 0));

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(initial_embed)).await?;

    let backup_pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&backup_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            let embed = create_error_embed(
                "Backup Connection Failed",
                &format!("Failed to connect to backup database: {}", e),
            );
            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
            return Ok(());
        }
    };

    let tables = vec![
        "guilds", "ticket_categories", "category_backups", "ticket_panel",
        "tickets", "ticket_messages", "support_roles", "blacklist",
        "premium", "priorities", "ticket_notes", "reminders", "tags",
        "ignored_channels"
    ];

    let mut backed_up_tables = Vec::new();
    let mut failed_tables = Vec::new();

    for table in &tables {
        match backup_table(&db.pool, &backup_pool, table).await {
            Ok(_) => backed_up_tables.push(table.to_string()),
            Err(e) => {
                failed_tables.push(format!("{}: {}", table, e));
            }
        }
    }

    let description = if failed_tables.is_empty() {
        format!(
            "Successfully backed up {} tables:\n{}",
            backed_up_tables.len(),
            backed_up_tables.join(", ")
        )
    } else {
        format!(
            "Backed up {} tables:\n{}\n\nFailed {} tables:\n{}",
            backed_up_tables.len(),
            backed_up_tables.join(", "),
            failed_tables.len(),
            failed_tables.join("\n")
        )
    };

    let embed = if failed_tables.is_empty() {
        create_success_embed("Database Backup Complete", description)
    } else {
        crate::utils::create_embed("Database Backup Partial", description)
            .color(serenity::all::Colour::from_rgb(255, 165, 0))
    };

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

async fn backup_table(source: &sqlx::PgPool, dest: &sqlx::PgPool, table_name: &str) -> Result<()> {
    let create_table_query = format!(
        "SELECT column_name, data_type, character_maximum_length, is_nullable, column_default
         FROM information_schema.columns
         WHERE table_name = '{}'
         ORDER BY ordinal_position",
        table_name
    );

    let columns: Vec<(String, String, Option<i32>, String, Option<String>)> =
        sqlx::query_as(&create_table_query)
            .fetch_all(source)
            .await?;

    if columns.is_empty() {
        return Err(anyhow::anyhow!("Table {} not found", table_name));
    }

    sqlx::query(&format!("DROP TABLE IF EXISTS {} CASCADE", table_name))
        .execute(dest)
        .await?;

    let mut create_parts = Vec::new();
    for (col_name, data_type, max_len, nullable, default_val) in &columns {
        let mut col_def = format!("{} {}", col_name, data_type.to_uppercase());

        if let Some(len) = max_len {
            if data_type.contains("char") {
                col_def = format!("{}({})", col_def, len);
            }
        }

        if nullable == "NO" {
            col_def.push_str(" NOT NULL");
        }

        if let Some(def) = default_val {
            col_def.push_str(&format!(" DEFAULT {}", def));
        }

        create_parts.push(col_def);
    }

    let create_table = format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table_name,
        create_parts.join(", ")
    );

    sqlx::query(&create_table).execute(dest).await?;

    let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(&format!("SELECT * FROM {}", table_name))
        .fetch_all(source)
        .await?;

    if rows.is_empty() {
        return Ok(());
    }

    let column_names: Vec<String> = columns.iter().map(|(name, _, _, _, _)| name.clone()).collect();
    let placeholders: Vec<String> = (1..=column_names.len())
        .map(|i| format!("${}", i))
        .collect();

    let insert_query = format!(
        "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT DO NOTHING",
        table_name,
        column_names.join(", "),
        placeholders.join(", ")
    );

    for row in rows {
        let mut query = sqlx::query(&insert_query);

        for (idx, _col_name) in column_names.iter().enumerate() {
            let value: Option<serde_json::Value> = row.try_get(idx).ok();
            query = query.bind(value);
        }

        query.execute(dest).await?;
    }

    Ok(())
}
