pub mod transcript;

use serenity::all::{Colour, CreateEmbed, Context, ChannelId};
use anyhow::Result;

pub fn create_embed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Colour::from_rgb(88, 101, 242))
}

pub fn create_error_embed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Colour::from_rgb(237, 66, 69))
}

pub fn create_success_embed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Colour::from_rgb(87, 242, 135))
}

pub async fn send_log(
    ctx: &Context,
    log_channel_id: Option<i64>,
    embed: CreateEmbed,
) -> Result<()> {
    if let Some(channel_id) = log_channel_id {
        let channel = ChannelId::new(channel_id as u64);
        let _ = channel.send_message(
            &ctx.http,
            serenity::all::CreateMessage::new().embed(embed)
        ).await;
    }
    Ok(())
}
