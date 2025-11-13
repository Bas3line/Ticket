use serenity::all::{ComponentInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage};
use crate::database::Database;
use crate::utils::{create_error_embed, create_success_embed};
use anyhow::Result;
use tracing::info;

pub async fn handle_ticket_create(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let guild_id = interaction.guild_id.unwrap().get() as i64;
    let user_id = interaction.user.id.get() as i64;

    let blacklisted: Option<(bool,)> = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM blacklist WHERE target_id = $1 AND target_type = 'user')"
    )
    .bind(user_id)
    .fetch_optional(&db.pool)
    .await?;

    if let Some((true,)) = blacklisted {
        let embed = create_error_embed(
            "Blacklisted",
            "You are not allowed to create tickets in this server",
        );

        interaction
            .create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true)
            ))
            .await?;
        return Ok(());
    }

    let existing_tickets = crate::database::ticket::get_user_tickets(&db.pool, guild_id, user_id).await?;

    let guild = crate::database::ticket::get_or_create_guild(&db.pool, guild_id).await?;

    let max_tickets = guild.ticket_limit_per_user.unwrap_or(1);

    if existing_tickets.len() >= max_tickets as usize {
        let embed = create_error_embed(
            "Ticket Limit Reached",
            format!("You can only have {} open ticket(s) at a time", max_tickets),
        );

        interaction
            .create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true)
            ))
            .await?;
        return Ok(());
    }

    let channel_name = format!("ticket-{}", interaction.user.id.get());

    let guild_id_obj = interaction.guild_id.unwrap();
    let everyone_role = serenity::all::RoleId::new(guild_id_obj.get());

    let mut channel_builder = serenity::all::CreateChannel::new(&channel_name)
        .kind(serenity::all::ChannelType::Text);

    if let Some(category_id) = guild.ticket_category_id {
        channel_builder = channel_builder.category(serenity::all::ChannelId::new(category_id as u64));
    }

    channel_builder = channel_builder
        .permissions(vec![
            serenity::all::PermissionOverwrite {
                allow: serenity::all::Permissions::empty(),
                deny: serenity::all::Permissions::VIEW_CHANNEL,
                kind: serenity::all::PermissionOverwriteType::Role(everyone_role),
            },
            serenity::all::PermissionOverwrite {
                allow: serenity::all::Permissions::VIEW_CHANNEL
                    | serenity::all::Permissions::SEND_MESSAGES
                    | serenity::all::Permissions::READ_MESSAGE_HISTORY,
                deny: serenity::all::Permissions::empty(),
                kind: serenity::all::PermissionOverwriteType::Member(interaction.user.id),
            },
        ]);

    let channel = guild_id_obj.create_channel(&ctx.http, channel_builder).await?;

    let support_roles = crate::database::ticket::get_support_roles(&db.pool, guild_id).await?;
    for role in support_roles {
        channel
            .create_permission(
                &ctx.http,
                serenity::all::PermissionOverwrite {
                    allow: serenity::all::Permissions::VIEW_CHANNEL
                        | serenity::all::Permissions::SEND_MESSAGES
                        | serenity::all::Permissions::READ_MESSAGE_HISTORY,
                    deny: serenity::all::Permissions::empty(),
                    kind: serenity::all::PermissionOverwriteType::Role(serenity::all::RoleId::new(role.role_id as u64)),
                },
            )
            .await?;
    }

    let ticket = crate::database::ticket::create_ticket(
        &db.pool,
        guild_id,
        channel.id.get() as i64,
        user_id,
        None,
    )
    .await?;

    let guild_icon = interaction
        .guild_id
        .and_then(|gid| ctx.cache.guild(gid))
        .and_then(|g| g.icon_url());

    let mut embed = crate::utils::create_embed(
        format!("Ticket - {}", user_id),
        format!(
            "Welcome <@{}>!\n\nA support team member will be with you shortly.\nTo close this ticket, use `/close`",
            user_id
        ),
    );

    if let Some(icon_url) = guild_icon {
        embed = embed.thumbnail(icon_url);
    }

    let guild = crate::database::ticket::get_or_create_guild(&db.pool, guild_id).await?;
    let claim_enabled = guild.claim_buttons_enabled.unwrap_or(true);

    let mut buttons = Vec::new();

    if claim_enabled {
        let claim_button = serenity::all::CreateButton::new("ticket_claim")
            .label("Claim")
            .style(serenity::all::ButtonStyle::Success);
        buttons.push(claim_button);
    }

    let close_button = serenity::all::CreateButton::new("ticket_close")
        .label("Close")
        .style(serenity::all::ButtonStyle::Danger);

    let transcript_button = serenity::all::CreateButton::new("ticket_transcript")
        .label("Transcript")
        .style(serenity::all::ButtonStyle::Secondary);

    buttons.push(close_button);
    buttons.push(transcript_button);

    let components = vec![serenity::all::CreateActionRow::Buttons(buttons)];

    let ping_role_mention = if let Ok(Some((ping_role_id,))) = sqlx::query_as::<_, (Option<i64>,)>(
        "SELECT ping_role_id FROM guilds WHERE guild_id = $1"
    )
    .bind(guild_id)
    .fetch_optional(&db.pool)
    .await
    {
        ping_role_id.map(|id| format!("<@&{}>", id))
    } else {
        None
    };

    let welcome_content = if let Some(role) = ping_role_mention {
        format!("{} New ticket opened!", role)
    } else {
        String::new()
    };

    let welcome_msg = channel
        .send_message(&ctx.http, serenity::all::CreateMessage::new().content(welcome_content).embed(embed).components(components))
        .await?;

    sqlx::query("UPDATE tickets SET opening_message_id = $1 WHERE id = $2")
        .bind(welcome_msg.id.get() as i64)
        .bind(ticket.id)
        .execute(&db.pool)
        .await?;

    let response_embed = create_success_embed(
        "Ticket Created",
        format!("Your ticket has been created: <#{}>", channel.id),
    );

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .embed(response_embed)
                .ephemeral(true)
        ))
        .await?;

    if let Some(log_channel_id) = guild.log_channel_id {
        let log_channel = serenity::all::ChannelId::new(log_channel_id as u64);
        let log_embed = crate::utils::create_embed(
            "Ticket Opened",
            format!(
                "Ticket: ticket-{}\nUser: <@{}>\nChannel: <#{}>",
                user_id, user_id, channel.id
            ),
        );

        log_channel
            .send_message(&ctx.http, serenity::all::CreateMessage::new().embed(log_embed))
            .await?;
    }

    Ok(())
}

pub async fn handle_ticket_claim(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let channel_id = interaction.channel_id.get() as i64;
    let claimer_id = interaction.user.id.get() as i64;

    let ticket = crate::database::ticket::get_ticket_by_channel(&db.pool, channel_id).await?;

    if let Some(ticket) = ticket {
        // Check if user has support role
        let support_roles = crate::database::ticket::get_support_roles(&db.pool, ticket.guild_id).await?;

        if support_roles.is_empty() {
            let embed = create_error_embed(
                "No Support Roles",
                "No support roles have been configured for this server",
            );

            interaction
                .create_response(&ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true)
                ))
                .await?;
            return Ok(());
        }

        let guild_id = interaction.guild_id.ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;
        let member = guild_id.member(&ctx.http, interaction.user.id).await?;

        let has_support_role = support_roles.iter().any(|role| {
            member.roles.contains(&serenity::all::RoleId::new(role.role_id as u64))
        });

        if !has_support_role {
            let embed = create_error_embed(
                "Permission Denied",
                "Only users with a support role can claim tickets",
            );

            interaction
                .create_response(&ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true)
                ))
                .await?;
            return Ok(());
        }

        if ticket.is_claimed() {
            let embed = create_error_embed(
                "Already Claimed",
                format!("This ticket is already claimed by <@{}>", ticket.claimed_by.unwrap()),
            );

            interaction
                .create_response(&ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true)
                ))
                .await?;
            return Ok(());
        }

        crate::database::ticket::claim_ticket(&db.pool, ticket.id, claimer_id).await?;

        let _ = crate::database::ticket::deactivate_escalation(&db.pool, ticket.id).await;

        let guild = crate::database::ticket::get_or_create_guild(&db.pool, ticket.guild_id).await?;
        let log_embed = crate::utils::create_embed(
            "Ticket Claimed",
            format!("Ticket: ticket-{}\nClaimed by: <@{}>\nOwner: <@{}>", ticket.owner_id, claimer_id, ticket.owner_id)
        );
        let _ = crate::utils::send_log(ctx, guild.log_channel_id, log_embed).await;

        let embed = create_success_embed(
            "Ticket Claimed",
            format!("<@{}> has claimed this ticket", claimer_id),
        );

        let unclaim_button = serenity::all::CreateButton::new("ticket_unclaim")
            .label("Unclaim")
            .style(serenity::all::ButtonStyle::Primary);

        let close_button = serenity::all::CreateButton::new("ticket_close")
            .label("Close")
            .style(serenity::all::ButtonStyle::Danger);

        let transcript_button = serenity::all::CreateButton::new("ticket_transcript")
            .label("Transcript")
            .style(serenity::all::ButtonStyle::Secondary);

        let components = vec![serenity::all::CreateActionRow::Buttons(vec![
            unclaim_button,
            close_button,
            transcript_button,
        ])];

        interaction.channel_id
            .send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed).components(components))
            .await?;

        interaction
            .create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Ticket claimed successfully")
                    .ephemeral(true)
            ))
            .await?;
    }

    Ok(())
}

pub async fn handle_ticket_unclaim(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let channel_id = interaction.channel_id.get() as i64;
    let user_id = interaction.user.id.get() as i64;

    let ticket = crate::database::ticket::get_ticket_by_channel(&db.pool, channel_id).await?;

    if let Some(ticket) = ticket {
        if let Some(claimed_by) = ticket.claimed_by {
            if claimed_by != user_id {
                let embed = create_error_embed(
                    "Permission Denied",
                    "Only the user who claimed this ticket can unclaim it",
                );

                interaction
                    .create_response(&ctx.http, CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .ephemeral(true)
                    ))
                    .await?;
                return Ok(());
            }
        } else {
            let embed = create_error_embed(
                "Not Claimed",
                "This ticket is not claimed",
            );

            interaction
                .create_response(&ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true)
                ))
                .await?;
            return Ok(());
        }

        crate::database::ticket::unclaim_ticket(&db.pool, ticket.id).await?;

        // Send log
        let guild = crate::database::ticket::get_or_create_guild(&db.pool, ticket.guild_id).await?;
        let log_embed = crate::utils::create_embed(
            "Ticket Unclaimed",
            format!("Ticket: ticket-{}\nUnclaimed by: <@{}>\nOwner: <@{}>", ticket.owner_id, user_id, ticket.owner_id)
        );
        let _ = crate::utils::send_log(ctx, guild.log_channel_id, log_embed).await;

        let embed = create_success_embed(
            "Ticket Unclaimed",
            format!("<@{}> has unclaimed this ticket", user_id),
        );

        let claim_button = serenity::all::CreateButton::new("ticket_claim")
            .label("Claim")
            .style(serenity::all::ButtonStyle::Success);

        let close_button = serenity::all::CreateButton::new("ticket_close")
            .label("Close")
            .style(serenity::all::ButtonStyle::Danger);

        let transcript_button = serenity::all::CreateButton::new("ticket_transcript")
            .label("Transcript")
            .style(serenity::all::ButtonStyle::Secondary);

        let components = vec![serenity::all::CreateActionRow::Buttons(vec![
            claim_button,
            close_button,
            transcript_button,
        ])];

        interaction.channel_id
            .send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed).components(components))
            .await?;

        interaction
            .create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Ticket unclaimed successfully")
                    .ephemeral(true)
            ))
            .await?;
    }

    Ok(())
}

pub async fn handle_ticket_close(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let channel_id = interaction.channel_id.get() as i64;

    let ticket = crate::database::ticket::get_ticket_by_channel(&db.pool, channel_id).await?;

    if let Some(ticket) = ticket {
        // Acknowledge the interaction IMMEDIATELY (within 3 seconds)
        interaction
            .create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Closing ticket and generating transcript... This channel will be deleted in 5 seconds.")
                    .ephemeral(true)
            ))
            .await?;

        let closed_at = chrono::Utc::now();

        info!("Ticket {} closed in channel {} by user {}", ticket.ticket_number, channel_id, interaction.user.id.get());

        let messages = crate::database::ticket::get_ticket_messages(&db.pool, ticket.id).await?;

        let owner = ctx.http.get_user(serenity::all::UserId::new(ticket.owner_id as u64)).await?;
        let claimed_by_name = if let Some(claimer_id) = ticket.claimed_by {
            let claimer = ctx.http.get_user(serenity::all::UserId::new(claimer_id as u64)).await?;
            Some(claimer.name)
        } else {
            None
        };

        let html = crate::utils::transcript::generate_transcript(
            ticket.ticket_number,
            owner.name,
            ticket.created_at,
            Some(closed_at),
            claimed_by_name,
            messages,
        )
        .await?;

        let filepath = crate::utils::transcript::save_transcript(ticket.guild_id, ticket.ticket_number, html).await?;

        if let Ok(guild) = sqlx::query_as::<_, crate::models::Guild>(
            "SELECT * FROM guilds WHERE guild_id = $1"
        )
        .bind(ticket.guild_id)
        .fetch_one(&db.pool)
        .await
        {
            // Send to transcript channel if configured
            if let Some(transcript_channel_id) = guild.transcript_channel_id {
                let channel = serenity::all::ChannelId::new(transcript_channel_id as u64);

                let file = serenity::all::CreateAttachment::path(&filepath).await?;

                let embed = crate::utils::create_embed(
                    format!("Ticket - {} Closed", ticket.owner_id),
                    format!(
                        "Owner: <@{}>\nClosed by: <@{}>\nClosed at: <t:{}:F>",
                        ticket.owner_id,
                        interaction.user.id,
                        closed_at.timestamp()
                    ),
                );

                channel
                    .send_message(&ctx.http, serenity::all::CreateMessage::new().embed(embed).add_file(file))
                    .await?;

                info!("Transcript for ticket {} sent to channel {}", ticket.ticket_number, transcript_channel_id);
            }

            // Send to ticket owner's DM
            let owner_user = serenity::all::UserId::new(ticket.owner_id as u64).to_user(&ctx.http).await;
            if let Ok(user) = owner_user {
                if let Ok(dm) = user.create_dm_channel(&ctx.http).await {
                    let dm_embed = crate::utils::create_embed(
                        "Ticket Closed - Transcript",
                        format!("Your ticket #{} has been closed. Here's the transcript.", ticket.ticket_number)
                    ).color(0x5865F2);
                    let dm_file = serenity::all::CreateAttachment::path(&filepath).await?;
                    let _ = dm.send_message(&ctx.http,
                        serenity::all::CreateMessage::new()
                            .embed(dm_embed)
                            .add_file(dm_file)
                    ).await;
                }
            }

            let _ = crate::utils::transcript::delete_transcript(&filepath).await;
        }

        crate::database::ticket::delete_ticket_messages(&db.pool, ticket.id).await?;

        let mut redis_conn = db.redis.clone();
        let _ = crate::database::ticket::cleanup_priority_ping(&mut redis_conn, ticket.id).await;

        let _ = crate::database::ticket::deactivate_escalation(&db.pool, ticket.id).await;

        // Send log
        if let Ok(guild) = crate::database::ticket::get_or_create_guild(&db.pool, ticket.guild_id).await {
            let log_embed = crate::utils::create_embed(
                "Ticket Closed",
                format!("Ticket: ticket-{}\nOwner: <@{}>\nClosed by: <@{}>\nClosed at: <t:{}:F>",
                    ticket.owner_id, ticket.owner_id, interaction.user.id, closed_at.timestamp())
            );
            let _ = crate::utils::send_log(ctx, guild.log_channel_id, log_embed).await;
        }

        crate::database::ticket::close_ticket(&db.pool, ticket.id).await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let _ = interaction.channel_id.delete(&ctx.http).await;
    }

    Ok(())
}

pub async fn handle_ticket_transcript(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
    let channel_id = interaction.channel_id.get() as i64;

    let ticket = crate::database::ticket::get_ticket_by_channel(&db.pool, channel_id).await?;

    if let Some(ticket) = ticket {
        let messages = crate::database::ticket::get_ticket_messages(&db.pool, ticket.id).await?;

        let owner = ctx.http.get_user(serenity::all::UserId::new(ticket.owner_id as u64)).await?;
        let claimed_by_name = if let Some(claimer_id) = ticket.claimed_by {
            let claimer = ctx.http.get_user(serenity::all::UserId::new(claimer_id as u64)).await?;
            Some(claimer.name)
        } else {
            None
        };

        let html = crate::utils::transcript::generate_transcript(
            ticket.ticket_number,
            owner.name,
            ticket.created_at,
            ticket.closed_at,
            claimed_by_name,
            messages,
        )
        .await?;

        let filepath = crate::utils::transcript::save_transcript(ticket.guild_id, ticket.ticket_number, html).await?;

        let file = serenity::all::CreateAttachment::path(&filepath).await?;

        interaction
            .create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Transcript generated")
                    .add_file(file)
                    .ephemeral(true)
            ))
            .await?;

        let _ = crate::utils::transcript::delete_transcript(&filepath).await;
    }

    Ok(())
}

pub async fn handle_ticket_create_category(
    ctx: &Context,
    interaction: &ComponentInteraction,
    db: &Database,
) -> Result<()> {
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

    let parts: Vec<&str> = interaction.data.custom_id.split('_').collect();
    if parts.len() < 4 {
        return Ok(());
    }

    let category_id = parts[3];

    use uuid::Uuid;
    let category_uuid = Uuid::parse_str(category_id)?;

    let category: Option<(String, String)> = sqlx::query_as(
        "SELECT name, description FROM ticket_categories WHERE id = $1 AND guild_id = $2"
    )
    .bind(&category_uuid)
    .bind(guild_id.get() as i64)
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

    let guild_data = crate::database::ticket::get_or_create_guild(&db.pool, guild_id.get() as i64).await?;

    let max_tickets = guild_data.ticket_limit_per_user.unwrap_or(1);

    if existing_tickets.len() >= max_tickets as usize {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(format!("You can only have {} open ticket(s) at a time", max_tickets))
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

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

    let embed = crate::utils::create_embed(
        format!("Ticket - {}", category_name),
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

    let ping_role_mention = if let Ok(Some((ping_role_id,))) = sqlx::query_as::<_, (Option<i64>,)>(
        "SELECT ping_role_id FROM guilds WHERE guild_id = $1"
    )
    .bind(guild_id.get() as i64)
    .fetch_optional(&db.pool)
    .await
    {
        ping_role_id.map(|id| format!("<@&{}>", id))
    } else {
        None
    };

    let welcome_content = if let Some(role) = ping_role_mention {
        format!("{} New ticket opened!", role)
    } else {
        String::new()
    };

    let welcome_msg = ticket_channel
        .send_message(
            &ctx.http,
            serenity::all::CreateMessage::new()
                .content(welcome_content)
                .embed(embed)
                .components(vec![serenity::all::CreateActionRow::Buttons(buttons)]),
        )
        .await?;

    sqlx::query("UPDATE tickets SET opening_message_id = $1 WHERE id = $2")
        .bind(welcome_msg.id.get() as i64)
        .bind(ticket.id)
        .execute(&db.pool)
        .await?;

    if let Ok(Some((use_custom, msg))) = crate::database::ticket::get_category_welcome_message(&db.pool, category_uuid).await {
        if use_custom {
            if let Some(custom_msg) = msg {
                ticket_channel.send_message(
                    &ctx.http,
                    serenity::all::CreateMessage::new()
                        .content(custom_msg.replace("{user}", &format!("<@{}>", user_id)))
                ).await?;
            }
        }
    }

    if let Some(log_channel_id) = guild_data.log_channel_id {
        let log_channel = serenity::all::ChannelId::new(log_channel_id as u64);
        let log_embed = crate::utils::create_embed(
            "Ticket Created",
            format!(
                "**Ticket:** ticket-{}\n**User:** <@{}>\n**Category:** {}\n**Channel:** <#{}>\n**Created:** <t:{}:F>",
                user_id,
                user_id,
                category_name,
                ticket_channel.id,
                ticket.created_at.timestamp()
            )
        ).color(0x57F287);

        let _ = log_channel.send_message(
            &ctx.http,
            serenity::all::CreateMessage::new().embed(log_embed)
        ).await;
    }

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
