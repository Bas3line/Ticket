use serenity::all::{CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue};
use crate::database::Database;
use crate::utils::{create_error_embed, create_success_embed, create_embed};
use anyhow::Result;

pub async fn run(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let options = &interaction.data.options();

    if let Some(ResolvedOption { value, name, .. }) = options.first() {
        match *name {
            "create" => handle_create(ctx, interaction, db, value).await?,
            "edit" => handle_edit(ctx, interaction, db, value).await?,
            "delete" => handle_delete(ctx, interaction, db, value).await?,
            "info" => handle_info(ctx, interaction, db, value).await?,
            "list" => handle_list(ctx, interaction, db).await?,
            "search" => handle_search(ctx, interaction, db, value).await?,
            "raw" => handle_raw(ctx, interaction, db, value).await?,
            "rename" => handle_rename(ctx, interaction, db, value).await?,
            "popular" => handle_popular(ctx, interaction, db).await?,
            "use" => handle_use(ctx, interaction, db, value).await?,
            _ => {}
        }
    }

    Ok(())
}

async fn handle_create(ctx: &Context, interaction: &CommandInteraction, db: &Database, options: &ResolvedValue<'_>) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;

    if let ResolvedValue::SubCommand(options) = options {
        let name = options.iter()
            .find(|opt| opt.name == "name")
            .and_then(|opt| if let ResolvedValue::String(s) = opt.value { Some(s) } else { None })
            .unwrap();

        let content = options.iter()
            .find(|opt| opt.name == "content")
            .and_then(|opt| if let ResolvedValue::String(s) = opt.value { Some(s) } else { None })
            .unwrap();

        if name.len() > 100 {
            let embed = create_error_embed("Invalid Name", "Tag name must be 100 characters or less");
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
            return Ok(());
        }

        if crate::database::tag::get_tag(&db.pool, guild_id, name).await?.is_some() {
            let embed = create_error_embed("Tag Exists", format!("Tag `{}` already exists", name));
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
            return Ok(());
        }

        crate::database::tag::create_tag(&db.pool, guild_id, name, content, interaction.user.id.get() as i64).await?;

        let embed = create_success_embed("Tag Created", format!("Tag `{}` has been created successfully", name));
        interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
            serenity::all::CreateInteractionResponseMessage::new().embed(embed)
        )).await?;
    }

    Ok(())
}

async fn handle_edit(ctx: &Context, interaction: &CommandInteraction, db: &Database, options: &ResolvedValue<'_>) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;

    if let ResolvedValue::SubCommand(options) = options {
        let name = options.iter()
            .find(|opt| opt.name == "name")
            .and_then(|opt| if let ResolvedValue::String(s) = opt.value { Some(s) } else { None })
            .unwrap();

        let content = options.iter()
            .find(|opt| opt.name == "content")
            .and_then(|opt| if let ResolvedValue::String(s) = opt.value { Some(s) } else { None })
            .unwrap();

        let tag = crate::database::tag::get_tag(&db.pool, guild_id, name).await?;

        if tag.is_none() {
            let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist", name));
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
            return Ok(());
        }

        let tag = tag.unwrap();
        if tag.creator_id != interaction.user.id.get() as i64 {
            let embed = create_error_embed("Permission Denied", "You can only edit tags you created");
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
            return Ok(());
        }

        crate::database::tag::update_tag(&db.pool, guild_id, name, content).await?;

        let embed = create_success_embed("Tag Updated", format!("Tag `{}` has been updated successfully", name));
        interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
            serenity::all::CreateInteractionResponseMessage::new().embed(embed)
        )).await?;
    }

    Ok(())
}

async fn handle_delete(ctx: &Context, interaction: &CommandInteraction, db: &Database, options: &ResolvedValue<'_>) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;

    if let ResolvedValue::SubCommand(options) = options {
        let name = options.iter()
            .find(|opt| opt.name == "name")
            .and_then(|opt| if let ResolvedValue::String(s) = opt.value { Some(s) } else { None })
            .unwrap();

        let tag = crate::database::tag::get_tag(&db.pool, guild_id, name).await?;

        if tag.is_none() {
            let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist", name));
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
            return Ok(());
        }

        let tag = tag.unwrap();
        if tag.creator_id != interaction.user.id.get() as i64 {
            let embed = create_error_embed("Permission Denied", "You can only delete tags you created");
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
            return Ok(());
        }

        crate::database::tag::delete_tag(&db.pool, guild_id, name).await?;

        let embed = create_success_embed("Tag Deleted", format!("Tag `{}` has been deleted", name));
        interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
            serenity::all::CreateInteractionResponseMessage::new().embed(embed)
        )).await?;
    }

    Ok(())
}

async fn handle_info(ctx: &Context, interaction: &CommandInteraction, db: &Database, options: &ResolvedValue<'_>) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;

    if let ResolvedValue::SubCommand(options) = options {
        let name = options.iter()
            .find(|opt| opt.name == "name")
            .and_then(|opt| if let ResolvedValue::String(s) = opt.value { Some(s) } else { None })
            .unwrap();

        let tag = crate::database::tag::get_tag(&db.pool, guild_id, name).await?;

        if let Some(tag) = tag {
            let embed = create_embed(
                format!("Tag: {}", tag.name),
                format!(
                    "**Creator:** <@{}>\n**Uses:** {}\n**Created:** <t:{}:R>\n**Updated:** <t:{}:R>",
                    tag.creator_id,
                    tag.uses,
                    tag.created_at.timestamp(),
                    tag.updated_at.timestamp()
                )
            );

            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed)
            )).await?;
        } else {
            let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist", name));
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
        }
    }

    Ok(())
}

async fn handle_list(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;
    let tags = crate::database::tag::list_tags(&db.pool, guild_id).await?;

    if tags.is_empty() {
        let embed = create_error_embed("No Tags", "There are no tags in this server");
        interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
            serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
        )).await?;
        return Ok(());
    }

    let tag_list: Vec<String> = tags.iter()
        .take(25)
        .map(|t| format!("`{}` (uses: {})", t.name, t.uses))
        .collect();

    let embed = create_embed(
        format!("Tags ({} total)", tags.len()),
        tag_list.join("\n")
    );

    interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
        serenity::all::CreateInteractionResponseMessage::new().embed(embed)
    )).await?;

    Ok(())
}

async fn handle_search(ctx: &Context, interaction: &CommandInteraction, db: &Database, options: &ResolvedValue<'_>) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;

    if let ResolvedValue::SubCommand(options) = options {
        let query = options.iter()
            .find(|opt| opt.name == "query")
            .and_then(|opt| if let ResolvedValue::String(s) = opt.value { Some(s) } else { None })
            .unwrap();

        let tags = crate::database::tag::search_tags(&db.pool, guild_id, query).await?;

        if tags.is_empty() {
            let embed = create_error_embed("No Results", format!("No tags found matching `{}`", query));
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
            return Ok(());
        }

        let tag_list: Vec<String> = tags.iter()
            .map(|t| format!("`{}` (uses: {})", t.name, t.uses))
            .collect();

        let embed = create_embed(
            format!("Search Results ({})", tags.len()),
            tag_list.join("\n")
        );

        interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
            serenity::all::CreateInteractionResponseMessage::new().embed(embed)
        )).await?;
    }

    Ok(())
}

async fn handle_raw(ctx: &Context, interaction: &CommandInteraction, db: &Database, options: &ResolvedValue<'_>) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;

    if let ResolvedValue::SubCommand(options) = options {
        let name = options.iter()
            .find(|opt| opt.name == "name")
            .and_then(|opt| if let ResolvedValue::String(s) = opt.value { Some(s) } else { None })
            .unwrap();

        let tag = crate::database::tag::get_tag(&db.pool, guild_id, name).await?;

        if let Some(tag) = tag {
            let content = format!("```\n{}\n```", tag.content.replace("```", "\\`\\`\\`"));

            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().content(content)
            )).await?;
        } else {
            let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist", name));
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
        }
    }

    Ok(())
}

async fn handle_rename(ctx: &Context, interaction: &CommandInteraction, db: &Database, options: &ResolvedValue<'_>) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;

    if let ResolvedValue::SubCommand(options) = options {
        let old_name = options.iter()
            .find(|opt| opt.name == "old_name")
            .and_then(|opt| if let ResolvedValue::String(s) = opt.value { Some(s) } else { None })
            .unwrap();

        let new_name = options.iter()
            .find(|opt| opt.name == "new_name")
            .and_then(|opt| if let ResolvedValue::String(s) = opt.value { Some(s) } else { None })
            .unwrap();

        if new_name.len() > 100 {
            let embed = create_error_embed("Invalid Name", "Tag name must be 100 characters or less");
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
            return Ok(());
        }

        let tag = crate::database::tag::get_tag(&db.pool, guild_id, old_name).await?;

        if tag.is_none() {
            let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist", old_name));
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
            return Ok(());
        }

        let tag = tag.unwrap();
        if tag.creator_id != interaction.user.id.get() as i64 {
            let embed = create_error_embed("Permission Denied", "You can only rename tags you created");
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
            return Ok(());
        }

        if crate::database::tag::get_tag(&db.pool, guild_id, new_name).await?.is_some() {
            let embed = create_error_embed("Tag Exists", format!("Tag `{}` already exists", new_name));
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
            return Ok(());
        }

        crate::database::tag::rename_tag(&db.pool, guild_id, old_name, new_name).await?;

        let embed = create_success_embed("Tag Renamed", format!("Tag `{}` has been renamed to `{}`", old_name, new_name));
        interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
            serenity::all::CreateInteractionResponseMessage::new().embed(embed)
        )).await?;
    }

    Ok(())
}

async fn handle_popular(ctx: &Context, interaction: &CommandInteraction, db: &Database) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;
    let tags = crate::database::tag::get_popular_tags(&db.pool, guild_id, 10).await?;

    if tags.is_empty() {
        let embed = create_error_embed("No Tags", "There are no tags in this server");
        interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
            serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
        )).await?;
        return Ok(());
    }

    let tag_list: Vec<String> = tags.iter()
        .map(|t| format!("`{}` - {} uses", t.name, t.uses))
        .collect();

    let embed = create_embed("Most Popular Tags", tag_list.join("\n"));

    interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
        serenity::all::CreateInteractionResponseMessage::new().embed(embed)
    )).await?;

    Ok(())
}

async fn handle_use(ctx: &Context, interaction: &CommandInteraction, db: &Database, options: &ResolvedValue<'_>) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;

    if let ResolvedValue::SubCommand(options) = options {
        let name = options.iter()
            .find(|opt| opt.name == "name")
            .and_then(|opt| if let ResolvedValue::String(s) = opt.value { Some(s) } else { None })
            .unwrap();

        let tag = crate::database::tag::get_tag(&db.pool, guild_id, name).await?;

        if let Some(tag) = tag {
            crate::database::tag::increment_tag_uses(&db.pool, guild_id, name).await?;

            let embed = create_embed(&tag.name, &tag.content);

            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed)
            )).await?;
        } else {
            let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist", name));
            interaction.create_response(&ctx.http, serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await?;
        }
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("tag")
        .description("Manage server tags")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "create", "Create a new tag")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Tag name").required(true))
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "content", "Tag content").required(true))
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "edit", "Edit an existing tag")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Tag name").required(true))
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "content", "New content").required(true))
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "delete", "Delete a tag")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Tag name").required(true))
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "info", "Get information about a tag")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Tag name").required(true))
        )
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "list", "List all tags"))
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "search", "Search for tags")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "query", "Search query").required(true))
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "raw", "Get raw tag content")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Tag name").required(true))
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "rename", "Rename a tag")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "old_name", "Current name").required(true))
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "new_name", "New name").required(true))
        )
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "popular", "Show most popular tags"))
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "use", "Use a tag")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Tag name").required(true))
        )
}
