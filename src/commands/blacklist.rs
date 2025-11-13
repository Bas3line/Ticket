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
                    value: ResolvedValue::User(user, _),
                    ..
                }) = sub_options.iter().find(|o| o.name == "user")
                {
                    let reason = sub_options
                        .iter()
                        .find(|o| o.name == "reason")
                        .and_then(|o| {
                            if let ResolvedValue::String(s) = o.value {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        });

                    let _guild_id = interaction.guild_id.unwrap().get() as i64;
                    let user_id = user.id.get() as i64;
                    let blacklisted_by = interaction.user.id.get() as i64;

                    sqlx::query(
                        "INSERT INTO blacklist (target_id, target_type, reason, blacklisted_by)
                         VALUES ($1, $2, $3, $4)
                         ON CONFLICT (target_id) DO UPDATE
                         SET reason = $3, blacklisted_by = $4"
                    )
                    .bind(user_id)
                    .bind("user")
                    .bind(reason.as_deref())
                    .bind(blacklisted_by)
                    .execute(&db.pool)
                    .await?;

                    let embed = create_success_embed(
                        "User Blacklisted",
                        format!("{} has been blacklisted from creating tickets", user.mention()),
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
                    value: ResolvedValue::User(user, _),
                    ..
                }) = sub_options.iter().find(|o| o.name == "user")
                {
                    let _guild_id = interaction.guild_id.unwrap().get() as i64;
                    let user_id = user.id.get() as i64;

                    sqlx::query("DELETE FROM blacklist WHERE target_id = $1 AND target_type = 'user'")
                        .bind(user_id)
                        .execute(&db.pool)
                        .await?;

                    let embed = create_success_embed(
                        "User Unblacklisted",
                        format!("{} can now create tickets", user.mention()),
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
                let _guild_id = interaction.guild_id.unwrap().get() as i64;

                let blacklisted: Vec<(i64, Option<String>)> = sqlx::query_as(
                    "SELECT target_id, reason FROM blacklist WHERE target_type = 'user' ORDER BY created_at DESC"
                )
                .fetch_all(&db.pool)
                .await?;

                let description = if blacklisted.is_empty() {
                    "No blacklisted users".to_string()
                } else {
                    blacklisted
                        .iter()
                        .map(|(user_id, reason)| {
                            let reason_str = reason.as_deref().unwrap_or("No reason provided");
                            format!("<@{}> - {}", user_id, reason_str)
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                let embed = create_success_embed("Blacklisted Users", description);

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
    CreateCommand::new("blacklist")
        .description("Manage blacklisted users")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "add",
                "Blacklist a user",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "The user to blacklist")
                    .required(true),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::String,
                "reason",
                "Reason for blacklist",
            )),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "remove",
                "Unblacklist a user",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "The user to unblacklist")
                    .required(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "List all blacklisted users",
        ))
}
