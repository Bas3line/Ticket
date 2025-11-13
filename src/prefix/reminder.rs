use anyhow::Result;
use serenity::all::{Context, CreateEmbed, CreateMessage, Message};
use std::sync::Arc;
use crate::database::Database;
use chrono::{Duration, Utc};

pub async fn reminder(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
    if args.len() < 2 {
        let embed = crate::utils::create_error_embed(
            "Invalid Usage",
            "Usage: `!reminder <time> <reason>`\n\n\
            Examples:\n\
            `!reminder 30m Check ticket status`\n\
            `!reminder 2h Close ticket`\n\
            `!reminder 1d Follow up with user`\n\n\
            Time formats: s (seconds), m (minutes), h (hours), d (days)"
        );
        msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let time_str = args[0];
    let reason = args[1..].join(" ");

    let duration = match parse_duration(time_str) {
        Ok(d) => d,
        Err(e) => {
            let embed = crate::utils::create_error_embed("Invalid Time Format", &format!("{}", e));
            msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(embed)).await?;
            return Ok(());
        }
    };

    let remind_at = Utc::now() + duration;

    let reminder = crate::database::ticket::create_reminder(
        &db.pool,
        msg.author.id.get() as i64,
        msg.channel_id.get() as i64,
        msg.guild_id.map(|g| g.get() as i64),
        Some(msg.id.get() as i64),
        reason.clone(),
        remind_at,
    )
    .await?;

    let embed = CreateEmbed::new()
        .title("Reminder Set")
        .description(format!(
            "**Reason:** {}\n\
            **Remind at:** <t:{}:F> (<t:{}:R>)",
            reason,
            remind_at.timestamp(),
            remind_at.timestamp()
        ))
        .color(0x5865F2)
        .footer(serenity::all::CreateEmbedFooter::new(format!("Reminder ID: {}", reminder.id)));

    msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(embed)).await?;

    Ok(())
}

fn parse_duration(time_str: &str) -> Result<Duration> {
    let time_str = time_str.trim();

    if time_str.is_empty() {
        return Err(anyhow::anyhow!("Time string is empty"));
    }

    let unit = time_str.chars().last().ok_or_else(|| anyhow::anyhow!("Invalid time format"))?;
    let value_str = &time_str[..time_str.len() - 1];
    let value: i64 = value_str.parse().map_err(|_| anyhow::anyhow!("Invalid number: {}", value_str))?;

    if value <= 0 {
        return Err(anyhow::anyhow!("Time value must be positive"));
    }

    match unit {
        's' => Ok(Duration::seconds(value)),
        'm' => Ok(Duration::minutes(value)),
        'h' => Ok(Duration::hours(value)),
        'd' => Ok(Duration::days(value)),
        'w' => Ok(Duration::weeks(value)),
        _ => Err(anyhow::anyhow!("Invalid time unit '{}'. Use s (seconds), m (minutes), h (hours), d (days), or w (weeks)", unit)),
    }
}
