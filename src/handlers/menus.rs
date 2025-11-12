use serenity::all::{ComponentInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage, CreateEmbed, CreateModal, CreateInputText, InputTextStyle};
use crate::database::Database;
use crate::utils::{create_error_embed, create_embed, create_success_embed};
use anyhow::Result;
use tracing::info;

pub async fn handle_help_menu(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let guild_id = interaction.guild_id.map(|g| g.get() as i64).unwrap_or(0);
    let prefix = crate::database::ticket::get_guild_prefix(&db.pool, guild_id).await?;

    let selection = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::StringSelect { values } => {
            if values.is_empty() {
                return Ok(());
            }
            &values[0]
        }
        _ => return Ok(()),
    };

    info!("Help menu selection: {} in guild {}", selection, guild_id);

    let embed = match selection.as_str() {
        "help_tickets" => create_ticket_commands_embed(&prefix),
        "help_setup" => create_setup_commands_embed(&prefix),
        "help_admin" => create_admin_commands_embed(&prefix),
        "help_premium" => create_premium_commands_embed(&prefix),
        "help_owner" => create_owner_commands_embed(&prefix),
        _ => create_error_embed("Error", "Unknown option selected"),
    };

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_setup_menu(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let selection = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::StringSelect { values } => {
            if values.is_empty() {
                return Ok(());
            }
            &values[0]
        }
        _ => return Ok(()),
    };

    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
    info!("Setup menu selection: {} in guild {}", selection, guild_id.get());

    match selection.as_str() {
        "setup_category" => {
            let embed = create_embed(
                "Select Ticket Category",
                "Select the category where new ticket channels will be created."
            ).color(0x5865F2);

            let channel_select = serenity::all::CreateSelectMenu::new(
                "setup_category_select",
                serenity::all::CreateSelectMenuKind::Channel {
                    channel_types: Some(vec![serenity::all::ChannelType::Category]),
                    default_channels: None,
                }
            ).placeholder("Select a category channel");

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .components(vec![serenity::all::CreateActionRow::SelectMenu(channel_select)])
                            .ephemeral(true),
                    ),
                )
                .await?;
        },
        "setup_logs" => {
            let embed = create_embed(
                "Select Log Channel",
                "Select the channel where all ticket actions will be logged."
            ).color(0x5865F2);

            let channel_select = serenity::all::CreateSelectMenu::new(
                "setup_logs_select",
                serenity::all::CreateSelectMenuKind::Channel {
                    channel_types: Some(vec![serenity::all::ChannelType::Text]),
                    default_channels: None,
                }
            ).placeholder("Select a log channel");

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .components(vec![serenity::all::CreateActionRow::SelectMenu(channel_select)])
                            .ephemeral(true),
                    ),
                )
                .await?;
        },
        "setup_transcripts" => {
            let embed = create_embed(
                "Select Transcript Channel",
                "Select the channel where ticket transcripts will be sent when tickets are closed."
            ).color(0x5865F2);

            let channel_select = serenity::all::CreateSelectMenu::new(
                "setup_transcripts_select",
                serenity::all::CreateSelectMenuKind::Channel {
                    channel_types: Some(vec![serenity::all::ChannelType::Text]),
                    default_channels: None,
                }
            ).placeholder("Select a transcript channel");

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .components(vec![serenity::all::CreateActionRow::SelectMenu(channel_select)])
                            .ephemeral(true),
                    ),
                )
                .await?;
        },
        "setup_claim" => {
            let guild = crate::database::ticket::get_or_create_guild(&db.pool, guild_id.get() as i64).await?;
            let current_status = guild.claim_buttons_enabled.unwrap_or(true);

            crate::database::ticket::update_guild_settings(
                &db.pool,
                guild_id.get() as i64,
                Some(!current_status),
                None,
                None,
                None,
                None
            ).await?;

            info!("Toggled claim buttons to {} for guild {}", !current_status, guild_id.get());

            let embed = create_success_embed(
                "Claim Buttons Toggled",
                format!("Claim buttons are now **{}**", if !current_status { "enabled" } else { "disabled" })
            );

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .ephemeral(true),
                    ),
                )
                .await?;
        },
        "setup_support_role" => {
            let embed = create_embed(
                "Select Support Role",
                "Select the role that should have access to manage tickets."
            ).color(0x5865F2);

            let role_select = serenity::all::CreateSelectMenu::new(
                "setup_support_role_select",
                serenity::all::CreateSelectMenuKind::Role {
                    default_roles: None,
                }
            ).placeholder("Select a support role");

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .components(vec![serenity::all::CreateActionRow::SelectMenu(role_select)])
                            .ephemeral(true),
                    ),
                )
                .await?;
        },
        "setup_ping_role" => {
            let embed = create_embed(
                "Select Ping Role",
                "Select the role that should be pinged when a new ticket is created."
            ).color(0x5865F2);

            let role_select = serenity::all::CreateSelectMenu::new(
                "setup_ping_role_select",
                serenity::all::CreateSelectMenuKind::Role {
                    default_roles: None,
                }
            ).placeholder("Select a ping role");

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .components(vec![serenity::all::CreateActionRow::SelectMenu(role_select)])
                            .ephemeral(true),
                    ),
                )
                .await?;
        },
        "setup_manage_categories" => {
            let categories: Vec<(String, String, Option<String>)> = sqlx::query_as(
                "SELECT id::text, name, emoji FROM ticket_categories WHERE guild_id = $1 ORDER BY created_at"
            )
            .bind(guild_id.get() as i64)
            .fetch_all(&db.pool)
            .await?;

            info!("Found {} categories for guild {}", categories.len(), guild_id.get());

            let add_category_button = serenity::all::CreateButton::new("category_add")
                .label("Add Category")
                .style(serenity::all::ButtonStyle::Success);

            if categories.is_empty() {
                let embed = create_embed(
                    "Manage Ticket Categories",
                    "No categories found. Add a category to get started."
                ).color(0x5865F2);

                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .components(vec![serenity::all::CreateActionRow::Buttons(vec![add_category_button])])
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            } else {
                let category_list = categories.iter()
                    .map(|(_, name, emoji)| {
                        let emoji_display = emoji.as_ref().map(|e| format!("{} ", e)).unwrap_or_default();
                        format!("• {}{}", emoji_display, name)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                let embed = create_embed(
                    "Manage Ticket Categories",
                    format!("**Current Categories:**\n{}\n\nUse the buttons below to manage categories.", category_list)
                ).color(0x5865F2);

                let select_options: Vec<_> = categories.iter().map(|(id, name, emoji)| {
                    let display = if let Some(e) = emoji {
                        format!("{} {}", e, name)
                    } else {
                        name.clone()
                    };
                    serenity::all::CreateSelectMenuOption::new(display, format!("category_edit_{}", id))
                }).collect();

                let edit_select = serenity::all::CreateSelectMenu::new(
                    "category_edit_select",
                    serenity::all::CreateSelectMenuKind::String {
                        options: select_options,
                    }
                ).placeholder("Select a category to edit/delete");

                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .components(vec![
                                    serenity::all::CreateActionRow::SelectMenu(edit_select),
                                    serenity::all::CreateActionRow::Buttons(vec![add_category_button]),
                                ])
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
        },
        "setup_settings" => {
            let guild = crate::database::ticket::get_or_create_guild(&db.pool, guild_id.get() as i64).await?;
            let prefix = crate::prefix::get_prefix(&db.pool, guild_id.get()).await;

            let embed = create_embed(
                "All Settings",
                format!(
                    "**Category:** {}\n\
                    **Logs:** {}\n\
                    **Transcripts:** {}\n\
                    **Claim Buttons:** {}\n\
                    **Auto Close:** {}\n\
                    **Prefix:** `{}`",
                    guild.ticket_category_id.map(|id| format!("<#{}>", id)).unwrap_or("Not Set".to_string()),
                    guild.log_channel_id.map(|id| format!("<#{}>", id)).unwrap_or("Not Set".to_string()),
                    guild.transcript_channel_id.map(|id| format!("<#{}>", id)).unwrap_or("Not Set".to_string()),
                    if guild.claim_buttons_enabled.unwrap_or(true) { "Enabled" } else { "Disabled" },
                    if guild.auto_close_hours.unwrap_or(0) == 0 { "Disabled".to_string() } else { format!("{} hours", guild.auto_close_hours.unwrap_or(0)) },
                    prefix
                )
            );

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .ephemeral(true),
                    ),
                )
                .await?;
        },
        _ => return Ok(()),
    }

    Ok(())
}

pub async fn handle_setup_category_select(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let channel_id = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::ChannelSelect { values } => {
            if values.is_empty() {
                return Ok(());
            }
            values[0].get() as i64
        }
        _ => return Ok(()),
    };

    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    crate::database::ticket::update_guild_category(&db.pool, guild_id.get() as i64, channel_id).await?;

    info!("Set ticket category to {} for guild {}", channel_id, guild_id.get());

    let mut redis_conn = db.redis.clone();
    let _: () = redis::cmd("SET")
        .arg(format!("setup:{}:category", guild_id.get()))
        .arg(channel_id)
        .query_async(&mut redis_conn)
        .await
        .unwrap_or(());

    let embed = create_success_embed(
        "Category Set",
        format!("Ticket category has been set to <#{}>", channel_id)
    );

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![])
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_setup_logs_select(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let channel_id = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::ChannelSelect { values } => {
            if values.is_empty() {
                return Ok(());
            }
            values[0].get() as i64
        }
        _ => return Ok(()),
    };

    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    crate::database::ticket::update_guild_log_channel(&db.pool, guild_id.get() as i64, channel_id).await?;

    info!("Set log channel to {} for guild {}", channel_id, guild_id.get());

    let mut redis_conn = db.redis.clone();
    let _: () = redis::cmd("SET")
        .arg(format!("setup:{}:logs", guild_id.get()))
        .arg(channel_id)
        .query_async(&mut redis_conn)
        .await
        .unwrap_or(());

    let embed = create_success_embed(
        "Log Channel Set",
        format!("Log channel has been set to <#{}>", channel_id)
    );

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![])
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_setup_transcripts_select(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let channel_id = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::ChannelSelect { values } => {
            if values.is_empty() {
                return Ok(());
            }
            values[0].get() as i64
        }
        _ => return Ok(()),
    };

    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    crate::database::ticket::update_guild_transcript_channel(&db.pool, guild_id.get() as i64, channel_id).await?;

    info!("Set transcript channel to {} for guild {}", channel_id, guild_id.get());

    let mut redis_conn = db.redis.clone();
    let _: () = redis::cmd("SET")
        .arg(format!("setup:{}:transcripts", guild_id.get()))
        .arg(channel_id)
        .query_async(&mut redis_conn)
        .await
        .unwrap_or(());

    let embed = create_success_embed(
        "Transcript Channel Set",
        format!("Transcript channel has been set to <#{}>", channel_id)
    );

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![])
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_setup_support_role_select(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let role_id = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::RoleSelect { values } => {
            if values.is_empty() {
                return Ok(());
            }
            values[0].get() as i64
        }
        _ => return Ok(()),
    };

    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    // Add to support_roles table
    sqlx::query(
        "INSERT INTO support_roles (guild_id, role_id) VALUES ($1, $2) ON CONFLICT (guild_id, role_id) DO NOTHING"
    )
    .bind(guild_id.get() as i64)
    .bind(role_id)
    .execute(&db.pool)
    .await?;

    info!("Added support role {} for guild {}", role_id, guild_id.get());

    let embed = create_success_embed(
        "Support Role Added",
        format!("Support role <@&{}> has been added", role_id)
    );

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![])
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_setup_ping_role_select(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let role_id = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::RoleSelect { values } => {
            if values.is_empty() {
                return Ok(());
            }
            values[0].get() as i64
        }
        _ => return Ok(()),
    };

    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    // Update guilds table with ping_role_id
    sqlx::query(
        "UPDATE guilds SET ping_role_id = $1 WHERE guild_id = $2"
    )
    .bind(role_id)
    .bind(guild_id.get() as i64)
    .execute(&db.pool)
    .await?;

    info!("Set ping role to {} for guild {}", role_id, guild_id.get());

    let embed = create_success_embed(
        "Ping Role Set",
        format!("Ping role has been set to <@&{}>", role_id)
    );

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![])
            ),
        )
        .await?;

    Ok(())
}

fn create_ticket_commands_embed(prefix: &str) -> CreateEmbed {
    create_embed(
        "Ticket Commands",
        format!(
            "**Prefix Commands:**\n\
            `{}close` - Close current ticket and generate transcript\n\
            `{}claim` - Claim ticket as yours\n\
            `{}transcript` - Generate and download transcript\n\n\
            **Slash Commands:**\n\
            `/close` - Close ticket channel\n\
            `/priority <level>` - Set priority (low/normal/high/urgent)\n\
            `/note <text>` - Add private note to ticket\n\n\
            **Auto-Ping Feature:**\n\
            • High/Urgent: Pings immediately + every 60min\n\
            • Low: Pings every 120min\n\
            • Transcripts include pfp, timestamps, and inline images\n\
            • Automatic message cleanup after close",
            prefix, prefix, prefix
        )
    )
    .color(0x5865F2)
}

fn create_setup_commands_embed(prefix: &str) -> CreateEmbed {
    create_embed(
        "Setup & Configuration",
        format!(
            "**Basic Setup:**\n\
            `{}setup` - Interactive setup wizard with dropdown menus\n\
            `{}prefix <new>` - Change bot prefix (default: `!`)\n\
            `{}settings` - View all server settings\n\n\
            **Setup Options:**\n\
            • Ticket category channel\n\
            • Log channel for ticket events\n\
            • Transcript channel for closed tickets\n\
            • Support roles (can manage tickets)\n\
            • Ping role (mentioned on ticket create)\n\
            • Toggle claim buttons\n\n\
            **Panel Management:**\n\
            • Send new ticket panel\n\
            • Edit existing panels (title, description, buttons)\n\
            • Delete panels",
            prefix, prefix, prefix
        )
    )
    .color(0x5865F2)
}

fn create_admin_commands_embed(prefix: &str) -> CreateEmbed {
    create_embed(
        "Admin Commands",
        format!(
            "**Panel Commands:**\n\
            `{}panel` - Create new ticket panel\n\
            `/panel [title] [description]` - Create panel (slash)\n\n\
            **Category Management:**\n\
            `{}category <name>` - Create ticket category\n\
            `/category` - Manage categories (slash)\n\n\
            **Role Management:**\n\
            `{}supportrole @role` - Add/remove support role\n\
            `/supportrole` - Manage support roles (slash)\n\n\
            **Ticket Management:**\n\
            `{}priority <level>` - Set ticket priority\n\
            `{}note <text>` - Add note to current ticket\n\
            `{}blacklist @user` - Blacklist user from creating tickets\n\
            `{}stats` - View server ticket statistics\n\n\
            **Panel Customization:**\n\
            • Custom button colors (red, blue, green, gray)\n\
            • Embed image, thumbnail, and footer\n\
            • Button or dropdown style selection\n\
            • Session-locked editing (one user at a time)",
            prefix, prefix, prefix, prefix, prefix, prefix, prefix
        )
    )
    .color(0x5865F2)
}

fn create_premium_commands_embed(prefix: &str) -> CreateEmbed {
    create_embed(
        "Premium Features",
        format!(
            "`{}profile` - View your server's premium status\n\n\
            **Premium Benefits:**\n\
            • **Up to 30 panels** (free: 1 panel)\n\
            • **Multiple tickets per user** (configurable limit)\n\
            • **Custom embed colors** and branding\n\
            • **Advanced customization** (images, thumbnails, footers)\n\
            • **Custom button colors** (red, blue, green, gray)\n\
            • **Priority system** with auto-ping for high priority\n\
            • **Enhanced transcripts** with inline images and formatting\n\
            • **Ticket categories** with emoji support\n\
            • **Private notes** on tickets\n\
            • **Detailed statistics** and analytics\n\
            • **Priority support** from bot team\n\n\
            **Free Features:**\n\
            • 1 ticket panel per server\n\
            • 1 open ticket per user\n\
            • Basic ticket management\n\
            • Transcripts and logging\n\
            • Support role management\n\n\
            Contact bot owner to upgrade to premium!",
            prefix
        )
    )
    .color(0xFFD700)
}

fn create_owner_commands_embed(prefix: &str) -> CreateEmbed {
    create_embed(
        "Owner Commands",
        format!(
            "`{}botstats` - View bot statistics\n\
            `{}roast` - Advanced debugging information\n\
            `{}addprem <guild_id> <days>` - Add premium\n\
            `{}removeprem <guild_id>` - Remove premium\n\
            `{}listprem` - List premium guilds\n\
            `{}blacklistuser <user_id>` - Blacklist user\n\
            `{}blacklistguild <guild_id>` - Blacklist guild\n\
            `{}unblacklistuser <user_id>` - Unblacklist user\n\
            `{}unblacklistguild <guild_id>` - Unblacklist guild\n\
            `{}listblacklist` - List blacklisted users/guilds",
            prefix, prefix, prefix, prefix, prefix, prefix, prefix, prefix, prefix, prefix
        )
    )
    .color(0xED4245)
}

pub async fn handle_send_panel(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    let categories: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT id::text, name, emoji FROM ticket_categories WHERE guild_id = $1 ORDER BY created_at LIMIT 25"
    )
    .bind(guild_id.get() as i64)
    .fetch_all(&db.pool)
    .await?;

    if categories.is_empty() {
        let embed = create_error_embed(
            "No Categories Found",
            "Please create at least one ticket category before sending a panel.\n\nUse the 'Manage Categories' option in setup."
        );

        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    let select_options: Vec<_> = categories.iter().map(|(id, name, emoji)| {
        let display = if let Some(e) = emoji {
            format!("{} {}", e, name)
        } else {
            name.clone()
        };
        serenity::all::CreateSelectMenuOption::new(display, id.clone())
    }).collect();

    let category_select = serenity::all::CreateSelectMenu::new(
        "panel_category_select",
        serenity::all::CreateSelectMenuKind::String {
            options: select_options,
        }
    ).placeholder("Select categories for this panel")
    .min_values(1)
    .max_values(categories.len().min(25) as u8);

    let embed = create_embed(
        "Configure Panel Categories",
        format!(
            "Select which ticket categories should appear on this panel.\n\n\
            **Available Categories:** {}\n\n\
            You can select multiple categories.",
            categories.len()
        )
    ).color(0x5865F2);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![serenity::all::CreateActionRow::SelectMenu(category_select)])
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_delete_panel(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    let panels: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT channel_id, message_id FROM ticket_panel WHERE guild_id = $1 ORDER BY created_at DESC LIMIT 25"
    )
    .bind(guild_id.get() as i64)
    .fetch_all(&db.pool)
    .await?;

    info!("Found {} panels for guild {}", panels.len(), guild_id.get());

    if panels.is_empty() {
        let embed = create_error_embed(
            "No Panels Found",
            "No ticket panels found in this server."
        );

        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    let select_options: Vec<_> = panels.iter().enumerate().map(|(idx, (channel_id, message_id))| {
        serenity::all::CreateSelectMenuOption::new(
            format!("Panel in #{} (ID: {})", channel_id, message_id),
            format!("delete_panel_{}_{}", channel_id, message_id)
        )
        .description(format!("Delete panel {}", idx + 1))
    }).collect();

    let select_menu = serenity::all::CreateSelectMenu::new(
        "delete_panel_select",
        serenity::all::CreateSelectMenuKind::String {
            options: select_options,
        }
    )
    .placeholder("Select a panel to delete");

    let components = vec![serenity::all::CreateActionRow::SelectMenu(select_menu)];

    let embed = create_embed(
        "Delete Panel",
        format!("Found {} panel(s). Select one to delete:", panels.len())
    ).color(0xED4245);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(components)
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_panel_edit(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let user_id = interaction.user.id.get();
    let session_key = format!("panel_edit_session:{}", user_id);

    let mut redis_conn = db.redis.clone();
    let existing_session: Option<String> = redis::cmd("GET")
        .arg(&session_key)
        .query_async(&mut redis_conn)
        .await
        .ok();

    if existing_session.is_some() {
        let embed = crate::utils::create_error_embed(
            "Session Active",
            "You already have an active panel editing session. Please finish or cancel it first."
        );

        interaction.create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true)
            )
        ).await?;
        return Ok(());
    }

    redis::cmd("SETEX")
        .arg(&session_key)
        .arg(600)
        .arg(user_id.to_string())
        .query_async::<()>(&mut redis_conn)
        .await?;

    let modal = CreateModal::new("panel_edit_modal", "Edit Panel Embed")
        .components(vec![
            serenity::all::CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Short, "Title", "panel_title")
                    .placeholder("Support Ticket")
                    .required(true)
            ),
            serenity::all::CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Paragraph, "Description", "panel_description")
                    .placeholder("Click the button below to create a ticket")
                    .required(true)
            ),
            serenity::all::CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Short, "Color (Hex)", "panel_color")
                    .placeholder("#5865F2")
                    .required(false)
            ),
            serenity::all::CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Short, "Footer", "panel_footer")
                    .placeholder("Powered by Ticket Bot")
                    .required(false)
            ),
        ]);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Modal(modal),
        )
        .await?;

    Ok(())
}

pub async fn handle_panel_edit_modal(
    ctx: &Context,
    interaction: &serenity::all::ModalInteraction,
    db: &Database,
) -> Result<()> {
    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    let inputs = &interaction.data.components;

    let title = inputs.get(0)
        .and_then(|row| row.components.get(0))
        .and_then(|comp| {
            if let serenity::all::ActionRowComponent::InputText(input) = comp {
                input.value.clone()
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Support Ticket".to_string());

    let description = inputs.get(1)
        .and_then(|row| row.components.get(0))
        .and_then(|comp| {
            if let serenity::all::ActionRowComponent::InputText(input) = comp {
                input.value.clone()
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Click the button below to create a ticket".to_string());

    let color_str = inputs.get(2)
        .and_then(|row| row.components.get(0))
        .and_then(|comp| {
            if let serenity::all::ActionRowComponent::InputText(input) = comp {
                input.value.clone()
            } else {
                None
            }
        });

    let footer = inputs.get(3)
        .and_then(|row| row.components.get(0))
        .and_then(|comp| {
            if let serenity::all::ActionRowComponent::InputText(input) = comp {
                input.value.as_ref().and_then(|v| if v.is_empty() { None } else { Some(v.clone()) })
            } else {
                None
            }
        });

    let color = if let Some(ref color_str) = color_str {
        let cleaned = color_str.trim_start_matches('#');
        i32::from_str_radix(cleaned, 16).unwrap_or(5865714)
    } else {
        5865714
    };

    sqlx::query(
        "UPDATE guilds SET embed_title = $1, embed_description = $2, embed_color = $3, embed_footer = $4 WHERE guild_id = $5"
    )
    .bind(&title)
    .bind(&description)
    .bind(color)
    .bind(&footer)
    .bind(guild_id.get() as i64)
    .execute(&db.pool)
    .await?;

    info!("Updated panel embed for guild {} - title: {}, color: #{:06X}", guild_id.get(), title, color);

    let mut redis_conn = db.redis.clone();
    let _: () = redis::cmd("SET")
        .arg(format!("panel:{}:title", guild_id.get()))
        .arg(&title)
        .query_async(&mut redis_conn)
        .await
        .unwrap_or(());

    let embed = create_success_embed(
        "Panel Updated",
        format!("Panel embed has been updated:\n\n**Title:** {}\n**Color:** #{:06X}", title, color)
    );

    interaction
        .create_response(
            &ctx.http,
            serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_panel_send_here(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
    let channel_id = interaction.channel_id;

    let guild = crate::database::ticket::get_or_create_guild(&db.pool, guild_id.get() as i64).await?;

    let embed_title = guild.embed_title.unwrap_or_else(|| "Support Ticket".to_string());
    let embed_description = guild.embed_description.unwrap_or_else(|| "Click the button below to create a ticket".to_string());
    let embed_color = guild.embed_color.unwrap_or(5865714);
    let embed_footer = guild.embed_footer;

    let mut panel_embed = create_embed(&embed_title, &embed_description)
        .color(embed_color);

    if let Some(footer) = &embed_footer {
        panel_embed = panel_embed.footer(serenity::all::CreateEmbedFooter::new(footer));
    }

    let create_button = serenity::all::CreateButton::new("ticket_create")
        .label("Create Ticket")
        .style(serenity::all::ButtonStyle::Primary);

    let components = vec![serenity::all::CreateActionRow::Buttons(vec![create_button])];

    let panel_msg = channel_id.send_message(
        &ctx.http,
        serenity::all::CreateMessage::new()
            .embed(panel_embed)
            .components(components)
    ).await?;

    sqlx::query(
        "INSERT INTO ticket_panel (guild_id, channel_id, message_id, title, description) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(guild_id.get() as i64)
    .bind(channel_id.get() as i64)
    .bind(panel_msg.id.get() as i64)
    .bind(&embed_title)
    .bind(&embed_description)
    .execute(&db.pool)
    .await?;

    info!("Panel sent to channel {} in guild {} (message ID: {})", channel_id.get(), guild_id.get(), panel_msg.id.get());

    let mut redis_conn = db.redis.clone();
    let _: () = redis::cmd("SADD")
        .arg(format!("panels:{}", guild_id.get()))
        .arg(panel_msg.id.get())
        .query_async(&mut redis_conn)
        .await
        .unwrap_or(());

    let success_embed = create_success_embed(
        "Panel Sent",
        format!("Panel has been sent to <#{}>", channel_id)
    );

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(success_embed)
                    .components(vec![])
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_panel_cancel(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let user_id = interaction.user.id.get();
    let session_key = format!("panel_edit_session:{}", user_id);

    let mut redis_conn = db.redis.clone();
    let _: () = redis::cmd("DEL")
        .arg(&session_key)
        .query_async(&mut redis_conn)
        .await
        .unwrap_or(());

    let embed = create_embed("Cancelled", "Panel creation cancelled.").color(0x95A5A6);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![])
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_delete_panel_select(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let selection = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::StringSelect { values } => {
            if values.is_empty() {
                return Ok(());
            }
            &values[0]
        }
        _ => return Ok(()),
    };

    let parts: Vec<&str> = selection.split('_').collect();
    if parts.len() < 4 {
        return Ok(());
    }

    let channel_id: i64 = parts[2].parse()?;
    let message_id: i64 = parts[3].parse()?;

    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    sqlx::query("DELETE FROM ticket_panel WHERE channel_id = $1 AND message_id = $2")
        .bind(channel_id)
        .bind(message_id)
        .execute(&db.pool)
        .await?;

    let _ = serenity::all::ChannelId::new(channel_id as u64)
        .delete_message(&ctx.http, serenity::all::MessageId::new(message_id as u64))
        .await;

    info!("Deleted panel {} from channel {} in guild {}", message_id, channel_id, guild_id.get());

    let mut redis_conn = db.redis.clone();
    let _: () = redis::cmd("SREM")
        .arg(format!("panels:{}", guild_id.get()))
        .arg(message_id)
        .query_async(&mut redis_conn)
        .await
        .unwrap_or(());

    let success_embed = create_success_embed(
        "Panel Deleted",
        format!("Panel (ID: {}) has been deleted.", message_id)
    );

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(success_embed)
                    .components(vec![])
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_category_add(
    ctx: &Context,
    interaction: &ComponentInteraction,
    _db: &Database,
) -> Result<()> {
    let modal = CreateModal::new("category_add_modal", "Add Ticket Category")
        .components(vec![
            serenity::all::CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Short, "Category Name", "category_name")
                    .placeholder("Support")
                    .required(true)
                    .max_length(50)
            ),
            serenity::all::CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Paragraph, "Description", "category_description")
                    .placeholder("General support tickets")
                    .required(true)
                    .max_length(500)
            ),
            serenity::all::CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Short, "Emoji (optional)", "category_emoji")
                    .placeholder("🎫")
                    .required(false)
            ),
        ]);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Modal(modal),
        )
        .await?;

    Ok(())
}

pub async fn handle_category_add_modal(
    ctx: &Context,
    interaction: &serenity::all::ModalInteraction,
    db: &Database,
) -> Result<()> {
    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    let inputs = &interaction.data.components;

    let name = inputs.get(0)
        .and_then(|row| row.components.get(0))
        .and_then(|comp| {
            if let serenity::all::ActionRowComponent::InputText(input) = comp {
                input.value.clone()
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Support".to_string());

    let description = inputs.get(1)
        .and_then(|row| row.components.get(0))
        .and_then(|comp| {
            if let serenity::all::ActionRowComponent::InputText(input) = comp {
                input.value.clone()
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Support tickets".to_string());

    let emoji = inputs.get(2)
        .and_then(|row| row.components.get(0))
        .and_then(|comp| {
            if let serenity::all::ActionRowComponent::InputText(input) = comp {
                input.value.as_ref().and_then(|v| if v.is_empty() { None } else { Some(v.clone()) })
            } else {
                None
            }
        });

    sqlx::query(
        "INSERT INTO ticket_categories (guild_id, name, description, emoji) VALUES ($1, $2, $3, $4)"
    )
    .bind(guild_id.get() as i64)
    .bind(&name)
    .bind(&description)
    .bind(&emoji)
    .execute(&db.pool)
    .await?;

    info!("Created category '{}' for guild {}", name, guild_id.get());

    let mut redis_conn = db.redis.clone();
    let _: () = redis::cmd("SADD")
        .arg(format!("categories:{}", guild_id.get()))
        .arg(&name)
        .query_async(&mut redis_conn)
        .await
        .unwrap_or(());

    let embed = create_success_embed(
        "Category Created",
        format!("Ticket category **{}** has been created successfully.", name)
    );

    interaction
        .create_response(
            &ctx.http,
            serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_category_edit_select(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let selection = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::StringSelect { values } => {
            if values.is_empty() {
                return Ok(());
            }
            &values[0]
        }
        _ => return Ok(()),
    };

    let parts: Vec<&str> = selection.split('_').collect();
    if parts.len() < 3 {
        return Ok(());
    }

    let category_id = parts[2];

    let category: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT name, description, emoji FROM ticket_categories WHERE id = $1"
    )
    .bind(category_id)
    .fetch_optional(&db.pool)
    .await?;

    let (name, description, emoji) = match category {
        Some(cat) => cat,
        None => {
            let embed = create_error_embed("Error", "Category not found");
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let edit_button = serenity::all::CreateButton::new(format!("category_edit_confirm_{}", category_id))
        .label("Edit")
        .style(serenity::all::ButtonStyle::Primary);

    let delete_button = serenity::all::CreateButton::new(format!("category_delete_confirm_{}", category_id))
        .label("Delete")
        .style(serenity::all::ButtonStyle::Danger);

    let embed = create_embed(
        format!("Category: {}", name),
        format!("**Description:** {}\n**Emoji:** {}", description, emoji.unwrap_or("None".to_string()))
    ).color(0x5865F2);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![serenity::all::CreateActionRow::Buttons(vec![edit_button, delete_button])])
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_category_delete_confirm(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let parts: Vec<&str> = interaction.data.custom_id.split('_').collect();
    if parts.len() < 4 {
        return Ok(());
    }

    let category_id = parts[3];
    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    let category_name: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM ticket_categories WHERE id = $1"
    )
    .bind(category_id)
    .fetch_optional(&db.pool)
    .await?;

    if let Some((name,)) = category_name {
        sqlx::query("DELETE FROM ticket_categories WHERE id = $1")
            .bind(category_id)
            .execute(&db.pool)
            .await?;

        info!("Deleted category '{}' ({}) from guild {}", name, category_id, guild_id.get());

        let mut redis_conn = db.redis.clone();
        let _: () = redis::cmd("SREM")
            .arg(format!("categories:{}", guild_id.get()))
            .arg(&name)
            .query_async(&mut redis_conn)
            .await
            .unwrap_or(());

        let embed = create_success_embed(
            "Category Deleted",
            format!("Category **{}** has been deleted.", name)
        );

        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    serenity::all::CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .components(vec![])
                ),
            )
            .await?;
    }

    Ok(())
}

pub async fn handle_panel_category_select(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    let selected_ids = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::StringSelect { values } => {
            if values.is_empty() {
                return Ok(());
            }
            values.clone()
        }
        _ => return Ok(()),
    };

    let placeholders = selected_ids.iter().enumerate()
        .map(|(i, _)| format!("${}::uuid", i + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let query_str = format!(
        "SELECT id::text, name, emoji FROM ticket_categories WHERE id IN ({}) ORDER BY created_at",
        placeholders
    );

    let mut query = sqlx::query_as::<_, (String, String, Option<String>)>(&query_str);
    for id in &selected_ids {
        query = query.bind(id);
    }

    let categories = query.fetch_all(&db.pool).await?;

    let guild = crate::database::ticket::get_or_create_guild(&db.pool, guild_id.get() as i64).await?;

    let embed_title = guild.embed_title.unwrap_or_else(|| "Support Ticket".to_string());
    let embed_description = guild.embed_description.unwrap_or_else(|| "Select a category to create a ticket".to_string());
    let embed_color = guild.embed_color.unwrap_or(5865714);
    let embed_footer = guild.embed_footer;

    let mut panel_embed = create_embed(&embed_title, &embed_description)
        .color(embed_color);

    if let Some(footer) = &embed_footer {
        panel_embed = panel_embed.footer(serenity::all::CreateEmbedFooter::new(footer));
    }

    let category_list = categories.iter()
        .map(|(_, name, emoji)| {
            let emoji_display = emoji.as_ref().map(|e| format!("{} ", e)).unwrap_or_default();
            format!("• {}{}", emoji_display, name)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let response_embed = create_embed(
        "Panel Preview",
        format!(
            "**Selected Categories:**\n{}\n\n\
            Choose how users will select categories:",
            category_list
        )
    ).color(0x5865F2);

    // Store category IDs in Redis with short key to avoid custom_id length limits
    use uuid::Uuid;
    let cache_key = Uuid::new_v4().to_string();
    let categories_json = serde_json::to_string(&selected_ids)?;

    let mut redis_conn = db.redis.clone();
    redis::cmd("SETEX")
        .arg(format!("panel_categories:{}", cache_key))
        .arg(3600) // 1 hour expiry
        .arg(&categories_json)
        .query_async::<()>(&mut redis_conn)
        .await?;

    let button_style_btn = serenity::all::CreateButton::new(format!("panel_style_button_{}", cache_key))
        .label("Button Style")
        .style(serenity::all::ButtonStyle::Primary);

    let dropdown_style_btn = serenity::all::CreateButton::new(format!("panel_style_dropdown_{}", cache_key))
        .label("Dropdown Style")
        .style(serenity::all::ButtonStyle::Secondary);

    let edit_embed_button = serenity::all::CreateButton::new("panel_edit_embed")
        .label("Edit Embed")
        .style(serenity::all::ButtonStyle::Primary);

    let advanced_button = serenity::all::CreateButton::new("panel_edit_advanced")
        .label("Advanced")
        .style(serenity::all::ButtonStyle::Secondary);

    let cancel_button = serenity::all::CreateButton::new("panel_cancel")
        .label("Cancel")
        .style(serenity::all::ButtonStyle::Danger);

    let components = vec![
        serenity::all::CreateActionRow::Buttons(vec![button_style_btn, dropdown_style_btn]),
        serenity::all::CreateActionRow::Buttons(vec![edit_embed_button, advanced_button, cancel_button]),
    ];

    info!("Panel preview with {} categories for guild {}", categories.len(), guild_id.get());

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(response_embed)
                    .embed(panel_embed)
                    .components(components)
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_panel_style_choice(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
    use_buttons: bool,
) -> Result<()> {
    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
    let channel_id = interaction.channel_id;

    let parts: Vec<&str> = interaction.data.custom_id.split('_').collect();
    if parts.len() < 3 {
        return Ok(());
    }

    let cache_key = parts[3];

    // Retrieve category IDs from Redis
    let mut redis_conn = db.redis.clone();
    let categories_json: String = redis::cmd("GET")
        .arg(format!("panel_categories:{}", cache_key))
        .query_async(&mut redis_conn)
        .await?;

    let category_ids: Vec<String> = serde_json::from_str(&categories_json)?;

    // If using buttons, ask for button colors first
    if use_buttons {
        // Parse UUIDs from strings
        use uuid::Uuid;
        let category_uuids: Vec<Uuid> = category_ids.iter()
            .filter_map(|id| Uuid::parse_str(id).ok())
            .collect();

        let placeholders = category_uuids.iter().enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");

        let query_str = format!(
            "SELECT id::text, name, emoji FROM ticket_categories WHERE id IN ({}) ORDER BY created_at",
            placeholders
        );

        let mut query = sqlx::query_as::<_, (String, String, Option<String>)>(&query_str);
        for uuid in &category_uuids {
            query = query.bind(uuid);
        }

        let categories = query.fetch_all(&db.pool).await?;

        // Store button color choices in Redis temporarily
        let color_cache_key = Uuid::new_v4().to_string();

        // Store both category IDs and cache key
        let panel_data = serde_json::json!({
            "categories": category_ids,
            "cache_key": cache_key,
            "color_key": color_cache_key
        });

        redis::cmd("SETEX")
            .arg(format!("panel_color_setup:{}", color_cache_key))
            .arg(3600)
            .arg(panel_data.to_string())
            .query_async::<()>(&mut redis_conn)
            .await?;

        let embed = create_embed(
            "Button Color Selection",
            format!(
                "Choose button colors for your categories:\n\n{}\n\n\
                **Available Colors:**\n\
                🔵 Primary (Blue) - Default Discord style\n\
                ⚪ Secondary (Gray) - Neutral style\n\
                🟢 Success (Green) - Positive action\n\
                🔴 Danger (Red) - Important/urgent",
                categories.iter()
                    .map(|(_, name, emoji)| {
                        let e = emoji.as_ref().map(|s| format!("{} ", s)).unwrap_or_default();
                        format!("• {}{}", e, name)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        ).color(0x5865F2);

        let select_options = vec![
            serenity::all::CreateSelectMenuOption::new("🔵 Primary (Blue)", "primary"),
            serenity::all::CreateSelectMenuOption::new("⚪ Secondary (Gray)", "secondary"),
            serenity::all::CreateSelectMenuOption::new("🟢 Success (Green)", "success"),
            serenity::all::CreateSelectMenuOption::new("🔴 Danger (Red)", "danger"),
        ];

        let select_menu = serenity::all::CreateSelectMenu::new(
            format!("panel_button_color_{}", color_cache_key),
            serenity::all::CreateSelectMenuKind::String {
                options: select_options,
            }
        ).placeholder("Select default button color");

        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .components(vec![serenity::all::CreateActionRow::SelectMenu(select_menu)])
                ),
            )
            .await?;

        return Ok(());
    }

    // For dropdown style, proceed directly to sending panel
    send_panel_with_categories(ctx, interaction, db, guild_id, channel_id, cache_key, category_ids, use_buttons, "primary".to_string()).await
}

async fn send_panel_with_categories(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
    guild_id: serenity::all::GuildId,
    channel_id: serenity::all::ChannelId,
    cache_key: &str,
    category_ids: Vec<String>,
    use_buttons: bool,
    button_style: String,
) -> Result<()> {
    // Parse UUIDs from strings
    use uuid::Uuid;
    let category_uuids: Vec<Uuid> = category_ids.iter()
        .filter_map(|id| Uuid::parse_str(id).ok())
        .collect();

    let placeholders = category_uuids.iter().enumerate()
        .map(|(i, _)| format!("${}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let query_str = format!(
        "SELECT id::text, name, emoji FROM ticket_categories WHERE id IN ({}) ORDER BY created_at",
        placeholders
    );

    let mut query = sqlx::query_as::<_, (String, String, Option<String>)>(&query_str);
    for uuid in &category_uuids {
        query = query.bind(uuid);
    }

    let categories = query.fetch_all(&db.pool).await?;

    let guild = crate::database::ticket::get_or_create_guild(&db.pool, guild_id.get() as i64).await?;

    let embed_title = guild.embed_title.unwrap_or_else(|| "Support Ticket".to_string());
    let embed_description = guild.embed_description.unwrap_or_else(|| "Select a category to create a ticket".to_string());
    let embed_color = guild.embed_color.unwrap_or(5865714);
    let embed_footer = guild.embed_footer;

    let mut panel_embed = create_embed(&embed_title, &embed_description)
        .color(embed_color);

    if let Some(footer) = &embed_footer {
        panel_embed = panel_embed.footer(serenity::all::CreateEmbedFooter::new(footer));
    }

    let panel_msg = if use_buttons {
        // Convert button_style string to ButtonStyle enum
        let style = match button_style.as_str() {
            "primary" => serenity::all::ButtonStyle::Primary,
            "secondary" => serenity::all::ButtonStyle::Secondary,
            "success" => serenity::all::ButtonStyle::Success,
            "danger" => serenity::all::ButtonStyle::Danger,
            _ => serenity::all::ButtonStyle::Primary,
        };

        let buttons: Vec<_> = categories.iter().take(5).map(|(id, name, emoji)| {
            let label = if name.len() > 80 { &name[..77] } else { name };
            let mut button = serenity::all::CreateButton::new(format!("ticket_create_cat_{}", id))
                .label(label)
                .style(style);

            if let Some(e) = emoji {
                if let Ok(emoji_parsed) = e.parse::<serenity::all::ReactionType>() {
                    button = button.emoji(emoji_parsed);
                }
            }
            button
        }).collect();

        let button_rows: Vec<_> = buttons.chunks(5)
            .map(|chunk| serenity::all::CreateActionRow::Buttons(chunk.to_vec()))
            .collect();

        channel_id.send_message(
            &ctx.http,
            serenity::all::CreateMessage::new()
                .embed(panel_embed)
                .components(button_rows)
        ).await?
    } else {
        let select_options: Vec<_> = categories.iter().map(|(id, name, emoji)| {
            let display = if let Some(e) = emoji {
                format!("{} {}", e, name)
            } else {
                name.clone()
            };
            serenity::all::CreateSelectMenuOption::new(display, format!("ticket_create_cat_{}", id))
        }).collect();

        let select_menu = serenity::all::CreateSelectMenu::new(
            "ticket_category_select",
            serenity::all::CreateSelectMenuKind::String {
                options: select_options,
            }
        ).placeholder("🎫 Open a ticket - Select a category");

        channel_id.send_message(
            &ctx.http,
            serenity::all::CreateMessage::new()
                .embed(panel_embed)
                .components(vec![serenity::all::CreateActionRow::SelectMenu(select_menu)])
        ).await?
    };

    sqlx::query(
        "INSERT INTO ticket_panel (guild_id, channel_id, message_id, title, description, selection_type) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(guild_id.get() as i64)
    .bind(channel_id.get() as i64)
    .bind(panel_msg.id.get() as i64)
    .bind(&embed_title)
    .bind(&embed_description)
    .bind(if use_buttons { "button" } else { "dropdown" })
    .execute(&db.pool)
    .await?;

    let panel_id: (String,) = sqlx::query_as(
        "SELECT id::text FROM ticket_panel WHERE message_id = $1"
    )
    .bind(panel_msg.id.get() as i64)
    .fetch_one(&db.pool)
    .await?;

    for (cat_id, name, emoji) in &categories {
        let cat_uuid = Uuid::parse_str(cat_id)?;
        sqlx::query(
            "INSERT INTO panel_categories (panel_id, category_id, button_label, button_emoji, button_style) VALUES ($1::uuid, $2, $3, $4, $5)"
        )
        .bind(&panel_id.0)
        .bind(&cat_uuid)
        .bind(name)
        .bind(emoji)
        .bind(&button_style)
        .execute(&db.pool)
        .await?;
    }

    info!("Panel sent to channel {} in guild {} with {} categories ({})",
          channel_id.get(), guild_id.get(), categories.len(), if use_buttons { "buttons" } else { "dropdown" });

    // Clean up Redis cache key
    let mut redis_conn_cleanup = db.redis.clone();
    let _: () = redis::cmd("DEL")
        .arg(format!("panel_categories:{}", cache_key))
        .query_async(&mut redis_conn_cleanup)
        .await
        .unwrap_or(());

    let mut redis_conn = db.redis.clone();
    let _: () = redis::cmd("SADD")
        .arg(format!("panels:{}", guild_id.get()))
        .arg(panel_msg.id.get())
        .query_async(&mut redis_conn)
        .await
        .unwrap_or(());

    let success_embed = create_success_embed(
        "Panel Sent",
        format!("Panel has been sent to <#{}> with {} {}",
                channel_id,
                categories.len(),
                if categories.len() == 1 { "category" } else { "categories" })
    );

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(success_embed)
                    .components(vec![])
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_panel_button_color_select(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let parts: Vec<&str> = interaction.data.custom_id.split('_').collect();
    let color_cache_key = parts.last().ok_or_else(|| anyhow::anyhow!("Invalid color cache key"))?;

    let button_style = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::StringSelect { values } => {
            values.first().ok_or_else(|| anyhow::anyhow!("No button style selected"))?.clone()
        }
        _ => return Ok(()),
    };

    let mut redis_conn = db.redis.clone();
    let panel_data_str: String = redis::cmd("GET")
        .arg(format!("panel_color_setup:{}", color_cache_key))
        .query_async(&mut redis_conn)
        .await?;

    let panel_data: serde_json::Value = serde_json::from_str(&panel_data_str)?;
    let category_ids: Vec<String> = serde_json::from_value(panel_data["categories"].clone())?;
    let cache_key = panel_data["cache_key"].as_str().ok_or_else(|| anyhow::anyhow!("Missing cache key"))?.to_string();

    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
    let channel_id = interaction.channel_id;

    let mut redis_conn_cleanup = db.redis.clone();
    let _: () = redis::cmd("DEL")
        .arg(format!("panel_color_setup:{}", color_cache_key))
        .query_async(&mut redis_conn_cleanup)
        .await
        .unwrap_or(());

    send_panel_with_categories(ctx, interaction, db, guild_id, channel_id, &cache_key, category_ids, true, button_style).await
}

pub async fn handle_ticket_category_select(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let selected_value = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::StringSelect { values } => {
            if values.is_empty() {
                return Ok(());
            }
            values[0].clone()
        }
        _ => return Ok(()),
    };

    let parts: Vec<&str> = selected_value.split('_').collect();
    if parts.len() < 4 {
        return Ok(());
    }

    let category_id = parts[3];

    // Parse category_id to UUID
    use uuid::Uuid;
    let category_uuid = Uuid::parse_str(category_id)?;

    let user_id = interaction.user.id.get() as i64;
    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    let blacklisted: Option<(bool,)> = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM blacklist WHERE target_id = $1 AND target_type = 'user')"
    )
    .bind(user_id)
    .fetch_optional(&db.pool)
    .await?;

    if let Some((true,)) = blacklisted {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("You are blacklisted from creating tickets")
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    let category: Option<(String, String)> = sqlx::query_as(
        "SELECT name, description FROM ticket_categories WHERE id = $1"
    )
    .bind(&category_uuid)
    .fetch_optional(&db.pool)
    .await?;

    let (category_name, _category_desc) = match category {
        Some(cat) => cat,
        None => {
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Category not found")
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let existing_tickets = crate::database::ticket::get_user_tickets(&db.pool, guild_id.get() as i64, user_id).await?;

    if !existing_tickets.is_empty() {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("You already have an open ticket")
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    let guild_data = crate::database::ticket::get_or_create_guild(&db.pool, guild_id.get() as i64).await?;

    let ticket_category_id = match guild_data.ticket_category_id {
        Some(id) => serenity::all::ChannelId::new(id as u64),
        None => {
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Ticket category not configured")
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let member = guild_id.member(&ctx.http, interaction.user.id).await?;

    // Create temporary channel name - will be updated after ticket creation
    let temp_name = format!("ticket-{}", user_id);
    let mut ticket_channel = guild_id
        .create_channel(
            &ctx.http,
            serenity::all::CreateChannel::new(temp_name)
                .kind(serenity::all::ChannelType::Text)
                .category(ticket_category_id)
                .permissions(vec![
                    serenity::all::PermissionOverwrite {
                        allow: serenity::all::Permissions::VIEW_CHANNEL
                            | serenity::all::Permissions::SEND_MESSAGES
                            | serenity::all::Permissions::READ_MESSAGE_HISTORY,
                        deny: serenity::all::Permissions::empty(),
                        kind: serenity::all::PermissionOverwriteType::Member(member.user.id),
                    },
                    serenity::all::PermissionOverwrite {
                        allow: serenity::all::Permissions::empty(),
                        deny: serenity::all::Permissions::VIEW_CHANNEL,
                        kind: serenity::all::PermissionOverwriteType::Role(serenity::all::RoleId::new(guild_id.get())),
                    },
                ]),
        )
        .await?;

    let ticket = crate::database::ticket::create_ticket(
        &db.pool,
        guild_id.get() as i64,
        ticket_channel.id.get() as i64,
        user_id,
        Some(category_uuid),
    )
    .await?;

    // Rename channel to ticket-userid
    ticket_channel.edit(&ctx.http, serenity::all::EditChannel::new().name(format!("ticket-{}", user_id))).await?;

    info!("Created ticket {} in channel {} for user {} (category: {})", ticket.ticket_number, ticket_channel.id.get(), user_id, category_name);

    let embed = create_embed(
        format!("Ticket #{} - {}", ticket.ticket_number, category_name),
        format!("Welcome <@{}>! Please describe your issue and our support team will be with you shortly.", user_id)
    ).color(0x5865F2);

    let mut buttons = vec![
        serenity::all::CreateButton::new("ticket_close")
            .label("Close Ticket")
            .style(serenity::all::ButtonStyle::Danger),
    ];

    if guild_data.claim_buttons_enabled.unwrap_or(true) {
        buttons.insert(0, serenity::all::CreateButton::new("ticket_claim")
            .label("Claim")
            .style(serenity::all::ButtonStyle::Primary));
    }

    ticket_channel
        .send_message(
            &ctx.http,
            serenity::all::CreateMessage::new()
                .embed(embed)
                .components(vec![serenity::all::CreateActionRow::Buttons(buttons)]),
        )
        .await?;

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("Ticket created: <#{}>", ticket_channel.id))
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_panel_edit_advanced(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let user_id = interaction.user.id.get();
    let session_key = format!("panel_edit_session:{}", user_id);

    let mut redis_conn = db.redis.clone();
    let existing_session: Option<String> = redis::cmd("GET")
        .arg(&session_key)
        .query_async(&mut redis_conn)
        .await
        .ok();

    if existing_session.is_none() {
        let embed = crate::utils::create_error_embed(
            "No Active Session",
            "Please start editing from the panel setup first."
        );

        interaction.create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true)
            )
        ).await?;
        return Ok(());
    }

    let modal = CreateModal::new("panel_advanced_modal", "Advanced Embed Options")
        .components(vec![
            serenity::all::CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Short, "Image URL", "embed_image")
                    .placeholder("https://example.com/image.png")
                    .required(false)
            ),
            serenity::all::CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Short, "Thumbnail URL", "embed_thumbnail")
                    .placeholder("https://example.com/thumb.png")
                    .required(false)
            ),
            serenity::all::CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Short, "Footer Text", "embed_footer_text")
                    .placeholder("Powered by Ticket Bot")
                    .required(false)
            ),
            serenity::all::CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Short, "Footer Icon URL", "embed_footer_icon")
                    .placeholder("https://example.com/icon.png")
                    .required(false)
            ),
        ]);

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Modal(modal),
        )
        .await?;

    Ok(())
}

pub async fn handle_panel_advanced_modal(
    ctx: &Context,
    interaction: &serenity::all::ModalInteraction,
    db: &Database,
) -> Result<()> {
    let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
    let user_id = interaction.user.id.get();
    let session_key = format!("panel_edit_session:{}", user_id);

    let inputs = &interaction.data.components;

    let image_url = inputs.get(0)
        .and_then(|row| row.components.get(0))
        .and_then(|comp| {
            if let serenity::all::ActionRowComponent::InputText(input) = comp {
                input.value.as_ref().and_then(|v| if v.is_empty() { None } else { Some(v.clone()) })
            } else {
                None
            }
        });

    let thumbnail_url = inputs.get(1)
        .and_then(|row| row.components.get(0))
        .and_then(|comp| {
            if let serenity::all::ActionRowComponent::InputText(input) = comp {
                input.value.as_ref().and_then(|v| if v.is_empty() { None } else { Some(v.clone()) })
            } else {
                None
            }
        });

    let footer_text = inputs.get(2)
        .and_then(|row| row.components.get(0))
        .and_then(|comp| {
            if let serenity::all::ActionRowComponent::InputText(input) = comp {
                input.value.as_ref().and_then(|v| if v.is_empty() { None } else { Some(v.clone()) })
            } else {
                None
            }
        });

    let footer_icon = inputs.get(3)
        .and_then(|row| row.components.get(0))
        .and_then(|comp| {
            if let serenity::all::ActionRowComponent::InputText(input) = comp {
                input.value.as_ref().and_then(|v| if v.is_empty() { None } else { Some(v.clone()) })
            } else {
                None
            }
        });

    let mut redis_conn = db.redis.clone();
    let _: () = redis::cmd("DEL")
        .arg(&session_key)
        .query_async(&mut redis_conn)
        .await
        .unwrap_or(());

    let panels: Vec<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM ticket_panel WHERE guild_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(guild_id.get() as i64)
    .fetch_all(&db.pool)
    .await?;

    if let Some((panel_id,)) = panels.first() {
        sqlx::query(
            "UPDATE ticket_panel SET embed_image_url = $1, embed_thumbnail_url = $2, embed_footer_text = $3, embed_footer_icon_url = $4 WHERE id = $5"
        )
        .bind(&image_url)
        .bind(&thumbnail_url)
        .bind(&footer_text)
        .bind(&footer_icon)
        .bind(panel_id)
        .execute(&db.pool)
        .await?;
    }

    let embed = create_success_embed(
        "Advanced Options Saved",
        "Embed image, thumbnail, and footer settings have been saved to your panel."
    );

    interaction
        .create_response(
            &ctx.http,
            serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}
