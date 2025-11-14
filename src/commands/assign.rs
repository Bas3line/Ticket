use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    ResolvedValue,
};
use crate::database::Database;
use crate::utils::{create_error_embed, create_success_embed};
use anyhow::Result;

pub async fn run(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let channel_id = interaction.channel_id.get() as i64;
    let assigner_id = interaction.user.id.get() as i64;

    // Get the ticket
    let ticket = crate::database::ticket::get_ticket_by_channel(&db.pool, channel_id).await?;

    if let Some(ticket) = ticket {
        // Get support roles
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

        // Check if user has support role
        let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
        let member = guild_id.member(&ctx.http, interaction.user.id).await?;

        let has_support_role = support_roles.iter().any(|role| {
            member.roles.contains(&serenity::all::RoleId::new(role.role_id as u64))
        });

        if !has_support_role {
            let embed = create_error_embed(
                "Permission Denied",
                "Only users with a support role can assign tickets",
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

        // Get the user to assign
        let options = &interaction.data.options();
        let user_option = options.iter().find(|opt| opt.name == "user");

        let assigned_user_id = if let Some(user_opt) = user_option {
            if let ResolvedValue::User(user, _) = &user_opt.value {
                user.id.get() as i64
            } else {
                let embed = create_error_embed("Invalid User", "Please provide a valid user");
                interaction
                    .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                        serenity::all::CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .ephemeral(true)
                    ))
                    .await?;
                return Ok(());
            }
        } else {
            let embed = create_error_embed("Missing User", "Please provide a user to assign");
            interaction
                .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                    serenity::all::CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true)
                ))
                .await?;
            return Ok(());
        };

        // Check if the assigned user has support role
        let assigned_member = guild_id.member(&ctx.http, serenity::all::UserId::new(assigned_user_id as u64)).await?;

        let assigned_has_support_role = support_roles.iter().any(|role| {
            assigned_member.roles.contains(&serenity::all::RoleId::new(role.role_id as u64))
        });

        if !assigned_has_support_role {
            let embed = create_error_embed(
                "Invalid Assignment",
                "Can only assign tickets to users with a support role",
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

        // Assign the ticket
        crate::database::ticket::assign_ticket(&db.pool, ticket.id, assigned_user_id).await?;

        // Send DM to assigned user
        if let Ok(user) = serenity::all::UserId::new(assigned_user_id as u64).to_user(&ctx.http).await {
            if let Ok(dm_channel) = user.create_dm_channel(&ctx.http).await {
                let dm_embed = crate::utils::create_embed(
                    "Ticket Assigned",
                    format!(
                        "You have been assigned to a ticket!\n\n\
                        **Ticket:** #{}\n\
                        **Channel:** <#{}>\n\
                        **Assigned by:** <@{}>\n\
                        **Ticket Owner:** <@{}>\n\n\
                        Please check the ticket channel to assist the user.",
                        ticket.ticket_number,
                        ticket.channel_id,
                        assigner_id,
                        ticket.owner_id
                    )
                ).color(0x5865F2);

                let _ = dm_channel.send_message(
                    &ctx.http,
                    serenity::all::CreateMessage::new().embed(dm_embed)
                ).await;
            }
        }

        // Send log message
        let guild = crate::database::ticket::get_or_create_guild(&db.pool, ticket.guild_id).await?;
        let log_embed = crate::utils::create_embed(
            "Ticket Assigned",
            format!(
                "Ticket: ticket-{}\nAssigned to: <@{}>\nAssigned by: <@{}>\nOwner: <@{}>",
                ticket.owner_id, assigned_user_id, assigner_id, ticket.owner_id
            )
        );
        let _ = crate::utils::send_log(ctx, guild.log_channel_id, log_embed).await;

        // Send success message in channel
        let embed = create_success_embed(
            "Ticket Assigned",
            format!("<@{}> has been assigned to this ticket by <@{}>", assigned_user_id, assigner_id),
        );

        interaction.channel_id
            .send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed))
            .await?;

        interaction
            .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .content("Ticket assigned successfully")
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
    CreateCommand::new("assign")
        .description("Assign a ticket to a support team member")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::User,
                "user",
                "The user to assign this ticket to"
            )
            .required(true)
        )
}
