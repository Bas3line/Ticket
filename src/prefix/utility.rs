use anyhow::Result;
use serenity::all::{Context, Message};
use std::sync::Arc;
use crate::database::Database;
use crate::utils::create_embed;

pub async fn ping(ctx: &Context, msg: &Message, _db: &Arc<Database>) -> Result<()> {
    let start = std::time::Instant::now();

    let mut response = msg.channel_id.send_message(
        &ctx.http,
        serenity::all::CreateMessage::new().content("Pinging...")
    ).await?;

    let api_latency = start.elapsed().as_millis();

    let embed = create_embed(
        "Pong!",
        format!("**Latency:** `{}ms`", api_latency)
    )
    .color(0x57F287);

    response.edit(
        &ctx.http,
        serenity::all::EditMessage::new()
            .content("")
            .embed(embed)
    ).await?;

    Ok(())
}

pub async fn bot_mention(ctx: &Context, msg: &Message, db: &Arc<Database>) -> Result<()> {
    let guild_id = msg.guild_id.map(|g| g.get() as i64).unwrap_or(0);
    let prefix = crate::database::ticket::get_guild_prefix(&db.pool, guild_id).await?;

    let embed = create_embed(
        "Hello! I'm the Ticket Bot",
        format!(
            "A powerful Discord ticket management system\n\
            Supports premium features, customization, and more!\n\n\
            **Current Prefix:** `{}`\n\n\
            **Quick Start:**\n\
            • `{}setup` - Run the interactive setup wizard\n\
            • `{}help` - View all available commands\n\
            • `{}panel` - Create a ticket panel (after setup)\n\
            • `{}ping` - Check bot latency\n\n\
            **Need Help?**\n\
            Type `{}help` to see the complete command list!",
            prefix,
            prefix,
            prefix,
            prefix,
            prefix,
            prefix
        )
    )
    .color(0x5865F2)
    .footer(serenity::all::CreateEmbedFooter::new("Ticket Bot - Ready to serve your server"))
    .timestamp(serenity::all::Timestamp::now());

    msg.channel_id.send_message(
        &ctx.http,
        serenity::all::CreateMessage::new().embed(embed)
    ).await?;

    Ok(())
}
