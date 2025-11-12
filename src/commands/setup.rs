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
            "category" => {
                if let Some(ResolvedOption {
                    value: ResolvedValue::Channel(channel),
                    ..
                }) = sub_options.iter().find(|o| o.name == "channel")
                {
                    let guild_id = interaction.guild_id.unwrap().get() as i64;
                    crate::database::ticket::update_guild_category(
                        &db.pool,
                        guild_id,
                        channel.id.get() as i64,
                    )
                    .await?;

                    let embed = create_success_embed(
                        "Category Set",
                        format!("Ticket category set to <#{}>", channel.id),
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
            "logs" => {
                if let Some(ResolvedOption {
                    value: ResolvedValue::Channel(channel),
                    ..
                }) = sub_options.iter().find(|o| o.name == "channel")
                {
                    let guild_id = interaction.guild_id.unwrap().get() as i64;
                    crate::database::ticket::update_guild_log_channel(
                        &db.pool,
                        guild_id,
                        channel.id.get() as i64,
                    )
                    .await?;

                    let embed = create_success_embed(
                        "Log Channel Set",
                        format!("Log channel set to <#{}>", channel.id),
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
            "transcripts" => {
                if let Some(ResolvedOption {
                    value: ResolvedValue::Channel(channel),
                    ..
                }) = sub_options.iter().find(|o| o.name == "channel")
                {
                    let guild_id = interaction.guild_id.unwrap().get() as i64;
                    crate::database::ticket::update_guild_transcript_channel(
                        &db.pool,
                        guild_id,
                        channel.id.get() as i64,
                    )
                    .await?;

                    let embed = create_success_embed(
                        "Transcript Channel Set",
                        format!("Transcript channel set to <#{}>", channel.id),
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
            _ => {}
        }
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("setup")
        .description("Configure ticket bot settings")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "category",
                "Set the category for ticket channels",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "The category channel",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "logs",
                "Set the log channel",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "The log channel",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "transcripts",
                "Set the transcript channel",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "The transcript channel",
                )
                .required(true),
            ),
        )
}
