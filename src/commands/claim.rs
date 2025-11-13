use serenity::all::{
    CommandInteraction, Context, CreateCommand,
};
use crate::database::Database;
use crate::utils::{create_error_embed, create_success_embed};
use anyhow::Result;

pub async fn run(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let channel_id = interaction.channel_id.get() as i64;
    let claimer_id = interaction.user.id.get() as i64;

    let ticket = crate::database::ticket::get_ticket_by_channel(&db.pool, channel_id).await?;

    if let Some(ticket) = ticket {
        let support_roles = crate::database::ticket::get_support_roles(&db.pool, ticket.guild_id).await?;

        if support_roles.is_empty() {
            let embed = create_error_embed(
                "No Support Roles",
                "No support roles have been configured for this server",
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

        let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
        let member = guild_id.member(&ctx.http, interaction.user.id).await?;

        let has_support_role = support_roles.iter().any(|role| {
            member.roles.contains(&serenity::all::RoleId::new(role.role_id as u64))
        });

        if !has_support_role {
            let embed = create_error_embed(
                "Permission Denied",
                "Only users with a support role can claim tickets",
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

        if ticket.is_claimed() {
            let embed = create_error_embed(
                "Already Claimed",
                format!("This ticket is already claimed by <@{}>", ticket.claimed_by.unwrap()),
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

        crate::database::ticket::claim_ticket(&db.pool, ticket.id, claimer_id).await?;

        let _ = crate::database::ticket::deactivate_escalation(&db.pool, ticket.id).await;

        let guild = crate::database::ticket::get_or_create_guild(&db.pool, ticket.guild_id).await?;
        let log_embed = crate::utils::create_embed(
            "Ticket Claimed",
            format!("Ticket: ticket-{}\nClaimed by: <@{}>\nOwner: <@{}>", ticket.owner_id, claimer_id, ticket.owner_id)
        );
        let _ = crate::utils::send_log(ctx, guild.log_channel_id, log_embed).await;

        let embed = create_success_embed(
            "Ticket Claimed",
            format!("<@{}> has claimed this ticket", claimer_id),
        );

        interaction.channel_id
            .send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed))
            .await?;

        interaction
            .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .content("Ticket claimed successfully")
                    .ephemeral(true)
            ))
            .await?;
    } else {
        let embed = create_error_embed(
            "Error",
            "This command can only be used in ticket channels.",
        );

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
    CreateCommand::new("claim")
        .description("Claim a ticket")
}
