pub mod help;
pub mod admin;
pub mod ticket;
pub mod setup;
pub mod owner;
pub mod settings;
pub mod utility;
pub mod roast;
pub mod toast;

use anyhow::Result;
use serenity::all::{Context, Message};
use std::sync::Arc;
use crate::database::Database;

pub async fn handle_prefix_command(
    ctx: &Context,
    msg: &Message,
    db: &Arc<Database>,
    prefix: &str,
    owner_id: u64,
) -> Result<()> {
    let content = msg.content.strip_prefix(prefix).unwrap_or(&msg.content);
    let parts: Vec<&str> = content.split_whitespace().collect();

    if parts.is_empty() {
        return Ok(());
    }

    let command = parts[0].to_lowercase();
    let args = &parts[1..];

    match command.as_str() {
        "help" => help::execute(ctx, msg, db).await,
        "ping" => utility::ping(ctx, msg, db).await,
        "roast" => roast::execute(ctx, msg, db, owner_id).await,
        "toast" => toast::execute(ctx, msg, db, args).await,
        "setup" | "panel" => setup::execute(ctx, msg, db, args).await,
        "settings" | "config" => settings::settings(ctx, msg, db, args).await,
        "prefix" => setup::set_prefix(ctx, msg, db, args).await,
        "supportrole" | "sr" => admin::supportrole(ctx, msg, db, args).await,
        "category" | "cat" => admin::category(ctx, msg, db, args).await,
        "priority" => admin::priority(ctx, msg, db, args).await,
        "blacklist" | "bl" => admin::blacklist(ctx, msg, db, args).await,
        "note" => admin::note(ctx, msg, db, args).await,
        "stats" => admin::stats(ctx, msg, db).await,
        "close" => ticket::close(ctx, msg, db).await,
        "claim" => ticket::claim(ctx, msg, db).await,
        "transcript" | "trans" => ticket::transcript(ctx, msg, db).await,
        "profile" => owner::profile(ctx, msg, db).await,
        "botstats" => owner::stats(ctx, msg, db, owner_id).await,
        "addprem" => owner::add_premium(ctx, msg, db, args, owner_id).await,
        "removeprem" => owner::remove_premium(ctx, msg, db, args, owner_id).await,
        "listprem" => owner::list_premium(ctx, msg, db, owner_id).await,
        "blacklistuser" => owner::blacklist_user(ctx, msg, db, args, owner_id).await,
        "blacklistguild" => owner::blacklist_guild(ctx, msg, db, args, owner_id).await,
        "unblacklistuser" => owner::unblacklist_user(ctx, msg, db, args, owner_id).await,
        "unblacklistguild" => owner::unblacklist_guild(ctx, msg, db, args, owner_id).await,
        "listblacklist" => owner::list_blacklist(ctx, msg, db, owner_id).await,
        _ => Ok(()),
    }
}

pub async fn get_prefix(pool: &sqlx::PgPool, guild_id: u64) -> String {
    crate::database::ticket::get_guild_prefix(pool, guild_id as i64)
        .await
        .unwrap_or_else(|_| "!".to_string())
}
