use serenity::all::{
    CommandInteraction, Context, CreateCommand,
};
use crate::database::Database;
use crate::utils::{create_error_embed, create_success_embed};
use anyhow::Result;

pub async fn run(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let channel_id = interaction.channel_id.get() as i64;

    let ticket = crate::database::ticket::get_ticket_by_channel(&db.pool, channel_id).await?;

    if let Some(ticket) = ticket {
        if crate::database::ticket::ticket_has_messages(&db.pool, ticket.id).await? {
            let embed = create_error_embed(
                "Cannot Escalate",
                "This ticket has already received messages. Escalation is only for tickets without responses.",
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

        crate::database::ticket::create_escalation(
            &db.pool,
            ticket.id,
            interaction.user.id.get() as i64,
        ).await?;

        let support_roles = crate::database::ticket::get_support_roles(&db.pool, ticket.guild_id).await?;

        let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in guild"))?;

        for role in &support_roles {
            let members = guild_id.members(&ctx.http, None, None).await?;
            let role_id = serenity::all::RoleId::new(role.role_id as u64);

            for member in members {
                if member.roles.contains(&role_id) && !member.user.bot {
                    let dm = member.user.create_dm_channel(&ctx.http).await?;
                    let _ = dm.send_message(
                        &ctx.http,
                        serenity::all::CreateMessage::new()
                            .embed(crate::utils::create_embed(
                                "Ticket Escalated",
                                format!(
                                    "A ticket has been escalated and requires attention!\n\n\
                                     **Ticket:** #{}\n\
                                     **User:** <@{}>\n\
                                     **Channel:** <#{}>\n\n\
                                     You will receive hourly reminders until this ticket is claimed or closed.",
                                    ticket.ticket_number,
                                    ticket.owner_id,
                                    ticket.channel_id
                                )
                            ).color(0xED4245))
                    ).await;
                }
            }
        }

        let role_mentions: Vec<String> = support_roles
            .iter()
            .map(|r| format!("<@&{}>", r.role_id))
            .collect();

        let mention_content = if !role_mentions.is_empty() {
            role_mentions.join(" ")
        } else {
            String::new()
        };

        if !mention_content.is_empty() {
            interaction.channel_id
                .send_message(
                    &ctx.http,
                    serenity::all::CreateMessage::new()
                        .content(mention_content)
                        .embed(crate::utils::create_embed(
                            "Ticket Escalated",
                            "This ticket has been escalated and will be monitored until claimed or closed.",
                        ).color(0xED4245))
                )
                .await?;
        }

        let embed = create_success_embed(
            "Ticket Escalated",
            "Support team has been notified and will receive hourly reminders.",
        );

        interaction
            .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(embed)
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
    CreateCommand::new("escalate")
        .description("Escalate ticket to support team (only for unanswered tickets)")
}
