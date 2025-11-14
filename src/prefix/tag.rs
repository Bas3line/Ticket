use anyhow::Result;
use serenity::all::{Context, Message};
use std::sync::Arc;
use crate::database::Database;
use crate::utils::{create_error_embed, create_success_embed, create_embed};

pub async fn handle(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str]) -> Result<()> {
    let guild_id = msg.guild_id.map(|g| g.get()).unwrap_or(0);
    let prefix = crate::prefix::get_prefix(&db.pool, guild_id).await;

    if args.is_empty() {
        show_doc_page(ctx, msg, &prefix).await?;
        return Ok(());
    }

    let subcommand = args[0];

    match subcommand {
        "create" => create(ctx, msg, db, &args[1..], &prefix).await?,
        "edit" => edit(ctx, msg, db, &args[1..], &prefix).await?,
        "delete" | "remove" => delete(ctx, msg, db, &args[1..], &prefix).await?,
        "info" => info(ctx, msg, db, &args[1..], &prefix).await?,
        "list" => list(ctx, msg, db).await?,
        "search" => search(ctx, msg, db, &args[1..], &prefix).await?,
        "raw" => raw(ctx, msg, db, &args[1..], &prefix).await?,
        "rename" => rename(ctx, msg, db, &args[1..], &prefix).await?,
        "popular" => popular(ctx, msg, db).await?,
        _ => use_tag(ctx, msg, db, args, &prefix).await?,
    }

    Ok(())
}

async fn show_doc_page(ctx: &Context, msg: &Message, prefix: &str) -> Result<()> {
    let embed = create_embed(
        "Tag System Commands",
        format!(
            "**Tag Commands:**\n\n\
            `{}tag <name>` - View a tag\n\
            `{}tag create <name> <content>` - Create a new tag\n\
            `{}tag edit <name> <new content>` - Edit your tag\n\
            `{}tag delete <name>` - Delete your tag\n\
            `{}tag list` - List all tags in this server\n\
            `{}tag search <query>` - Search for tags\n\
            `{}tag info <name>` - View tag information\n\
            `{}tag raw <name>` - View raw tag content\n\
            `{}tag rename <old> <new>` - Rename your tag\n\
            `{}tag popular` - View most popular tags",
            prefix, prefix, prefix, prefix, prefix, prefix, prefix, prefix, prefix, prefix
        )
    ).color(0x5865F2);

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    Ok(())
}

async fn create(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], prefix: &str) -> Result<()> {
    if args.len() < 2 {
        let embed = create_error_embed("Invalid Usage", format!("Use `{}tag create <name> <content>`", prefix));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap().get() as i64;
    let name = args[0];
    let content = args[1..].join(" ");

    if name.len() > 100 {
        let embed = create_error_embed("Invalid Name", "Tag name must be 100 characters or less");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if crate::database::tag::get_tag(&db.pool, guild_id, name).await?.is_some() {
        let embed = create_error_embed("Tag Exists", format!("Tag `{}` already exists", name));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    crate::database::tag::create_tag(&db.pool, guild_id, name, &content, msg.author.id.get() as i64).await?;

    let embed = create_success_embed("Tag Created", format!("Tag `{}` has been created successfully", name));
    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;

    Ok(())
}

async fn edit(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], prefix: &str) -> Result<()> {
    if args.len() < 2 {
        let embed = create_error_embed("Invalid Usage", format!("Use `{}tag edit <name> <new content>`", prefix));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap().get() as i64;
    let name = args[0];
    let content = args[1..].join(" ");

    let tag = crate::database::tag::get_tag(&db.pool, guild_id, name).await?;

    if tag.is_none() {
        let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist", name));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let tag = tag.unwrap();
    if tag.creator_id != msg.author.id.get() as i64 {
        let embed = create_error_embed("Permission Denied", "You can only edit tags you created");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    crate::database::tag::update_tag(&db.pool, guild_id, name, &content).await?;

    let embed = create_success_embed("Tag Updated", format!("Tag `{}` has been updated successfully", name));
    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;

    Ok(())
}

async fn delete(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], prefix: &str) -> Result<()> {
    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", format!("Use `{}tag delete <name>`", prefix));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap().get() as i64;
    let name = args[0];

    let tag = crate::database::tag::get_tag(&db.pool, guild_id, name).await?;

    if tag.is_none() {
        let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist", name));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let tag = tag.unwrap();
    if tag.creator_id != msg.author.id.get() as i64 {
        let embed = create_error_embed("Permission Denied", "You can only delete tags you created");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    crate::database::tag::delete_tag(&db.pool, guild_id, name).await?;

    let embed = create_success_embed("Tag Deleted", format!("Tag `{}` has been deleted", name));
    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;

    Ok(())
}

async fn info(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], prefix: &str) -> Result<()> {
    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", format!("Use `{}tag info <name>`", prefix));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap().get() as i64;
    let name = args[0];

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

        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    } else {
        let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist", name));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    }

    Ok(())
}

async fn list(ctx: &Context, msg: &Message, db: &Arc<Database>) -> Result<()> {
    let guild_id = msg.guild_id.unwrap().get() as i64;
    let tags = crate::database::tag::list_tags(&db.pool, guild_id).await?;

    if tags.is_empty() {
        let embed = create_error_embed("No Tags", "There are no tags in this server");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
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

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;

    Ok(())
}

async fn search(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], prefix: &str) -> Result<()> {
    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", format!("Use `{}tag search <query>`", prefix));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap().get() as i64;
    let query = args.join(" ");

    let tags = crate::database::tag::search_tags(&db.pool, guild_id, &query).await?;

    if tags.is_empty() {
        let embed = create_error_embed("No Results", format!("No tags found matching `{}`", query));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let tag_list: Vec<String> = tags.iter()
        .map(|t| format!("`{}` (uses: {})", t.name, t.uses))
        .collect();

    let embed = create_embed(
        format!("Search Results ({})", tags.len()),
        tag_list.join("\n")
    );

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;

    Ok(())
}

async fn raw(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], prefix: &str) -> Result<()> {
    if args.is_empty() {
        let embed = create_error_embed("Invalid Usage", format!("Use `{}tag raw <name>`", prefix));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap().get() as i64;
    let name = args[0];

    let tag = crate::database::tag::get_tag(&db.pool, guild_id, name).await?;

    if let Some(tag) = tag {
        let content = format!("```\n{}\n```", tag.content.replace("```", "\\`\\`\\`"));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().content(content)).await?;
    } else {
        let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist", name));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    }

    Ok(())
}

async fn rename(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], prefix: &str) -> Result<()> {
    if args.len() < 2 {
        let embed = create_error_embed("Invalid Usage", format!("Use `{}tag rename <old_name> <new_name>`", prefix));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap().get() as i64;
    let old_name = args[0];
    let new_name = args[1];

    if new_name.len() > 100 {
        let embed = create_error_embed("Invalid Name", "Tag name must be 100 characters or less");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let tag = crate::database::tag::get_tag(&db.pool, guild_id, old_name).await?;

    if tag.is_none() {
        let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist", old_name));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let tag = tag.unwrap();
    if tag.creator_id != msg.author.id.get() as i64 {
        let embed = create_error_embed("Permission Denied", "You can only rename tags you created");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    if crate::database::tag::get_tag(&db.pool, guild_id, new_name).await?.is_some() {
        let embed = create_error_embed("Tag Exists", format!("Tag `{}` already exists", new_name));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    crate::database::tag::rename_tag(&db.pool, guild_id, old_name, new_name).await?;

    let embed = create_success_embed("Tag Renamed", format!("Tag `{}` has been renamed to `{}`", old_name, new_name));
    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;

    Ok(())
}

async fn popular(ctx: &Context, msg: &Message, db: &Arc<Database>) -> Result<()> {
    let guild_id = msg.guild_id.unwrap().get() as i64;
    let tags = crate::database::tag::get_popular_tags(&db.pool, guild_id, 10).await?;

    if tags.is_empty() {
        let embed = create_error_embed("No Tags", "There are no tags in this server");
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let tag_list: Vec<String> = tags.iter()
        .map(|t| format!("`{}` - {} uses", t.name, t.uses))
        .collect();

    let embed = create_embed("Most Popular Tags", tag_list.join("\n"));

    msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;

    Ok(())
}

async fn use_tag(ctx: &Context, msg: &Message, db: &Arc<Database>, args: &[&str], prefix: &str) -> Result<()> {
    let guild_id = msg.guild_id.unwrap().get() as i64;
    let name = args.join(" ");

    let tag = crate::database::tag::get_tag(&db.pool, guild_id, &name).await?;

    if let Some(tag) = tag {
        crate::database::tag::increment_tag_uses(&db.pool, guild_id, &name).await?;

        let embed = create_embed(&tag.name, &tag.content);

        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    } else {
        let embed = create_error_embed("Tag Not Found", format!("Tag `{}` does not exist. Use `{}tag list` to see all tags.", name, prefix));
        msg.channel_id.send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed)).await?;
    }

    Ok(())
}
