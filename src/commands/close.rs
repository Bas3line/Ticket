use serenity::all::{CommandInteraction, Context, CreateCommand};
use crate::database::Database;
use crate::utils::{create_error_embed, create_success_embed};
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

        let embed = create_success_embed("Ticket Closed", "This channel will be deleted in 5 seconds");

        interaction
            .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(embed)
            ))
            .await?;

        crate::utils::close_ticket_unified(ctx, ticket, interaction.user.id.get(), db).await?;
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
