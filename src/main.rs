mod commands;
mod config;
mod database;
mod handlers;
mod models;
mod prefix;
mod utils;

use anyhow::Result;
use serenity::all::{
    Client, Context, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler,
    GatewayIntents, Interaction, Message, Ready,
};
use std::sync::Arc;
use tracing::{error, info};

struct Handler {
    db: Arc<database::Database>,
    owner_id: u64,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected and ready!", ready.user.name);

        let commands = vec![
            commands::setup::register(),
            commands::supportrole::register(),
            commands::category::register(),
            commands::panel::register(),
            commands::close::register(),
            commands::stats::register(),
            commands::priority::register(),
            commands::blacklist::register(),
            commands::note::register(),
        ];

        for command in commands {
            if let Err(e) = serenity::all::Command::create_global_command(&ctx.http, command).await {
                error!("Failed to register command: {}", e);
            }
        }

        info!("Commands registered successfully");
    }

    async fn guild_create(&self, ctx: Context, guild: serenity::all::Guild, _is_new: Option<bool>) {
        if let Ok(true) = database::ticket::is_blacklisted(&self.db.pool, guild.id.get() as i64, "guild").await {
            info!("Leaving blacklisted guild: {} ({})", guild.name, guild.id);
            let _ = guild.id.leave(&ctx.http).await;
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let result = match command.data.name.as_str() {
                    "setup" => commands::setup::run(&ctx, &command, &self.db).await,
                    "supportrole" => commands::supportrole::run(&ctx, &command, &self.db).await,
                    "category" => commands::category::run(&ctx, &command, &self.db).await,
                    "panel" => commands::panel::run(&ctx, &command, &self.db).await,
                    "close" => commands::close::run(&ctx, &command, &self.db).await,
                    "stats" => commands::stats::run(&ctx, &command, &self.db).await,
                    "priority" => commands::priority::run(&ctx, &command, &self.db).await,
                    "blacklist" => commands::blacklist::run(&ctx, &command, &self.db).await,
                    "note" => commands::note::run(&ctx, &command, &self.db).await,
                    _ => Ok(()),
                };

                if let Err(e) = result {
                    error!("Command error: {}", e);
                    let embed = utils::create_error_embed("Error", format!("An error occurred: {}", e));
                    let _ = command
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .embed(embed)
                                    .ephemeral(true),
                            ),
                        )
                        .await;
                }
            }
            Interaction::Component(component) => {
                let result = match component.data.custom_id.as_str() {
                    "ticket_create" => handlers::button::handle_ticket_create(&ctx, &component, &self.db).await,
                    "ticket_claim" => handlers::button::handle_ticket_claim(&ctx, &component, &self.db).await,
                    "ticket_unclaim" => handlers::button::handle_ticket_unclaim(&ctx, &component, &self.db).await,
                    "ticket_close" => handlers::button::handle_ticket_close(&ctx, &component, &self.db).await,
                    "ticket_transcript" => handlers::button::handle_ticket_transcript(&ctx, &component, &self.db).await,
                    "help_menu" => handlers::menus::handle_help_menu(&ctx, &component, &self.db).await,
                    "setup_menu" => handlers::menus::handle_setup_menu(&ctx, &component, &self.db).await,
                    "setup_category_select" => handlers::menus::handle_setup_category_select(&ctx, &component, &self.db).await,
                    "setup_logs_select" => handlers::menus::handle_setup_logs_select(&ctx, &component, &self.db).await,
                    "setup_transcripts_select" => handlers::menus::handle_setup_transcripts_select(&ctx, &component, &self.db).await,
                    "setup_support_role_select" => handlers::menus::handle_setup_support_role_select(&ctx, &component, &self.db).await,
                    "setup_ping_role_select" => handlers::menus::handle_setup_ping_role_select(&ctx, &component, &self.db).await,
                    "setup_send_panel" => handlers::menus::handle_send_panel(&ctx, &component, &self.db).await,
                    "setup_delete_panel" => handlers::menus::handle_delete_panel(&ctx, &component, &self.db).await,
                    "panel_edit_embed" => handlers::menus::handle_panel_edit(&ctx, &component, &self.db).await,
                    "panel_edit_advanced" => handlers::menus::handle_panel_edit_advanced(&ctx, &component, &self.db).await,
                    "panel_send_here" => handlers::menus::handle_panel_send_here(&ctx, &component, &self.db).await,
                    "panel_cancel" => handlers::menus::handle_panel_cancel(&ctx, &component, &self.db).await,
                    "delete_panel_select" => handlers::menus::handle_delete_panel_select(&ctx, &component, &self.db).await,
                    "category_add" => handlers::menus::handle_category_add(&ctx, &component, &self.db).await,
                    "category_edit_select" => handlers::menus::handle_category_edit_select(&ctx, &component, &self.db).await,
                    "panel_category_select" => handlers::menus::handle_panel_category_select(&ctx, &component, &self.db).await,
                    "ticket_category_select" => handlers::menus::handle_ticket_category_select(&ctx, &component, &self.db).await,
                    id if id.starts_with("category_delete_confirm_") => handlers::menus::handle_category_delete_confirm(&ctx, &component, &self.db).await,
                    id if id.starts_with("panel_style_button_") => handlers::menus::handle_panel_style_choice(&ctx, &component, &self.db, true).await,
                    id if id.starts_with("panel_style_dropdown_") => handlers::menus::handle_panel_style_choice(&ctx, &component, &self.db, false).await,
                    id if id.starts_with("panel_button_color_") => handlers::menus::handle_panel_button_color_select(&ctx, &component, &self.db).await,
                    id if id.starts_with("ticket_create_cat_") => handlers::button::handle_ticket_create_category(&ctx, &component, &self.db).await,
                    _ => Ok(()),
                };

                if let Err(e) = result {
                    error!("Component interaction error: {}", e);
                    let embed = utils::create_error_embed("Error", format!("An error occurred: {}", e));
                    let _ = component
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .embed(embed)
                                    .ephemeral(true),
                            ),
                        )
                        .await;
                }
            }
            Interaction::Modal(modal) => {
                let result = match modal.data.custom_id.as_str() {
                    "panel_edit_modal" => handlers::menus::handle_panel_edit_modal(&ctx, &modal, &self.db).await,
                    "category_add_modal" => handlers::menus::handle_category_add_modal(&ctx, &modal, &self.db).await,
                    _ => Ok(()),
                };

                if let Err(e) = result {
                    error!("Modal interaction error: {}", e);
                    let embed = utils::create_error_embed("Error", format!("An error occurred: {}", e));
                    let _ = modal
                        .create_response(
                            &ctx.http,
                            serenity::all::CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .embed(embed)
                                    .ephemeral(true),
                            ),
                        )
                        .await;
                }
            }
            _ => {}
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if let Ok(true) = database::ticket::is_blacklisted(&self.db.pool, msg.author.id.get() as i64, "user").await {
            return;
        }

        if let Some(guild_id) = msg.guild_id {
            if let Ok(true) = database::ticket::is_blacklisted(&self.db.pool, guild_id.get() as i64, "guild").await {
                let _ = guild_id.leave(&ctx.http).await;
                return;
            }
        }

        if msg.mentions.iter().any(|u| u.id == ctx.cache.current_user().id) {
            if let Err(e) = prefix::utility::bot_mention(&ctx, &msg, &self.db).await {
                error!("Bot mention handler error: {}", e);
            }
            return;
        }

        let guild_id = msg.guild_id.map(|g| g.get()).unwrap_or(0);
        let prefix = prefix::get_prefix(&self.db.pool, guild_id).await;

        if msg.content.starts_with(&prefix) {
            if let Err(e) = prefix::handle_prefix_command(&ctx, &msg, &self.db, &prefix, self.owner_id).await {
                error!("Prefix command error: {}", e);
            }
            return;
        }

        let channel_id = msg.channel_id.get() as i64;

        if let Ok(Some(ticket)) = database::ticket::get_ticket_by_channel(&self.db.pool, channel_id).await {
            let attachments = if msg.attachments.is_empty() {
                serde_json::json!([])
            } else {
                serde_json::json!(
                    msg.attachments
                        .iter()
                        .map(|a| {
                            serde_json::json!({
                                "filename": a.filename,
                                "url": a.url,
                                "size": a.size
                            })
                        })
                        .collect::<Vec<_>>()
                )
            };

            let author_avatar_url = msg.author.avatar_url();
            let author_discriminator = if let Some(disc) = msg.author.discriminator {
                if disc.get() != 0 {
                    Some(format!("{:04}", disc.get()))
                } else {
                    None
                }
            } else {
                None
            };

            if let Err(e) = database::ticket::add_ticket_message(
                &self.db.pool,
                ticket.id,
                msg.id.get() as i64,
                msg.author.id.get() as i64,
                msg.author.name.clone(),
                author_discriminator,
                author_avatar_url,
                msg.content.clone(),
                attachments,
            )
            .await
            {
                error!("Failed to log ticket message: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let config = config::Config::from_env()?;

    info!("Connecting to database...");
    let db = Arc::new(
        database::Database::new(&config.database_url, &config.redis_url).await?,
    );
    info!("Database connected successfully");

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(Handler {
            db,
            owner_id: config.owner_id,
        })
        .await?;

    info!("Starting bot...");
    client.start().await?;

    Ok(())
}
