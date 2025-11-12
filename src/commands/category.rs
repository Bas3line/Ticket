use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    ResolvedOption, ResolvedValue,
};
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
                let name = sub_options
                    .iter()
                    .find(|o| o.name == "name")
                    .and_then(|o| {
                        if let ResolvedValue::String(s) = o.value {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap();

                let description = sub_options
                    .iter()
                    .find(|o| o.name == "description")
                    .and_then(|o| {
                        if let ResolvedValue::String(s) = o.value {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    });

                let emoji = sub_options
                    .iter()
                    .find(|o| o.name == "emoji")
                    .and_then(|o| {
                        if let ResolvedValue::String(s) = o.value {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    });

                let guild_id = interaction.guild_id.unwrap().get() as i64;
                crate::database::ticket::create_ticket_category(
                    &db.pool,
                    guild_id,
                    name.clone(),
                    description,
                    emoji,
                )
                .await?;

                let embed = create_success_embed(
                    "Category Created",
                    format!("Created ticket category: {}", name),
                );

                interaction
                    .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                        serenity::all::CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .ephemeral(true)
                    ))
                    .await?;
            }
            "list" => {
                let guild_id = interaction.guild_id.unwrap().get() as i64;
                let categories = crate::database::ticket::get_ticket_categories(&db.pool, guild_id).await?;

                let description = if categories.is_empty() {
                    "No categories configured".to_string()
                } else {
                    categories
                        .iter()
                        .map(|c| {
                            let emoji_str = c.emoji.as_deref().unwrap_or("");
                            format!("{} {} - {}", emoji_str, c.name, c.description.as_deref().unwrap_or("No description"))
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                let embed = create_success_embed("Ticket Categories", description);

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
    CreateCommand::new("category")
        .description("Manage ticket categories")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "add",
                "Add a ticket category",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "name",
                    "Category name",
                )
                .required(true),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::String,
                "description",
                "Category description",
            ))
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::String,
                "emoji",
                "Category emoji",
            )),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "List all categories",
        ))
}
