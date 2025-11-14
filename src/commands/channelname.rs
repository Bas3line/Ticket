use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    ResolvedValue,
};
use crate::database::Database;
use crate::utils::{create_error_embed, create_success_embed, create_embed};
use anyhow::Result;

pub async fn run(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;

    let options = &interaction.data.options();
    let subcommand_name = options.first().map(|o| o.name.as_ref());

    match subcommand_name {
        Some("set") => {
            let template = options
                .iter()
                .find(|o| o.name == "set")
                .and_then(|o| {
                    if let ResolvedValue::SubCommand(sub_options) = &o.value {
                        sub_options.iter().find(|so| so.name == "template")
                            .and_then(|so| {
                                if let ResolvedValue::String(s) = so.value {
                                    Some(s.to_string())
                                } else {
                                    None
                                }
                            })
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "ticket-$ticket_number".to_string());

            crate::database::ticket::update_channel_name_template(&db.pool, guild_id, &template).await?;

            let embed = create_success_embed(
                "Channel Name Template Updated",
                format!("Channel name template set to: `{}`\n\nExample: `{}`",
                    template,
                    crate::database::ticket::format_channel_name(&template, 123, 123456789, "username")
                )
            );

            interaction
                .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                    serenity::all::CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true)
                ))
                .await?;
        },
        Some("view") | None => {
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
                    `{}`",
                    template,
                    crate::database::ticket::format_channel_name(&template, 123, 123456789, "username")
                )
            ).color(0x5865F2);

            interaction
                .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                    serenity::all::CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true)
                ))
                .await?;
        },
        _ => {
            let embed = create_error_embed("Error", "Invalid subcommand");
            interaction
                .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                    serenity::all::CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true)
                ))
                .await?;
        }
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("channel-name")
        .description("Manage ticket channel name template")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "set",
                "Set custom channel name template",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "template",
                    "Template (use $ticket_number, $user_id, $user_name)",
                )
                .required(true)
            )
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "view",
                "View current channel name template and documentation",
            )
        )
}
