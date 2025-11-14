use serenity::all::{Context, Message};
use crate::database::Database;
use crate::utils::{create_success_embed, create_embed};
use anyhow::Result;
use std::sync::Arc;

pub async fn channel_name(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
    let guild_id = msg.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;
    let prefix = crate::database::ticket::get_guild_prefix(&db.pool, guild_id).await?;

    if args.is_empty() {
        let guild = crate::database::ticket::get_or_create_guild(&db.pool, guild_id).await?;
        let template = guild.channel_name_template.unwrap_or_else(|| "ticket-$ticket_number".to_string());

        let embed = create_embed(
            "Channel Name Template",
            format!(
                "**Current Template:** `{}`\n\n\
                **Available Variables:**\n\
                `$ticket_number` - Ticket number (e.g., 123)\n\
                `$user_id` - User's Discord ID (e.g., 123456789)\n\
                `$user_name` - User's username (e.g., john)\n\n\
                **Examples:**\n\
                `ticket-$ticket_number` → ticket-123\n\
                `ticket-$user_name-$ticket_number` → ticket-john-123\n\
                `support-$user_id` → support-123456789\n\
                `$user_name-ticket` → john-ticket\n\n\
                **Current Output Example:**\n\
                `{}`\n\n\
                **Usage:**\n\
                `{}channel-name <template>` - Set custom template\n\
                `{}channel-name` - View current template",
                template,
                crate::database::ticket::format_channel_name(&template, 123, 123456789, "username"),
                prefix,
                prefix
            )
        ).color(0x5865F2);

        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    } else {
        let template = args.join(" ");

        crate::database::ticket::update_channel_name_template(&db.pool, guild_id, &template).await?;

        let embed = create_success_embed(
            "Channel Name Template Updated",
            format!("Channel name template set to: `{}`\n\nExample: `{}`",
                template,
                crate::database::ticket::format_channel_name(&template, 123, 123456789, "username")
            )
        );

        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    }

    Ok(())
}
