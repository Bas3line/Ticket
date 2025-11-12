use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateMessage, ResolvedValue,
    CreateActionRow, CreateButton, ButtonStyle,
};
use crate::database::Database;
use crate::utils::create_success_embed;
use anyhow::Result;

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

    let guild_icon = interaction
        .guild_id
        .and_then(|gid| ctx.cache.guild(gid))
        .and_then(|g| g.icon_url());

    let mut embed = CreateEmbed::new()
        .title(&title)
        .description(&description)
        .color(serenity::all::Colour::from_rgb(88, 101, 242));

    if let Some(icon_url) = guild_icon {
        embed = embed.thumbnail(icon_url);
    }

    let button = CreateButton::new("ticket_create")
        .label("Create Ticket")
        .style(ButtonStyle::Primary);

    let components = vec![CreateActionRow::Buttons(vec![button])];

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
        .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
            serenity::all::CreateInteractionResponseMessage::new()
                .embed(response_embed)
                .ephemeral(true)
        ))
        .await?;

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
}
