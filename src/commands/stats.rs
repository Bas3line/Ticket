use serenity::all::{CommandInteraction, Context, CreateCommand, CreateEmbed};
use crate::database::Database;
use anyhow::Result;

pub async fn run(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;

    let open_tickets: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tickets WHERE guild_id = $1 AND status = 'open'"
    )
    .bind(guild_id)
    .fetch_one(&db.pool)
    .await?;

    let closed_tickets: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tickets WHERE guild_id = $1 AND status = 'closed'"
    )
    .bind(guild_id)
    .fetch_one(&db.pool)
    .await?;

    let claimed_tickets: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tickets WHERE guild_id = $1 AND status = 'open' AND claimed_by IS NOT NULL"
    )
    .bind(guild_id)
    .fetch_one(&db.pool)
    .await?;

    let unclaimed_tickets: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tickets WHERE guild_id = $1 AND status = 'open' AND claimed_by IS NULL"
    )
    .bind(guild_id)
    .fetch_one(&db.pool)
    .await?;

    let avg_response_time: Option<(f64,)> = sqlx::query_as(
        "SELECT AVG(EXTRACT(EPOCH FROM (closed_at - created_at)))
         FROM tickets WHERE guild_id = $1 AND status = 'closed' AND closed_at IS NOT NULL"
    )
    .bind(guild_id)
    .fetch_optional(&db.pool)
    .await?;

    let total_messages: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM ticket_messages tm
         JOIN tickets t ON tm.ticket_id = t.id
         WHERE t.guild_id = $1"
    )
    .bind(guild_id)
    .fetch_one(&db.pool)
    .await?;

    let top_support: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT claimed_by, COUNT(*) as count
         FROM tickets
         WHERE guild_id = $1 AND claimed_by IS NOT NULL AND status = 'closed'
         GROUP BY claimed_by
         ORDER BY count DESC
         LIMIT 5"
    )
    .bind(guild_id)
    .fetch_all(&db.pool)
    .await?;

    let avg_time_str = if let Some((avg_seconds,)) = avg_response_time {
        let hours = (avg_seconds / 3600.0).floor();
        let minutes = ((avg_seconds % 3600.0) / 60.0).floor();
        format!("{}h {}m", hours, minutes)
    } else {
        "N/A".to_string()
    };

    let top_support_str = if top_support.is_empty() {
        "No data yet".to_string()
    } else {
        top_support
            .iter()
            .map(|(user_id, count)| format!("<@{}> - {} tickets", user_id, count))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let embed = CreateEmbed::new()
        .title("Ticket Statistics")
        .color(serenity::all::Colour::from_rgb(88, 101, 242))
        .field("Open Tickets", open_tickets.0.to_string(), true)
        .field("Closed Tickets", closed_tickets.0.to_string(), true)
        .field("Total Tickets", (open_tickets.0 + closed_tickets.0).to_string(), true)
        .field("Claimed Tickets", claimed_tickets.0.to_string(), true)
        .field("Unclaimed Tickets", unclaimed_tickets.0.to_string(), true)
        .field("Total Messages", total_messages.0.to_string(), true)
        .field("Avg. Resolution Time", avg_time_str, false)
        .field("Top Support Staff", top_support_str, false);

    interaction
        .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
            serenity::all::CreateInteractionResponseMessage::new()
                .embed(embed)
        ))
        .await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("stats").description("View ticket statistics")
}
