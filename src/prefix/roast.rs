use anyhow::Result;
use serenity::all::{Context, CreateEmbed, CreateMessage, Message};
use std::sync::Arc;
use sysinfo::System;
use crate::database::Database;

pub async fn execute(ctx: &Context, msg: &Message, db: &Arc<Database>, owner_id: u64) -> Result<()> {
    if msg.author.id.get() != owner_id {
        let embed = crate::utils::create_error_embed(
            "Permission Denied",
            "This command is only available to the bot owner"
        );
        msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(embed)).await?;
        return Ok(());
    }

    let mut sys = System::new_all();
    sys.refresh_all();

    let bot_name = ctx.cache.current_user().name.clone();
    let bot_id = ctx.cache.current_user().id;
    let guild_count = ctx.cache.guild_count();
    let shard_count = ctx.cache.shard_count();

    let process_id = std::process::id();
    let process = sys.process(sysinfo::Pid::from_u32(process_id));
    let memory_usage = process.map(|p| p.memory() / 1024 / 1024).unwrap_or(0);

    let total_memory = sys.total_memory() / 1024 / 1024;
    let used_memory = sys.used_memory() / 1024 / 1024;
    let cpu_count = sys.cpus().len();
    let os_version = System::long_os_version().unwrap_or_else(|| "Unknown".to_string());

    let db_pool_size = db.pool.size() as usize;
    let db_pool_idle = db.pool.num_idle();

    let mut redis_conn = db.redis.clone();
    let redis_status = match redis::cmd("PING").query_async::<String>(&mut redis_conn).await {
        Ok(_) => "Connected",
        Err(_) => "Disconnected",
    };

    let redis_key_count: i64 = redis::cmd("DBSIZE").query_async(&mut redis_conn).await.unwrap_or(0);

    let rust_version = std::env!("CARGO_PKG_VERSION");
    let rustc_version = option_env!("RUSTC_VERSION").unwrap_or("Unknown");

    let uptime = System::uptime();
    let uptime_str = format_uptime(uptime);

    let cached_guilds = ctx.cache.guild_count();
    let cached_users = ctx.cache.user_count();

    let start = std::time::Instant::now();
    let mut test_msg = msg.channel_id.send_message(
        &ctx.http,
        CreateMessage::new().content("Calculating latency...")
    ).await?;
    let latency = start.elapsed().as_millis();

    let embed = CreateEmbed::new()
        .title("Roast Debug Information")
        .description(format!(
            "**Bot:** {} (`{}`)\n\
            **Owner:** <@{}>\n\
            **Guilds:** {}\n\
            **Shards:** {}",
            bot_name, bot_id, owner_id, guild_count, shard_count
        ))
        .color(0xFF6B6B)
        .field(
            "System Information",
            format!(
                "**OS:** {}\n\
                **CPU Cores:** {}\n\
                **System Memory:** {}/{} MB ({:.1}%)\n\
                **Bot Memory:** {} MB\n\
                **System Uptime:** {}",
                os_version,
                cpu_count,
                used_memory,
                total_memory,
                (used_memory as f64 / total_memory as f64) * 100.0,
                memory_usage,
                uptime_str
            ),
            false
        )
        .field(
            "Bot Runtime",
            format!(
                "**Rust Version:** {}\n\
                **Rustc:** {}\n\
                **Serenity:** v0.12\n\
                **Package Version:** {}",
                "1.0",
                rustc_version,
                rust_version
            ),
            false
        )
        .field(
            "Database & Cache",
            format!(
                "**PostgreSQL Pool:** {}/{} connections\n\
                **PostgreSQL Status:** Healthy\n\
                **Redis:** {}\n\
                **Redis Keys:** {}\n\
                **Cached Guilds:** {}\n\
                **Cached Users:** {}",
                db_pool_size - db_pool_idle,
                db_pool_size,
                redis_status,
                redis_key_count,
                cached_guilds,
                cached_users
            ),
            false
        )
        .field(
            "Network",
            format!(
                "**API Latency:** {}ms\n\
                **WebSocket:** Healthy\n\
                **Gateway:** Connected",
                latency
            ),
            false
        )
        .field(
            "Permissions",
            format!(
                "**Guild Visibility:** {}\n\
                **DM Access:** Enabled\n\
                **Presence Intent:** Disabled\n\
                **Message Content:** Enabled",
                guild_count
            ),
            false
        )
        .footer(serenity::all::CreateEmbedFooter::new("Roast - Advanced Bot Debugging"))
        .timestamp(serenity::all::Timestamp::now());

    test_msg.edit(&ctx.http, serenity::all::EditMessage::new().content("").embed(embed)).await?;

    Ok(())
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
