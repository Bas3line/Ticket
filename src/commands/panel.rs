use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateMessage, ResolvedValue,
    CreateActionRow, CreateButton, CreateSelectMenu, CreateSelectMenuOption, CreateSelectMenuKind,
    ButtonStyle, ComponentInteractionDataKind,
};
use crate::database::Database;
use crate::utils::create_success_embed;
use anyhow::Result;
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose};

pub async fn run(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let options = &interaction.data.options();

    let title = options
        .iter()
        .find(|o| o.name == "title")
        .and_then(|o| {
            if let ResolvedValue::String(s) = o.value {
                Some(s.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Open a Ticket".to_string());

    let description = options
        .iter()
        .find(|o| o.name == "description")
        .and_then(|o| {
            if let ResolvedValue::String(s) = o.value {
                Some(s.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Click the button below to create a new support ticket".to_string());

    let thumbnail_url = options
        .iter()
        .find(|o| o.name == "thumbnail")
        .and_then(|o| {
            if let ResolvedValue::String(s) = o.value {
                Some(s.to_string())
            } else {
                None
            }
        });

    let image_url = options
        .iter()
        .find(|o| o.name == "image")
        .and_then(|o| {
            if let ResolvedValue::String(s) = o.value {
                Some(s.to_string())
            } else {
                None
            }
        });

    let channel = interaction.channel_id;
    let guild_id = interaction.guild_id.unwrap().get() as i64;

    let is_premium = crate::database::ticket::is_premium(&db.pool, guild_id).await?;
    let panel_count = crate::database::ticket::get_panel_count(&db.pool, guild_id).await?;

    if !is_premium && panel_count >= 1 {
        let embed = crate::utils::create_error_embed(
            "Premium Required",
            "Non-premium servers are limited to 1 panel. Upgrade to premium to create up to 30 panels!",
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

    if is_premium && panel_count >= 30 {
        let embed = crate::utils::create_error_embed(
            "Panel Limit Reached",
            "You have reached the maximum limit of 30 panels for premium servers",
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

    let categories: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT id::text, name, emoji FROM ticket_categories WHERE guild_id = $1 ORDER BY created_at"
    )
    .bind(guild_id)
    .fetch_all(&db.pool)
    .await?;

    if categories.is_empty() {
        let embed = crate::utils::create_error_embed(
            "No Categories",
            "You need to create ticket categories first using `/setup category`",
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

    let embed = CreateEmbed::new()
        .title("Panel Button Color Configuration")
        .description("Do you want to use the same color for all category buttons or customize each one?")
        .color(serenity::all::Colour::from_rgb(88, 101, 242));

    let select_menu = CreateSelectMenu::new(
        format!("panel_color_type:{}:{}:{}:{}",
            general_purpose::STANDARD.encode(&title),
            general_purpose::STANDARD.encode(&description),
            general_purpose::STANDARD.encode(thumbnail_url.as_deref().unwrap_or("")),
            general_purpose::STANDARD.encode(image_url.as_deref().unwrap_or(""))
        ),
        CreateSelectMenuKind::String {
            options: vec![
                CreateSelectMenuOption::new("Use Same Color", "same")
                    .description("All buttons will have the same color"),
                CreateSelectMenuOption::new("Custom Colors", "custom")
                    .description("Choose different colors for each category"),
            ]
        }
    )
    .placeholder("Select button color configuration");

    let components = vec![CreateActionRow::SelectMenu(select_menu)];

    interaction
        .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
            serenity::all::CreateInteractionResponseMessage::new()
                .embed(embed)
                .components(components)
                .ephemeral(true)
        ))
        .await?;

    Ok(())
}

pub async fn handle_color_type_selection(
    ctx: &Context,
    interaction: &serenity::all::ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let data = &interaction.data;
    let custom_id_parts: Vec<&str> = data.custom_id.split(':').collect();

    if custom_id_parts.len() < 5 {
        return Ok(());
    }

    let title = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[1])?)?;
    let description = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[2])?)?;
    let thumbnail = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[3])?)?;
    let image = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[4])?)?;

    let thumbnail_url = if thumbnail.is_empty() { None } else { Some(thumbnail) };
    let image_url = if image.is_empty() { None } else { Some(image) };

    let guild_id = interaction.guild_id.unwrap().get() as i64;

    let categories: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT id::text, name, emoji FROM ticket_categories WHERE guild_id = $1 ORDER BY created_at"
    )
    .bind(guild_id)
    .fetch_all(&db.pool)
    .await?;

    if let ComponentInteractionDataKind::StringSelect { values } = &data.kind {
        let selection = &values[0];

        if selection == "same" {
            let embed = CreateEmbed::new()
                .title("Select Button Color")
                .description("Choose the color for all category buttons")
                .color(serenity::all::Colour::from_rgb(88, 101, 242));

            let select_menu = CreateSelectMenu::new(
                format!("panel_same_color:{}:{}:{}:{}",
                    general_purpose::STANDARD.encode(&title),
                    general_purpose::STANDARD.encode(&description),
                    general_purpose::STANDARD.encode(thumbnail_url.as_deref().unwrap_or("")),
                    general_purpose::STANDARD.encode(image_url.as_deref().unwrap_or(""))
                ),
                CreateSelectMenuKind::String {
                    options: vec![
                        CreateSelectMenuOption::new("Primary (Blurple)", "primary")
                            .description("Discord's signature blurple color"),
                        CreateSelectMenuOption::new("Secondary (Gray)", "secondary")
                            .description("Subtle gray color"),
                        CreateSelectMenuOption::new("Success (Green)", "success")
                            .description("Positive green color"),
                        CreateSelectMenuOption::new("Danger (Red)", "danger")
                            .description("Urgent red color"),
                    ]
                }
            )
            .placeholder("Select button color");

            let components = vec![CreateActionRow::SelectMenu(select_menu)];

            interaction
                .create_response(&ctx.http, serenity::all::CreateInteractionResponse::UpdateMessage(
                    serenity::all::CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .components(components)
                ))
                .await?;
        } else {
            let mut embed = CreateEmbed::new()
                .title("Customize Category Button Colors")
                .description("Select colors for each category button below")
                .color(serenity::all::Colour::from_rgb(88, 101, 242));

            for (_, name, emoji) in &categories {
                let display_name = if let Some(e) = emoji {
                    format!("{} {}", e, name)
                } else {
                    name.clone()
                };
                embed = embed.field(&display_name, "Not configured yet", true);
            }

            let mut components = vec![];
            for (i, (_cat_id, name, emoji)) in categories.iter().enumerate() {
                let display_name = if let Some(e) = emoji {
                    format!("{} {}", e, name)
                } else {
                    name.clone()
                };

                let select_menu = CreateSelectMenu::new(
                    format!("panel_custom_color:{}:{}:{}:{}:{}",
                        i,
                        general_purpose::STANDARD.encode(&title),
                        general_purpose::STANDARD.encode(&description),
                        general_purpose::STANDARD.encode(thumbnail_url.as_deref().unwrap_or("")),
                        general_purpose::STANDARD.encode(image_url.as_deref().unwrap_or(""))
                    ),
                    CreateSelectMenuKind::String {
                        options: vec![
                            CreateSelectMenuOption::new("Primary (Blurple)", "primary"),
                            CreateSelectMenuOption::new("Secondary (Gray)", "secondary"),
                            CreateSelectMenuOption::new("Success (Green)", "success"),
                            CreateSelectMenuOption::new("Danger (Red)", "danger"),
                        ]
                    }
                )
                .placeholder(&display_name);

                components.push(CreateActionRow::SelectMenu(select_menu));

                if components.len() >= 5 {
                    break;
                }
            }

            let finish_button = CreateButton::new(
                format!("panel_finish_custom:{}:{}:{}:{}",
                    general_purpose::STANDARD.encode(&title),
                    general_purpose::STANDARD.encode(&description),
                    general_purpose::STANDARD.encode(thumbnail_url.as_deref().unwrap_or("")),
                    general_purpose::STANDARD.encode(image_url.as_deref().unwrap_or(""))
                )
            )
            .label("Create Panel")
            .style(ButtonStyle::Success);

            components.push(CreateActionRow::Buttons(vec![finish_button]));

            interaction
                .create_response(&ctx.http, serenity::all::CreateInteractionResponse::UpdateMessage(
                    serenity::all::CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .components(components)
                ))
                .await?;
        }
    }

    Ok(())
}

pub async fn handle_same_color_selection(
    ctx: &Context,
    interaction: &serenity::all::ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let data = &interaction.data;
    let custom_id_parts: Vec<&str> = data.custom_id.split(':').collect();

    if custom_id_parts.len() < 5 {
        return Ok(());
    }

    let title = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[1])?)?;
    let description = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[2])?)?;
    let thumbnail = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[3])?)?;
    let image = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[4])?)?;

    let thumbnail_url = if thumbnail.is_empty() { None } else { Some(thumbnail) };
    let image_url = if image.is_empty() { None } else { Some(image) };

    if let ComponentInteractionDataKind::StringSelect { values } = &data.kind {
        let button_color = &values[0];

        create_panel(
            ctx,
            interaction,
            db,
            title,
            description,
            thumbnail_url,
            image_url,
            button_color.to_string(),
            None,
        ).await?;
    }

    Ok(())
}

async fn create_panel(
    ctx: &Context,
    interaction: &serenity::all::ComponentInteraction,
    db: &Database,
    title: String,
    description: String,
    thumbnail_url: Option<String>,
    image_url: Option<String>,
    default_color: String,
    custom_colors: Option<HashMap<usize, String>>,
) -> Result<()> {
    let channel = interaction.channel_id;
    let guild_id = interaction.guild_id.unwrap().get() as i64;

    let categories: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT id::text, name, emoji FROM ticket_categories WHERE guild_id = $1 ORDER BY created_at"
    )
    .bind(guild_id)
    .fetch_all(&db.pool)
    .await?;

    let guild_icon = interaction
        .guild_id
        .and_then(|gid| ctx.cache.guild(gid))
        .and_then(|g| g.icon_url());

    let mut embed = CreateEmbed::new()
        .title(&title)
        .description(&description)
        .color(serenity::all::Colour::from_rgb(88, 101, 242));

    if let Some(thumb_url) = &thumbnail_url {
        embed = embed.thumbnail(thumb_url);
    } else if let Some(icon_url) = guild_icon {
        embed = embed.thumbnail(icon_url);
    }

    if let Some(img_url) = &image_url {
        embed = embed.image(img_url);
    }

    let mut buttons = vec![];
    for (i, (cat_id, name, emoji)) in categories.iter().enumerate() {
        let color = if let Some(ref colors) = custom_colors {
            colors.get(&i).unwrap_or(&default_color)
        } else {
            &default_color
        };

        let button_style = match color.as_str() {
            "primary" | "blurple" => ButtonStyle::Primary,
            "secondary" | "gray" | "grey" => ButtonStyle::Secondary,
            "success" | "green" => ButtonStyle::Success,
            "danger" | "red" => ButtonStyle::Danger,
            _ => ButtonStyle::Primary,
        };

        let mut button = CreateButton::new(format!("ticket_create:{}", cat_id))
            .label(name)
            .style(button_style);

        if let Some(e) = emoji {
            button = button.emoji(serenity::all::ReactionType::Unicode(e.clone()));
        }

        buttons.push(button);

        if buttons.len() >= 5 {
            break;
        }
    }

    let components = vec![CreateActionRow::Buttons(buttons)];

    let msg = channel
        .send_message(&ctx.http, CreateMessage::new().embed(embed).components(components))
        .await?;

    crate::database::ticket::create_ticket_panel(
        &db.pool,
        guild_id,
        channel.get() as i64,
        msg.id.get() as i64,
        title,
        Some(description),
    )
    .await?;

    let response_embed = create_success_embed(
        "Panel Created",
        "Ticket panel has been created successfully",
    );

    interaction
        .create_response(&ctx.http, serenity::all::CreateInteractionResponse::UpdateMessage(
            serenity::all::CreateInteractionResponseMessage::new()
                .embed(response_embed)
                .components(vec![])
        ))
        .await?;

    Ok(())
}

pub async fn handle_custom_color_selection(
    _ctx: &Context,
    _interaction: &serenity::all::ComponentInteraction,
    _db: &Database,
) -> Result<()> {
    Ok(())
}

pub async fn handle_finish_custom(
    ctx: &Context,
    interaction: &serenity::all::ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let data = &interaction.data;
    let custom_id_parts: Vec<&str> = data.custom_id.split(':').collect();

    if custom_id_parts.len() < 5 {
        return Ok(());
    }

    let title = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[1])?)?;
    let description = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[2])?)?;
    let thumbnail = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[3])?)?;
    let image = String::from_utf8(general_purpose::STANDARD.decode(custom_id_parts[4])?)?;

    let thumbnail_url = if thumbnail.is_empty() { None } else { Some(thumbnail) };
    let image_url = if image.is_empty() { None } else { Some(image) };

    create_panel(
        ctx,
        interaction,
        db,
        title,
        description,
        thumbnail_url,
        image_url,
        "primary".to_string(),
        None,
    ).await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("panel")
        .description("Create a ticket panel")
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "title",
            "Panel title",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "description",
            "Panel description",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "thumbnail",
            "Embed thumbnail URL",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "image",
            "Embed image URL",
        ))
}
