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
        let support_roles = crate::database::ticket::get_support_roles(&db.pool, ticket.guild_id).await?;

        if support_roles.is_empty() {
            let embed = create_error_embed(
                "No Support Roles",
                "No support roles configured for this server.",
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
                                "Urgent: Ticket Requires Immediate Attention",
                                format!(
                                    "**Ticket:** #{}\n\
                                     **User:** <@{}>\n\
                                     **Channel:** <#{}>\n\n\
                                     Please claim this ticket using `/claim` or `!claim` command.",
                                    ticket.ticket_number,
                                    ticket.owner_id,
                                    ticket.channel_id
                                )
                            ).color(0xFEE75C))
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

        interaction.channel_id
            .send_message(
                &ctx.http,
                serenity::all::CreateMessage::new()
                    .content(mention_content)
                    .embed(crate::utils::create_embed(
                        "Support Team Notified",
                        format!(
                            "All support staff have been notified about this ticket.\n\
                             **Ticket:** #{}\n\
                             **User:** <@{}>\n\n\
                             Use `/claim` or `!claim` to claim this ticket.",
                            ticket.ticket_number,
                            ticket.owner_id
                        )
                    ).color(0xFEE75C))
            )
            .await?;

        let embed = create_success_embed(
            "Support Team Notified",
            "All support members have been DM'd about this ticket.",
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
    CreateCommand::new("handle")
        .description("Immediately notify all support staff about this ticket")
}
