use serenity::all::{CommandInteraction, Context, CreateCommand};
use crate::database::Database;
use crate::utils::create_error_embed;
use anyhow::Result;

pub async fn run(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let channel_id = interaction.channel_id.get() as i64;

    let ticket = crate::database::ticket::get_ticket_by_channel(&db.pool, channel_id).await?;

    if let Some(ticket) = ticket {
        if ticket.owner_id != interaction.user.id.get() as i64 {
            let embed = create_error_embed(
                "Permission Denied",
                "Only the ticket owner can close this ticket",
            );

            interaction
                .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                    serenity::all::CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true)
                ))
                .await?;
            return Ok(());
        }

        let messages = crate::database::ticket::get_ticket_messages(&db.pool, ticket.id).await?;

        let owner = ctx.http.get_user(serenity::all::UserId::new(ticket.owner_id as u64)).await?;
        let claimed_by_name = if let Some(claimer_id) = ticket.claimed_by {
            let claimer = ctx.http.get_user(serenity::all::UserId::new(claimer_id as u64)).await?;
            Some(claimer.name)
        } else {
            None
        };

        let html = crate::utils::transcript::generate_transcript(
            ticket.ticket_number,
            owner.name,
            ticket.created_at,
            ticket.closed_at,
            claimed_by_name,
            messages,
        )
        .await?;

        let filepath = crate::utils::transcript::save_transcript(ticket.guild_id, ticket.ticket_number, html).await?;

        if let Ok(guild) = sqlx::query_as::<_, crate::models::Guild>(
            "SELECT * FROM guilds WHERE guild_id = $1"
        )
        .bind(ticket.guild_id)
        .fetch_one(&db.pool)
        .await
        {
            // Send to transcript channel if configured
            if let Some(transcript_channel_id) = guild.transcript_channel_id {
                let channel = serenity::all::ChannelId::new(transcript_channel_id as u64);

                let file = serenity::all::CreateAttachment::path(&filepath).await?;

                let embed = crate::utils::create_embed(
                    format!("Ticket - {} Closed", ticket.owner_id),
                    format!(
                        "Owner: <@{}>\nClosed at: <t:{}:F>",
                        ticket.owner_id,
                        ticket.closed_at.unwrap().timestamp()
                    ),
                );

                channel
                    .send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed).add_file(file))
                    .await?;
            }

            // Send to ticket owner's DM
            let owner_user = serenity::all::UserId::new(ticket.owner_id as u64).to_user(&ctx.http).await;
            if let Ok(user) = owner_user {
                if let Ok(dm) = user.create_dm_channel(&ctx.http).await {
                    let dm_embed = crate::utils::create_embed(
                        "Ticket Closed - Transcript",
                        format!("Your ticket #{} has been closed. Here's the transcript.", ticket.ticket_number)
                    ).color(0x5865F2);
                    let dm_file = serenity::all::CreateAttachment::path(&filepath).await?;
                    let _ = dm.send_message(&ctx.http,
                        serenity::all::CreateMessage::new()
                            .embed(dm_embed)
                            .add_file(dm_file)
                    ).await;
                }
            }

            let _ = crate::utils::transcript::delete_transcript(&filepath).await;
        }

        crate::database::ticket::delete_ticket_messages(&db.pool, ticket.id).await?;

        let mut redis_conn = db.redis.clone();
        let _ = crate::database::ticket::cleanup_priority_ping(&mut redis_conn, ticket.id).await;

        // Send log
        if let Ok(guild) = crate::database::ticket::get_or_create_guild(&db.pool, ticket.guild_id).await {
            let log_embed = crate::utils::create_embed(
                "Ticket Closed",
                format!("Ticket: ticket-{}\nOwner: <@{}>\nClosed by: <@{}>\nClosed at: <t:{}:F>",
                    ticket.owner_id, ticket.owner_id, interaction.user.id, chrono::Utc::now().timestamp())
            );
            let _ = crate::utils::send_log(ctx, guild.log_channel_id, log_embed).await;
        }

        crate::database::ticket::close_ticket(&db.pool, ticket.id).await?;

        interaction
            .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .content("Ticket closed. This channel will be deleted in 5 seconds.")
            ))
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        interaction.channel_id.delete(&ctx.http).await?;
    } else {
        let embed = create_error_embed("Error", "This is not a ticket channel");

        interaction
            .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true)
            ))
            .await?;
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("close").description("Close the current ticket")
}
