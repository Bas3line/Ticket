use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    ResolvedOption, ResolvedValue,
};
use crate::database::Database;
use crate::utils::{create_error_embed, create_success_embed};
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
                let channel_id = interaction.channel_id.get() as i64;

                let ticket = crate::database::ticket::get_ticket_by_channel(&db.pool, channel_id).await?;

                if let Some(ticket) = ticket {
                    let note = sub_options
                        .iter()
                        .find(|o| o.name == "note")
                        .and_then(|o| {
                            if let ResolvedValue::String(s) = o.value {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap();

                    let author_id = interaction.user.id.get() as i64;

                    sqlx::query(
                        "INSERT INTO ticket_notes (ticket_id, author_id, note)
                         VALUES ($1, $2, $3)"
                    )
                    .bind(ticket.id)
                    .bind(author_id)
                    .bind(&note)
                    .execute(&db.pool)
                    .await?;

                    // Send log
                    let guild = crate::database::ticket::get_or_create_guild(&db.pool, ticket.guild_id).await?;
                    let log_embed = crate::utils::create_embed(
                        "Note Added",
                        format!("Ticket #{}\nNote added by: <@{}>\nNote: {}", ticket.ticket_number, author_id, note)
                    );
                    let _ = crate::utils::send_log(ctx, guild.log_channel_id, log_embed).await;

                    let embed = create_success_embed("Note Added", "Note has been added to this ticket");

                    interaction
                        .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                            serenity::all::CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .ephemeral(true)
                        ))
                        .await?;
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
            }
            "list" => {
                let channel_id = interaction.channel_id.get() as i64;

                let ticket = crate::database::ticket::get_ticket_by_channel(&db.pool, channel_id).await?;

                if let Some(ticket) = ticket {
                    let notes: Vec<(i64, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
                        "SELECT author_id, note, created_at FROM ticket_notes
                         WHERE ticket_id = $1 ORDER BY created_at ASC"
                    )
                    .bind(ticket.id)
                    .fetch_all(&db.pool)
                    .await?;

                    let description = if notes.is_empty() {
                        "No notes for this ticket".to_string()
                    } else {
                        notes
                            .iter()
                            .map(|(author_id, note, created_at)| {
                                format!(
                                    "<@{}> - <t:{}:R>\n{}",
                                    author_id,
                                    created_at.timestamp(),
                                    note
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n\n")
                    };

                    let embed = create_success_embed("Ticket Notes", description);

                    interaction
                        .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                            serenity::all::CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .ephemeral(true)
                        ))
                        .await?;
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
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("note")
        .description("Manage ticket notes")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "add",
                "Add a note to this ticket",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "note", "The note content")
                    .required(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "List all notes for this ticket",
        ))
}
