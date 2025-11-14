use anyhow::Result;
use serenity::all::{Context, Message, Permissions};
use std::sync::Arc;
use crate::database::Database;
use crate::database::ignore as db_ignore;
use crate::utils::{create_success_embed, create_error_embed, create_embed};

async fn has_admin_permissions(ctx: &Context, msg: &Message) -> Result<bool> {
    if let Some(guild_id) = msg.guild_id {
        if let Some(member) = msg.member.as_ref() {
            if let Some(permissions) = member.permissions {
                if permissions.contains(Permissions::ADMINISTRATOR) {
                    return Ok(true);
                }
            }
        }

        if let Ok(guild) = ctx.http.get_guild(guild_id).await {
            if let Ok(member) = guild.member(&ctx.http, msg.author.id).await {
                if let Ok(guild_obj) = ctx.cache.guild(guild_id).ok_or_else(|| anyhow::anyhow!("Guild not in cache")) {
                    let permissions = guild_obj.member_permissions(&member);
                    return Ok(permissions.contains(Permissions::ADMINISTRATOR));
                }
            }
        }
    }
    Ok(false)
}

fn parse_channel_id(s: &str) -> Result<i64> {
    let cleaned = s.trim_start_matches("<#").trim_end_matches('>');
    Ok(cleaned.parse()?)
}

pub async fn ignore(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
    if !has_admin_permissions(ctx, msg).await? {
        let embed = create_error_embed("Permission Denied", "You need Administrator permission to use this command");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", "Usage: `!ignore <add|remove|list> [channel]`");
        msg.channel_id.send_message(&ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;

    match args[0] {
        "add" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Channel", "Usage: `!ignore add <channel>`");
                msg.channel_id.send_message(&ctx.http,
                    serenity::all::CreateMessage::new().embed(embed)
                ).await?;
                return Ok(());
            }

            let channel_id = parse_channel_id(args[1])?;
            db_ignore::add_ignored_channel(&db.pool, guild_id, channel_id).await?;

            let embed = create_success_embed("Channel Ignored", format!("Bot will now ignore messages in <#{}>", channel_id));
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        "remove" => {
            if args.len() < 2 {
                let embed = create_error_embed("Missing Channel", "Usage: `!ignore remove <channel>`");
                msg.channel_id.send_message(&ctx.http,
                    serenity::all::CreateMessage::new().embed(embed)
                ).await?;
                return Ok(());
            }

            let channel_id = parse_channel_id(args[1])?;
            db_ignore::remove_ignored_channel(&db.pool, guild_id, channel_id).await?;

            let embed = create_success_embed("Channel Unignored", format!("Bot will now respond to messages in <#{}>", channel_id));
            msg.channel_id.send_message(&ctx.http,
                serenity::all::CreateMessage::new().embed(embed)
            ).await?;
        }
        "list" => {
            let channels = db_ignore::get_ignored_channels(&db.pool, guild_id).await?;

            let description = if channels.is_empty() {
                "No ignored channels configured".to_string()
            } else {
                channels.iter()
                    .map(|c| format!("<#{}>", c))
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            let embed = create_embed("Ignored Channels", description);
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
