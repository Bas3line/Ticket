use anyhow::Result;
use serenity::all::{Context, Message, Permissions, CreateEmbed, CreateButton, CreateActionRow, ButtonStyle};
use std::sync::Arc;
use crate::database::Database;
use crate::database::ticket as db_ticket;
use crate::utils::{create_success_embed, create_error_embed, create_embed};

pub async fn supportrole(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
    if !has_admin_permissions(ctx, msg).await? {
        let embed = create_error_embed("Permission Denied", "You need Administrator permission to use this command");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!supportrole <add|remove|list> [role]`");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;

    match args[0] {
        "add" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Role", "Usage: `!supportrole add <role>`");
                msg.channel_id.send_message(&ctx.http,
                    serenity::all::CreateMessage::new().embed(embed)
                ).await?;
                return Ok(());
            }

            let role_id = parse_role_id(args[1])?;
            db_ticket::add_support_role(&db.pool, guild_id, role_id).await?;

            let embed = create_success_embed("Support Role Added", format!("<@&{}> has been added as a support role", role_id));
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        "remove" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Role", "Usage: `!supportrole remove <role>`");
                msg.channel_id.send_message(&ctx.http,
                    serenity::all::CreateMessage::new().embed(embed)
                ).await?;
                return Ok(());
            }

            let role_id = parse_role_id(args[1])?;
            db_ticket::remove_support_role(&db.pool, guild_id, role_id).await?;

            let embed = create_success_embed("Support Role Removed", format!("<@&{}> has been removed from support roles", role_id));
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        "list" => {
            let roles = db_ticket::get_support_roles(&db.pool, guild_id).await?;

            let description = if roles.is_empty() {
                "No support roles configured".to_string()
            } else {
                roles.iter()
                    .map(|r| format!("<@&{}>", r.role_id))
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            let embed = create_embed("Support Roles", description);
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        _ => {
            let embed = create_error_embed("Invalid Subcommand", "Valid subcommands: `add`, `remove`, `list`");
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
    }

    Ok(())
}

pub async fn category(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
    if !has_admin_permissions(ctx, msg).await? {
        let embed = create_error_embed("Permission Denied", "You need Administrator permission to use this command");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!category <add|list> [name] [description] [emoji]`");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;

    match args[0] {
        "add" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Name", "Usage: `!category add <name> [description] [emoji]`");
                msg.channel_id.send_message(&ctx.http,
                    serenity::all::CreateMessage::new().embed(embed)
                ).await?;
                return Ok(());
            }

            let name = args[1].to_string();
            let description = if args.len() > 2 { Some(args[2..].join(" ")) } else { None };
            let emoji = None;

            db_ticket::create_ticket_category(&db.pool, guild_id, name.clone(), description, emoji).await?;

            let embed = create_success_embed("Category Created", format!("Category '{}' has been created", name));
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        "list" => {
            let categories = db_ticket::get_ticket_categories(&db.pool, guild_id).await?;

            let description = if categories.is_empty() {
                "No categories configured".to_string()
            } else {
                categories.iter()
                    .map(|c| {
                        let desc = c.description.as_ref()
                            .map(|d| format!(": {}", d))
                            .unwrap_or_default();
                        format!("**{}**{}", c.name, desc)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            let embed = create_embed("Ticket Categories", description);
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        _ => {
            let embed = create_error_embed("Invalid Subcommand", "Valid subcommands: `add`, `list`");
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
    }

    Ok(())
}

pub async fn panel(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
    if !has_admin_permissions(ctx, msg).await? {
        let embed = create_error_embed("Permission Denied", "You need Administrator permission to use this command");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Missing Title", "Usage: `!panel <title> [description]`");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;
    let title = args[0].to_string();
    let description = if args.len() > 1 {
        Some(args[1..].join(" "))
    } else {
        None
    };

    let embed = CreateEmbed::new()
        .title(&title)
        .description(description.as_deref().unwrap_or("Click the button below to create a ticket"))
        .color(0x5865F2);

    let button = CreateButton::new("ticket_create")
        .label("Create Ticket")
        .style(ButtonStyle::Primary);

    let sent_msg = msg.channel_id.send_message(&ctx.http,
        serenity::all::CreateMessage::new()
            .embed(embed)
            .components(vec![CreateActionRow::Buttons(vec![button])])
    ).await?;

    db_ticket::create_ticket_panel(
        &db.pool,
        guild_id,
        msg.channel_id.get() as i64,
        sent_msg.id.get() as i64,
        title,
        description,
    ).await?;

    let success_embed = create_success_embed("Panel Created", "Ticket panel has been created successfully");
    msg.channel_id.send_message(&ctx.http,
        serenity::all::CreateMessage::new().embed(success_embed)
    ).await?;

    Ok(())
}

pub async fn priority(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
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
        let embed = create_error_embed("Permission Denied", "Only support staff can set ticket priority");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Missing Priority", "Usage: `!priority <low|normal|high|urgent>`");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    let priority = match args[0].to_lowercase().as_str() {
        "low" => "low",
        "normal" => "normal",
        "high" => "high",
        "urgent" => "urgent",
        _ => {
            let embed = create_error_embed("Invalid Priority", "Valid priorities: `low`, `normal`, `high`, `urgent`");
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
            return Ok(());
        }
    };

    sqlx::query("UPDATE tickets SET priority = $1 WHERE id = $2")
        .bind(priority)
        .bind(ticket.id)
        .execute(&db.pool)
        .await?;

    // Handle auto-ping for low/high/urgent priorities
    if priority == "low" || priority == "high" || priority == "urgent" {
        let redis_key = format!("priority_ping:{}", ticket.id);
        let mut redis_conn = db.redis.clone();
        let _: () = redis::cmd("SETEX")
            .arg(&redis_key)
            .arg(86400)
            .arg(priority)
            .query_async(&mut redis_conn)
            .await
            .unwrap_or(());

        let ctx_clone = ctx.clone();
        let channel_id = msg.channel_id;
        let ticket_id = ticket.id;
        let db_clone = db.clone();
        let guild_id = ticket.guild_id;
        let priority_string = priority.to_string();

        tokio::spawn(async move {
            let interval_secs = match priority_string.as_str() {
                "low" => 7200,      // 120 minutes
                "high" => 3600,     // 60 minutes
                "urgent" => 3600,   // 60 minutes
                _ => 3600,
            };

            // Ping immediately for high/urgent
            if priority_string == "high" || priority_string == "urgent" {
                if let Ok(Some((ping_role_id,))) = sqlx::query_as::<_, (Option<i64>,)>(
                    "SELECT ping_role_id FROM guilds WHERE guild_id = $1"
                )
                .bind(guild_id)
                .fetch_optional(&db_clone.pool)
                .await
                {
                    if let Some(role_id) = ping_role_id {
                        let priority_label = if priority_string == "urgent" { "URGENT" } else { "High priority" };
                        let _ = channel_id.send_message(
                            &ctx_clone.http,
                            serenity::all::CreateMessage::new()
                                .content(format!("<@&{}> {} ticket requires attention!", role_id, priority_label))
                        ).await;
                    }
                }
            }

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;

                let mut redis_conn = db_clone.redis.clone();
                let exists: Option<String> = redis::cmd("GET")
                    .arg(&format!("priority_ping:{}", ticket_id))
                    .query_async(&mut redis_conn)
                    .await
                    .ok();

                if exists.is_none() {
                    break;
                }

                let ticket_check = sqlx::query_as::<_, crate::models::Ticket>(
                    "SELECT * FROM tickets WHERE id = $1"
                )
                .bind(ticket_id)
                .fetch_optional(&db_clone.pool)
                .await;

                if ticket_check.is_err() || ticket_check.unwrap().is_none() {
                    let _: () = redis::cmd("DEL")
                        .arg(&format!("priority_ping:{}", ticket_id))
                        .query_async(&mut redis_conn)
                        .await
                        .unwrap_or(());
                    break;
                }

                if let Ok(Some((ping_role_id,))) = sqlx::query_as::<_, (Option<i64>,)>(
                    "SELECT ping_role_id FROM guilds WHERE guild_id = $1"
                )
                .bind(guild_id)
                .fetch_optional(&db_clone.pool)
                .await
                {
                    if let Some(role_id) = ping_role_id {
                        let priority_label = match priority_string.as_str() {
                            "urgent" => "URGENT",
                            "high" => "High priority",
                            "low" => "Low priority",
                            _ => "Priority",
                        };
                        let _ = channel_id.send_message(
                            &ctx_clone.http,
                            serenity::all::CreateMessage::new()
                                .content(format!("<@&{}> {} ticket still needs attention!", role_id, priority_label))
                        ).await;
                    }
                }
            }
        });
    } else {
        // Remove auto-ping for normal priority
        let redis_key = format!("priority_ping:{}", ticket.id);
        let mut redis_conn = db.redis.clone();
        let _: () = redis::cmd("DEL")
            .arg(&redis_key)
            .query_async(&mut redis_conn)
            .await
            .unwrap_or(());
    }

    // Delete the admin's command message
    let _ = msg.delete(&ctx.http).await;

    // Send log
    let guild_obj = crate::database::ticket::get_or_create_guild(&db.pool, ticket.guild_id).await?;
    let log_embed = crate::utils::create_embed(
        "Priority Set",
        format!("Ticket #{}\nPriority: **{}**\nSet by: <@{}>",
            ticket.ticket_number, priority.to_uppercase(), msg.author.id.get())
    );
    let _ = crate::utils::send_log(ctx, guild_obj.log_channel_id, log_embed).await;

    // Send priority update as bot message
    let priority_emoji = match priority {
        "low" => "🟢",
        "normal" => "🔵",
        "high" => "🟡",
        "urgent" => "🔴",
        _ => "🔵",
    };

    msg.channel_id.send_message(&ctx.http,
        serenity::all::CreateMessage::new()
            .content(format!("**Priority:** {} {}", priority_emoji, priority.to_uppercase()))
    ).await?;

    Ok(())
}

pub async fn blacklist(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
    if !has_admin_permissions(ctx, msg).await? {
        let embed = create_error_embed("Permission Denied", "You need Administrator permission to use this command");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!blacklist <add|remove|list> [user]`");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;

    match args[0] {
        "add" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing User", "Usage: `!blacklist add <user>`");
                msg.channel_id.send_message(&ctx.http,
                    serenity::all::CreateMessage::new().embed(embed)
                ).await?;
                return Ok(());
            }

            let user_id = parse_user_id(args[1])?;
            let reason = if args.len() > 2 { Some(args[2..].join(" ")) } else { None };

            sqlx::query(
                "INSERT INTO blacklist (target_id, target_type, reason, blacklisted_by) VALUES ($1, $2, $3, $4)
                 ON CONFLICT (target_id) DO UPDATE SET reason = $3"
            )
            .bind(user_id)
            .bind("user")
            .bind(reason)
            .bind(msg.author.id.get() as i64)
            .execute(&db.pool)
            .await?;

            let embed = create_success_embed("User Blacklisted", format!("<@{}> has been blacklisted from creating tickets", user_id));
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        "remove" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing User", "Usage: `!blacklist remove <user>`");
                msg.channel_id.send_message(&ctx.http,
                    serenity::all::CreateMessage::new().embed(embed)
                ).await?;
                return Ok(());
            }

            let user_id = parse_user_id(args[1])?;

            sqlx::query("DELETE FROM blacklist WHERE target_id = $1 AND target_type = 'user'")
                .bind(user_id)
                .execute(&db.pool)
                .await?;

            let embed = create_success_embed("User Unblacklisted", format!("<@{}> has been removed from the blacklist", user_id));
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        "list" => {
            let blacklist: Vec<(i64, Option<String>)> = sqlx::query_as(
                "SELECT target_id, reason FROM blacklist WHERE target_type = 'user'"
            )
            .fetch_all(&db.pool)
            .await?;

            let description = if blacklist.is_empty() {
                "No users are blacklisted".to_string()
            } else {
                blacklist.iter()
                    .map(|(user_id, reason)| {
                        let reason_text = reason.as_ref()
                            .map(|r| format!(": {}", r))
                            .unwrap_or_default();
                        format!("<@{}>{}", user_id, reason_text)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            let embed = create_embed("Blacklisted Users", description);
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        _ => {
            let embed = create_error_embed("Invalid Subcommand", "Valid subcommands: `add`, `remove`, `list`");
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
    }

    Ok(())
}

pub async fn note(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
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
        let embed = create_error_embed("Permission Denied", "Only support staff can manage notes");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!note <add|list> [message]`");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    match args[0] {
        "add" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Message", "Usage: `!note add <message>`");
                msg.channel_id.send_message(&ctx.http,
                    serenity::all::CreateMessage::new().embed(embed)
                ).await?;
                return Ok(());
            }

            let note_text = args[1..].join(" ");

            sqlx::query(
                "INSERT INTO ticket_notes (ticket_id, author_id, note) VALUES ($1, $2, $3)"
            )
            .bind(ticket.id)
            .bind(msg.author.id.get() as i64)
            .bind(&note_text)
            .execute(&db.pool)
            .await?;

            // Delete the admin's command message
            let _ = msg.delete(&ctx.http).await;

            // Send log
            let guild = db_ticket::get_or_create_guild(&db.pool, ticket.guild_id).await?;
            let log_embed = crate::utils::create_embed(
                "Note Added",
                format!("Ticket #{}\nNote added by: <@{}>\nNote: {}", ticket.ticket_number, msg.author.id.get(), note_text)
            );
            let _ = crate::utils::send_log(ctx, guild.log_channel_id, log_embed).await;

            // Send note confirmation as bot message
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new()
                    .content(format!("**Note:** {}", note_text))
            ).await?;
        }
        "list" => {
            let notes: Vec<(i64, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
                "SELECT author_id, note, created_at FROM ticket_notes WHERE ticket_id = $1 ORDER BY created_at ASC"
            )
            .bind(ticket.id)
            .fetch_all(&db.pool)
            .await?;

            let description = if notes.is_empty() {
                "No notes for this ticket".to_string()
            } else {
                notes.iter()
                    .map(|(author_id, note, created_at)| {
                        format!("<@{}> - <t:{}:R>\n{}", author_id, created_at.timestamp(), note)
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n")
            };

            let embed = create_embed("Ticket Notes", description);
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        _ => {
            let embed = create_error_embed("Invalid Subcommand", "Valid subcommands: `add`, `list`");
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
    }

    Ok(())
}

pub async fn stats(ctx: &Context, msg: &Message, db: &Arc<Database>) -> Result<()> {
    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;

    let total_tickets: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tickets WHERE guild_id = $1")
        .bind(guild_id)
        .fetch_one(&db.pool)
        .await?;

    let open_tickets: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tickets WHERE guild_id = $1 AND status = 'open'")
        .bind(guild_id)
        .fetch_one(&db.pool)
        .await?;

    let closed_tickets: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tickets WHERE guild_id = $1 AND status = 'closed'")
        .bind(guild_id)
        .fetch_one(&db.pool)
        .await?;

    let claimed_tickets: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tickets WHERE guild_id = $1 AND claimed_by IS NOT NULL")
        .bind(guild_id)
        .fetch_one(&db.pool)
        .await?;

    let embed = CreateEmbed::new()
        .title("Ticket Statistics")
        .color(0x5865F2)
        .field("Total Tickets", total_tickets.0.to_string(), true)
        .field("Open Tickets", open_tickets.0.to_string(), true)
        .field("Closed Tickets", closed_tickets.0.to_string(), true)
        .field("Claimed Tickets", claimed_tickets.0.to_string(), true);

    msg.channel_id.send_message(&ctx.http,
        serenity::all::CreateMessage::new().embed(embed)
    ).await?;

    Ok(())
}

async fn has_admin_permissions(ctx: &Context, msg: &Message) -> Result<bool> {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(false),
    };

    let member = guild_id.member(&ctx.http, msg.author.id).await?;
    let permissions = {
        let guild_obj = ctx.cache.guild(guild_id).ok_or_else(|| anyhow::anyhow!("Guild not found"))?;
        guild_obj.member_permissions(&member)
    };

    Ok(permissions.contains(Permissions::ADMINISTRATOR))
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

fn parse_role_id(input: &str) -> Result<i64> {
    let cleaned = input.trim_start_matches("<@&").trim_end_matches('>');
    cleaned.parse::<i64>()
        .map_err(|_| anyhow::anyhow!("Invalid role mention"))
}

fn parse_user_id(input: &str) -> Result<i64> {
    let cleaned = input.trim_start_matches("<@").trim_end_matches('>').trim_start_matches('!');
    cleaned.parse::<i64>()
        .map_err(|_| anyhow::anyhow!("Invalid user mention"))
}
