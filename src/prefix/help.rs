use anyhow::Result;
use serenity::all::{
    Context, CreateEmbed, CreateMessage, Message, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, CreateActionRow,
};
use std::sync::Arc;
use crate::database::Database;

pub async fn execute(ctx: &Context, msg: &Message, db: &Arc<Database>) -> Result<()> {
    let guild_id = msg.guild_id.map(|g| g.get() as i64).unwrap_or(0);
    let prefix = crate::database::ticket::get_guild_prefix(&db.pool, guild_id).await?;
    let is_premium = crate::database::ticket::is_premium(&db.pool, guild_id).await?;

    let guild_name = msg.guild_id
        .and_then(|g| ctx.cache.guild(g))
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "Unknown Server".to_string());

    let bot_name = ctx.cache.current_user().name.clone();
    let server_count = ctx.cache.guild_count();

    let embed = create_main_help_embed(&prefix, &bot_name, server_count, &guild_name, is_premium);

    let select_menu = CreateSelectMenu::new(
        "help_menu",
        CreateSelectMenuKind::String {
            options: vec![
                CreateSelectMenuOption::new("Ticket Commands", "help_tickets")
                    .description("Commands for managing tickets"),
                CreateSelectMenuOption::new("Setup & Configuration", "help_setup")
                    .description("Setup channels and bot settings"),
                CreateSelectMenuOption::new("Admin Commands", "help_admin")
                    .description("Panel, categories, roles, and stats"),
                CreateSelectMenuOption::new("Premium Features", "help_premium")
                    .description("Premium status and benefits"),
                CreateSelectMenuOption::new("Owner Commands", "help_owner")
                    .description("Owner-only management commands"),
            ],
        }
    )
    .placeholder("Select a category to view commands");

    let components = vec![CreateActionRow::SelectMenu(select_menu)];

    msg.channel_id.send_message(
        &ctx.http,
        CreateMessage::new().embed(embed).components(components)
    ).await?;

    Ok(())
}

fn create_main_help_embed(prefix: &str, bot_name: &str, server_count: usize, guild_name: &str, is_premium: bool) -> CreateEmbed {
    let premium_badge = if is_premium { " [PREMIUM]" } else { "" };

    CreateEmbed::new()
        .title(format!("Ticket Bot - Help Menu"))
        .description(format!(
            "**Bot:** {}\n\
            **Servers:** {}\n\
            **Current Server:** {}{}\n\
            **Prefix:** `{}`\n\n\
            Use the dropdown menu below to view commands by category.\n\
            For quick help, try `{}ping` or mention the bot.",
            bot_name,
            server_count,
            guild_name,
            premium_badge,
            prefix,
            prefix
        ))
        .color(0x5865F2)
        .field(
            "Quick Start",
            format!(
                "`{}setup` - Interactive setup wizard\n\
                `{}panel` - Create ticket panel\n\
                `{}doc <command>` - Get detailed docs for any command\n\
                `{}help` - Show this menu",
                prefix, prefix, prefix, prefix
            ),
            false,
        )
        .footer(serenity::all::CreateEmbedFooter::new("Select a category from the dropdown menu"))
        .timestamp(serenity::all::Timestamp::now())
}
