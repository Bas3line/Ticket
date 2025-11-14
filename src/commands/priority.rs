use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    ResolvedValue,
};
use crate::database::Database;
use crate::utils::create_error_embed;
use anyhow::Result;

pub async fn run(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let channel_id = interaction.channel_id.get() as i64;

    let ticket = crate::database::ticket::get_ticket_by_channel(&db.pool, channel_id).await?;

    if let Some(ticket) = ticket {
        let options = &interaction.data.options();

        let priority = options
            .iter()
            .find(|o| o.name == "level")
            .and_then(|o| {
                if let ResolvedValue::String(s) = o.value {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "normal".to_string());

        let final_priority = if priority == "reset" { "normal" } else { &priority };

        sqlx::query("UPDATE tickets SET priority = $1 WHERE id = $2")
            .bind(final_priority)
            .bind(ticket.id)
            .execute(&db.pool)
            .await?;

        if priority == "low" || priority == "high" || priority == "urgent" {
            let redis_key = format!("priority_ping:{}", ticket.id);
            let mut redis_conn = db.redis.clone();
            let _: () = redis::cmd("SETEX")
                .arg(&redis_key)
                .arg(86400)
                .arg(&priority)
                .query_async(&mut redis_conn)
                .await
                .unwrap_or(());

            let ctx_clone = ctx.clone();
            let channel_id = interaction.channel_id;
            let ticket_id = ticket.id;
            let db_clone = db.clone();
            let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?.get() as i64;
            let priority_clone = priority.clone();

            tokio::spawn(async move {
                let interval_secs = match priority_clone.as_str() {
                    "low" => 7200,      // 120 minutes
                    "high" => 3600,     // 60 minutes
                    "urgent" => 3600,   // 60 minutes
                    _ => 3600,
                };

                // Ping immediately for high/urgent
                if priority_clone == "high" || priority_clone == "urgent" {
                    if let Ok(Some((ping_role_id,))) = sqlx::query_as::<_, (Option<i64>,)>(
                        "SELECT ping_role_id FROM guilds WHERE guild_id = $1"
                    )
                    .bind(guild_id)
                    .fetch_optional(&db_clone.pool)
                    .await
                    {
                        if let Some(role_id) = ping_role_id {
                            let priority_label = if priority_clone == "urgent" { "URGENT" } else { "High priority" };
                            let _ = channel_id.send_message(
                                &ctx_clone.http,
                                serenity::all::CreateMessage::new()
                                    .content(format!("<@&{}> {} ticket requires attention!", role_id, priority_label))
                            ).await;
                        }
                    }
                }

                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;

                    let mut redis_conn = db_clone.redis.clone();
                    let exists: Option<String> = redis::cmd("GET")
                        .arg(&format!("priority_ping:{}", ticket_id))
                        .query_async(&mut redis_conn)
                        .await
                        .ok();

                    if exists.is_none() {
                        break;
                    }

                    let ticket_check = sqlx::query_as::<_, crate::models::Ticket>(
                        "SELECT id, guild_id, channel_id, ticket_number, owner_id, category_id, claimed_by, assigned_to,
                                status, created_at, closed_at, priority, rating, last_activity, opening_message_id,
                                has_messages, last_message_at
                         FROM tickets WHERE id = $1"
                    )
                    .bind(ticket_id)
                    .fetch_optional(&db_clone.pool)
                    .await;

                    if ticket_check.is_err() || ticket_check.unwrap().is_none() {
                        let _: () = redis::cmd("DEL")
                            .arg(&format!("priority_ping:{}", ticket_id))
                            .query_async(&mut redis_conn)
                            .await
                            .unwrap_or(());
                        break;
                    }

                    if let Ok(Some((ping_role_id,))) = sqlx::query_as::<_, (Option<i64>,)>(
                        "SELECT ping_role_id FROM guilds WHERE guild_id = $1"
                    )
                    .bind(guild_id)
                    .fetch_optional(&db_clone.pool)
                    .await
                    {
                        if let Some(role_id) = ping_role_id {
                            let priority_label = match priority_clone.as_str() {
                                "urgent" => "URGENT",
                                "high" => "High priority",
                                "low" => "Low priority",
                                _ => "Priority",
                            };
                            let _ = channel_id.send_message(
                                &ctx_clone.http,
                                serenity::all::CreateMessage::new()
                                    .content(format!("<@&{}> {} ticket still needs attention!", role_id, priority_label))
                            ).await;
                        }
                    }
                }
            });
        } else {
            let redis_key = format!("priority_ping:{}", ticket.id);
            let mut redis_conn = db.redis.clone();
            let _: () = redis::cmd("DEL")
                .arg(&redis_key)
                .query_async(&mut redis_conn)
                .await
                .unwrap_or(());
        }

        let display_priority = if priority == "reset" { "normal" } else { &priority };

        let color = match display_priority {
            "low" => serenity::all::Colour::from_rgb(149, 165, 166),
            "normal" => serenity::all::Colour::from_rgb(88, 101, 242),
            "high" => serenity::all::Colour::from_rgb(241, 196, 15),
            "urgent" => serenity::all::Colour::from_rgb(231, 76, 60),
            _ => serenity::all::Colour::from_rgb(88, 101, 242),
        };

        let guild = crate::database::ticket::get_or_create_guild(&db.pool, ticket.guild_id).await?;
        let action_text = if priority == "reset" { "Priority Reset" } else { "Priority Set" };
        let log_embed = crate::utils::create_embed(
            action_text,
            format!("Ticket: ticket-{}\nPriority: **{}**\nSet by: <@{}>",
                ticket.owner_id, display_priority.to_uppercase(), interaction.user.id)
        );
        let _ = crate::utils::send_log(ctx, guild.log_channel_id, log_embed).await;

        let embed = serenity::all::CreateEmbed::new()
            .title("Priority Updated")
            .description(format!("Ticket priority set to: **{}**", display_priority.to_uppercase()))
            .color(color);

        interaction
            .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(embed)
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

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("priority")
        .description("Set ticket priority")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "level",
                "Priority level",
            )
            .required(true)
            .add_string_choice("Low", "low")
            .add_string_choice("Normal", "normal")
            .add_string_choice("High", "high")
            .add_string_choice("Urgent", "urgent")
            .add_string_choice("Reset", "reset"),
        )
}
