use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    ResolvedOption, ResolvedValue,
};
use serenity::prelude::Mentionable;
use crate::database::Database;
use crate::utils::create_success_embed;
use anyhow::Result;

pub async fn run(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let options = &interaction.data.options();

    if let Some(ResolvedOption {
        value: ResolvedValue::SubCommand(sub_options),
        ..
    }) = options.first()
    {
        match options.first().unwrap().name {
            "add" => {
                if let Some(ResolvedOption {
                    value: ResolvedValue::Role(role),
                    ..
                }) = sub_options.iter().find(|o| o.name == "role")
                {
                    let guild_id = interaction.guild_id.unwrap().get() as i64;
                    crate::database::ticket::add_support_role(
                        &db.pool,
                        guild_id,
                        role.id.get() as i64,
                    )
                    .await?;

                    let embed = create_success_embed(
                        "Support Role Added",
                        format!("Added {} as a support role", role.mention()),
                    );

                    interaction
                        .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                            serenity::all::CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .ephemeral(true)
                        ))
                        .await?;
                }
            }
            "remove" => {
                if let Some(ResolvedOption {
                    value: ResolvedValue::Role(role),
                    ..
                }) = sub_options.iter().find(|o| o.name == "role")
                {
                    let guild_id = interaction.guild_id.unwrap().get() as i64;
                    crate::database::ticket::remove_support_role(
                        &db.pool,
                        guild_id,
                        role.id.get() as i64,
                    )
                    .await?;

                    let embed = create_success_embed(
                        "Support Role Removed",
                        format!("Removed {} from support roles", role.mention()),
                    );

                    interaction
                        .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                            serenity::all::CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .ephemeral(true)
                        ))
                        .await?;
                }
            }
            "list" => {
                let guild_id = interaction.guild_id.unwrap().get() as i64;
                let roles = crate::database::ticket::get_support_roles(&db.pool, guild_id).await?;

                let description = if roles.is_empty() {
                    "No support roles configured".to_string()
                } else {
                    roles
                        .iter()
                        .map(|r| format!("<@&{}>", r.role_id))
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                let embed = create_success_embed("Support Roles", description);

                interaction
                    .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                        serenity::all::CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .ephemeral(true)
                    ))
                    .await?;
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("supportrole")
        .description("Manage support roles")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "add",
                "Add a support role",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Role, "role", "The role to add")
                    .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "remove",
                "Remove a support role",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Role, "role", "The role to remove")
                    .required(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "List all support roles",
        ))
}
