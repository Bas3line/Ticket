use anyhow::Result;
use serenity::all::{Context, Message, Permissions};
use std::sync::Arc;
use crate::database::Database;
use crate::utils::{create_success_embed, create_error_embed};

pub async fn settings(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
    if !has_admin_permissions(ctx, msg).await? {
        let embed = create_error_embed("Permission Denied", "You need Administrator permission to use this command");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;

    if args.is_empty() {
        return show_settings(ctx, msg, db, guild_id).await;
    }

    let setting = args[0].to_lowercase();

    match setting.as_str() {
        "claimbuttons" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Value", "Usage: `!settings claimbuttons <true|false>`");
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
                return Ok(());
            }

            let value = args[1].to_lowercase() == "true" || args[1] == "1" || args[1] == "on";

            crate::database::ticket::update_guild_settings(
                &db.pool,
                guild_id,
                Some(value),
                None,
                None,
                None,
                None,
            ).await?;

            let embed = create_success_embed(
                "Setting Updated",
                format!("Claim buttons have been **{}**", if value { "enabled" } else { "disabled" }),
            );

            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        }
        "autoclose" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Value", "Usage: `!settings autoclose <hours>` (0 to disable)");
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
                return Ok(());
            }

            let hours: i32 = args[1].parse()
                .map_err(|_| anyhow::anyhow!("Invalid hours value"))?;

            let value = if hours == 0 { None } else { Some(hours) };

            crate::database::ticket::update_guild_settings(
                &db.pool,
                guild_id,
                None,
                value,
                None,
                None,
                None,
            ).await?;

            let msg_text = if hours == 0 {
                "Auto-close has been **disabled**".to_string()
            } else {
                format!("Tickets will automatically close after **{} hours** of inactivity", hours)
            };

            let embed = create_success_embed("Setting Updated", msg_text);
            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        }
        "ticketlimit" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Value", "Usage: `!settings ticketlimit <number>` (0 for unlimited)");
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
                return Ok(());
            }

            let limit: i32 = args[1].parse()
                .map_err(|_| anyhow::anyhow!("Invalid limit value"))?;

            crate::database::ticket::update_guild_settings(
                &db.pool,
                guild_id,
                None,
                None,
                Some(limit),
                None,
                None,
            ).await?;

            let msg_text = if limit == 0 {
                "Ticket limit has been set to **unlimited**".to_string()
            } else {
                format!("Users can now have a maximum of **{} ticket(s)** open at once", limit)
            };

            let embed = create_success_embed("Setting Updated", msg_text);
            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        }
        "cooldown" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Value", "Usage: `!settings cooldown <seconds>` (0 to disable)");
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
                return Ok(());
            }

            let cooldown: i32 = args[1].parse()
                .map_err(|_| anyhow::anyhow!("Invalid cooldown value"))?;

            crate::database::ticket::update_guild_settings(
                &db.pool,
                guild_id,
                None,
                None,
                None,
                Some(cooldown),
                None,
            ).await?;

            let msg_text = if cooldown == 0 {
                "Ticket cooldown has been **disabled**".to_string()
            } else {
                format!("Users must wait **{} seconds** between creating tickets", cooldown)
            };

            let embed = create_success_embed("Setting Updated", msg_text);
            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        }
        "dmoncreate" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Value", "Usage: `!settings dmoncreate <true|false>`");
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
                return Ok(());
            }

            let value = args[1].to_lowercase() == "true" || args[1] == "1" || args[1] == "on";

            crate::database::ticket::update_guild_settings(
                &db.pool,
                guild_id,
                None,
                None,
                None,
                None,
                Some(value),
            ).await?;

            let embed = create_success_embed(
                "Setting Updated",
                format!("DM notifications on ticket creation have been **{}**", if value { "enabled" } else { "disabled" }),
            );

            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        }
        "embedcolor" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Value", "Usage: `!settings embedcolor <hex_color>` (e.g., #5865F2)");
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
                return Ok(());
            }

            let color_str = args[1].trim_start_matches('#');
            let color_int = i32::from_str_radix(color_str, 16)
                .map_err(|_| anyhow::anyhow!("Invalid hex color"))?;

            crate::database::ticket::update_embed_settings(
                &db.pool,
                guild_id,
                Some(color_int),
                None,
                None,
                None,
            ).await?;

            let embed = create_success_embed(
                "Embed Color Updated",
                format!("Embed color has been set to `#{}`", color_str.to_uppercase()),
            )
            .color(serenity::all::Colour::from(color_int as u32));

            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        }
        "embedtitle" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Value", "Usage: `!settings embedtitle <title>`");
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
                return Ok(());
            }

            let title = args[1..].join(" ");

            crate::database::ticket::update_embed_settings(
                &db.pool,
                guild_id,
                None,
                Some(title.clone()),
                None,
                None,
            ).await?;

            let embed = create_success_embed(
                "Embed Title Updated",
                format!("Embed title has been set to: **{}**", title),
            );

            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        }
        "embeddescription" | "embeddesc" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Value", "Usage: `!settings embeddescription <description>`");
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
                return Ok(());
            }

            let description = args[1..].join(" ");

            crate::database::ticket::update_embed_settings(
                &db.pool,
                guild_id,
                None,
                None,
                Some(description.clone()),
                None,
            ).await?;

            let embed = create_success_embed(
                "Embed Description Updated",
                format!("Embed description has been set to:\n{}", description),
            );

            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        }
        "embedfooter" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Value", "Usage: `!settings embedfooter <footer>` (use 'none' to remove)");
                msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
                return Ok(());
            }

            let footer = if args[1].to_lowercase() == "none" {
                None
            } else {
                Some(args[1..].join(" "))
            };

            crate::database::ticket::update_embed_settings(
                &db.pool,
                guild_id,
                None,
                None,
                None,
                footer.clone(),
            ).await?;

            let msg_text = if footer.is_none() {
                "Embed footer has been removed".to_string()
            } else {
                format!("Embed footer has been set to: {}", footer.unwrap())
            };

            let embed = create_success_embed("Embed Footer Updated", msg_text);
            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        }
        _ => {
            let embed = create_error_embed(
                "Unknown Setting",
                "Available settings:\n\
                • `claimbuttons` - Enable/disable claim buttons\n\
                • `autoclose` - Auto-close tickets after X hours\n\
                • `ticketlimit` - Max tickets per user\n\
                • `cooldown` - Cooldown between tickets (seconds)\n\
                • `dmoncreate` - DM users when ticket is created\n\
                • `embedcolor` - Panel embed color (hex)\n\
                • `embedtitle` - Panel embed title\n\
                • `embeddescription` - Panel embed description\n\
                • `embedfooter` - Panel embed footer",
            );

            msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        }
    }

    Ok(())
}

async fn show_settings(ctx: &Context, msg: &Message, db: &Arc<Database>, guild_id: i64) -> Result<()> {
    let guild = crate::database::ticket::get_or_create_guild(&db.pool, guild_id).await?;
    let is_premium = crate::database::ticket::is_premium(&db.pool, guild_id).await?;

    let claim_buttons = guild.claim_buttons_enabled.unwrap_or(true);
    let auto_close = guild.auto_close_hours.unwrap_or(0);
    let ticket_limit = guild.ticket_limit_per_user.unwrap_or(1);
    let cooldown = guild.ticket_cooldown_seconds.unwrap_or(0);
    let dm_on_create = guild.dm_on_create.unwrap_or(true);

    let embed_color = guild.embed_color.unwrap_or(5865714);
    let embed_title = guild.embed_title.unwrap_or_else(|| "Support Ticket".to_string());
    let embed_desc = guild.embed_description.unwrap_or_else(|| "Click the button below to create a ticket".to_string());
    let embed_footer = guild.embed_footer.clone().unwrap_or_else(|| "None".to_string());

    let settings_text = format!(
        "**Ticket Settings:**\n\
        • Claim Buttons: **{}**\n\
        • Auto Close: **{}**\n\
        • Ticket Limit: **{}** per user\n\
        • Cooldown: **{}** seconds\n\
        • DM on Create: **{}**\n\n\
        **Embed Customization:**\n\
        • Color: `#{:06X}`\n\
        • Title: {}\n\
        • Description: {}\n\
        • Footer: {}\n\n\
        {}",
        if claim_buttons { "Enabled" } else { "Disabled" },
        if auto_close == 0 { "Disabled".to_string() } else { format!("{} hours", auto_close) },
        if ticket_limit == 0 { "Unlimited".to_string() } else { ticket_limit.to_string() },
        cooldown,
        if dm_on_create { "Enabled" } else { "Disabled" },
        embed_color,
        embed_title,
        if embed_desc.len() > 100 { format!("{}...", &embed_desc[..100]) } else { embed_desc },
        embed_footer,
        if is_premium { "**Premium Server** - All features unlocked!" } else { "**Free Plan** - Upgrade to premium for more features!" }
    );

    let embed = crate::utils::create_embed("Server Settings", settings_text)
        .footer(serenity::all::CreateEmbedFooter::new("Use !settings <setting> <value> to change"));

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
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
