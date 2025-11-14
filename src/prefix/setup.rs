use anyhow::Result;
use serenity::all::{Context, Message, Permissions, CreateEmbed};
use std::sync::Arc;
use crate::database::Database;
use crate::database::ticket;
use crate::utils::{create_success_embed, create_error_embed};

pub async fn execute(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
    if !has_permissions(ctx, msg).await? {
        let embed = create_error_embed("Permission Denied", "You need Administrator permission to use this command");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    if args.is_empty() {
        return show_interactive_setup(ctx, msg, db).await;
    }

    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;

    match args[0] {
        "category" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Channel", "Usage: `!setup category <channel>`");
                msg.channel_id.send_message(&ctx.http,
                    serenity::all::CreateMessage::new().embed(embed)
                ).await?;
                return Ok(());
            }

            let channel_id = parse_channel_id(args[1])?;
            ticket::update_guild_category(&db.pool, guild_id, channel_id).await?;

            let embed = create_success_embed("Category Set", format!("Ticket category channel set to <#{}>", channel_id));
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        "logs" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Channel", "Usage: `!setup logs <channel>`");
                msg.channel_id.send_message(&ctx.http,
                    serenity::all::CreateMessage::new().embed(embed)
                ).await?;
                return Ok(());
            }

            let channel_id = parse_channel_id(args[1])?;
            ticket::update_guild_log_channel(&db.pool, guild_id, channel_id).await?;

            let embed = create_success_embed("Log Channel Set", format!("Log channel set to <#{}>", channel_id));
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        "transcripts" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Channel", "Usage: `!setup transcripts <channel>`");
                msg.channel_id.send_message(&ctx.http,
                    serenity::all::CreateMessage::new().embed(embed)
                ).await?;
                return Ok(());
            }

            let channel_id = parse_channel_id(args[1])?;
            ticket::update_guild_transcript_channel(&db.pool, guild_id, channel_id).await?;

            let embed = create_success_embed("Transcript Channel Set", format!("Transcript channel set to <#{}>", channel_id));
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        _ => {
            let embed = create_error_embed("Invalid Subcommand", "Valid subcommands: `category`, `logs`, `transcripts`");
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
    }

    Ok(())
}

pub async fn set_prefix(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
    if !has_permissions(ctx, msg).await? {
        let embed = create_error_embed("Permission Denied", "You need Administrator permission to use this command");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Missing Prefix", "Usage: `!prefix <new_prefix>`");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;
    let new_prefix = args[0];

    if new_prefix.len() > 10 {
        let embed = create_error_embed("Prefix Too Long", "Prefix must be 10 characters or less");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    ticket::get_or_create_guild(&db.pool, guild_id).await?;
    ticket::set_guild_prefix(&db.pool, guild_id, new_prefix.to_string()).await?;

    let embed = create_success_embed("Prefix Updated", format!("Bot prefix has been set to `{}`", new_prefix));
    msg.channel_id.send_message(&ctx.http,
        serenity::all::CreateMessage::new().embed(embed)
    ).await?;

    Ok(())
}

async fn has_permissions(ctx: &Context, msg: &Message) -> Result<bool> {
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

fn parse_channel_id(input: &str) -> Result<i64> {
    let cleaned = input.trim_start_matches("<#").trim_end_matches('>');
    cleaned.parse::<i64>()
        .map_err(|_| anyhow::anyhow!("Invalid channel mention"))
}

async fn show_interactive_setup(ctx: &Context, msg: &Message, db: &Arc<Database>) -> Result<()> {
    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;
    let guild = ticket::get_or_create_guild(&db.pool, guild_id).await?;
    let is_premium = ticket::is_premium(&db.pool, guild_id).await?;

    let category_status = if guild.ticket_category_id.is_some() {
        format!("Set: <#{}>", guild.ticket_category_id.unwrap())
    } else {
        "Not Set".to_string()
    };

    let logs_status = if guild.log_channel_id.is_some() {
        format!("Set: <#{}>", guild.log_channel_id.unwrap())
    } else {
        "Not Set".to_string()
    };

    let transcripts_status = if guild.transcript_channel_id.is_some() {
        format!("Set: <#{}>", guild.transcript_channel_id.unwrap())
    } else {
        "Not Set".to_string()
    };

    let claim_buttons = if guild.claim_buttons_enabled.unwrap_or(true) { "Enabled" } else { "Disabled" };
    let auto_close = guild.auto_close_hours.unwrap_or(0);

    let embed = CreateEmbed::new()
        .title("Ticket Bot Setup")
        .description(format!(
            "Configure your ticket system using the dropdown menu below.\n\n\
            **Premium Status:** {}\n\n\
            **Current Configuration:**\n\
            • Ticket Category: {}\n\
            • Log Channel: {}\n\
            • Transcript Channel: {}\n\
            • Claim Buttons: {}\n\
            • Auto Close: {}",
            if is_premium { "Active" } else { "Inactive" },
            category_status,
            logs_status,
            transcripts_status,
            claim_buttons,
            if auto_close == 0 { "Disabled".to_string() } else { format!("{} hours", auto_close) }
        ))
        .color(0x5865F2)
        .footer(serenity::all::CreateEmbedFooter::new("Select an option to configure"))
        .timestamp(serenity::all::Timestamp::now());

    let select_menu = serenity::all::CreateSelectMenu::new(
        "setup_menu",
        serenity::all::CreateSelectMenuKind::String {
            options: vec![
                serenity::all::CreateSelectMenuOption::new("Set Ticket Category", "setup_category")
                    .description("Configure ticket category channel"),
                serenity::all::CreateSelectMenuOption::new("Set Log Channel", "setup_logs")
                    .description("Configure logging channel"),
                serenity::all::CreateSelectMenuOption::new("Set Transcript Channel", "setup_transcripts")
                    .description("Configure transcript channel"),
                serenity::all::CreateSelectMenuOption::new("Manage Categories", "setup_manage_categories")
                    .description("Add/edit ticket categories"),
                serenity::all::CreateSelectMenuOption::new("Set Support Role", "setup_support_role")
                    .description("Configure support staff role"),
                serenity::all::CreateSelectMenuOption::new("Set Ping Role", "setup_ping_role")
                    .description("Configure role to ping on ticket creation"),
                serenity::all::CreateSelectMenuOption::new("Toggle Claim Buttons", "setup_claim")
                    .description("Enable/disable claim buttons"),
                serenity::all::CreateSelectMenuOption::new("Configure Auto-Close", "setup_autoclose")
                    .description("Configure automatic ticket closing"),
                serenity::all::CreateSelectMenuOption::new("Ticket Limit", "setup_ticket_limit")
                    .description("Set max tickets per user or allow multiple"),
                serenity::all::CreateSelectMenuOption::new("View All Settings", "setup_settings")
                    .description("View and configure advanced settings"),
            ],
        }
    )
    .placeholder("Select a configuration option");

    let send_panel_button = serenity::all::CreateButton::new("setup_send_panel")
        .label("Send Panel")
        .style(serenity::all::ButtonStyle::Success);

    let delete_panel_button = serenity::all::CreateButton::new("setup_delete_panel")
        .label("Delete Panel")
        .style(serenity::all::ButtonStyle::Danger);

    let components = vec![
        serenity::all::CreateActionRow::SelectMenu(select_menu),
        serenity::all::CreateActionRow::Buttons(vec![send_panel_button, delete_panel_button]),
    ];

    msg.channel_id.send_message(
        &ctx.http,
        serenity::all::CreateMessage::new().embed(embed).components(components)
    ).await?;

    Ok(())
}
