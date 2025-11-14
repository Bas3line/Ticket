use anyhow::Result;
use serenity::all::{Context, CreateEmbed, CreateMessage, Message};
use std::collections::HashMap;
use crate::database::Database;

pub async fn execute(ctx: &Context, msg: &Message, _db: &Database, args: &[&str]) -> Result<()> {
    let command_name = if args.is_empty() { "help" } else { args[0] };

    let docs = get_command_docs();

    let embed = if let Some(doc) = docs.get(command_name) {
        CreateEmbed::new()
            .title(format!("Command: {}", doc.name))
            .description(&doc.description)
            .field("Usage", &doc.usage, false)
            .field("Examples", &doc.examples, false)
            .field("Permissions", &doc.permissions, false)
            .color(0x5865F2)
    } else {
        CreateEmbed::new()
            .title("Command Not Found")
            .description(format!(
                "Command `{}` not found. Use `!doc help` to see all available commands.",
                command_name
            ))
            .color(0xFF0000)
    };

    msg.channel_id
        .send_message(&ctx.http, CreateMessage::new().embed(embed))
        .await?;

    Ok(())
}

struct CommandDoc {
    name: String,
    description: String,
    usage: String,
    examples: String,
    permissions: String,
}

fn get_command_docs() -> HashMap<String, CommandDoc> {
    let mut docs = HashMap::new();

    docs.insert(
        "help".to_string(),
        CommandDoc {
            name: "help".to_string(),
            description: "Displays a list of all available commands and their descriptions.".to_string(),
            usage: "`/help` or `!help`".to_string(),
            examples: "`/help` - Shows all commands\n`!help` - Prefix version".to_string(),
            permissions: "Everyone".to_string(),
        },
    );

    docs.insert(
        "setup".to_string(),
        CommandDoc {
            name: "setup".to_string(),
            description: "Configure the ticket system for your server. Set up ticket categories, support roles, log channels, and more.".to_string(),
            usage: "`/setup` or `!setup`".to_string(),
            examples: "`/setup` - Opens the interactive setup panel".to_string(),
            permissions: "Administrator".to_string(),
        },
    );

    docs.insert(
        "panel".to_string(),
        CommandDoc {
            name: "panel".to_string(),
            description: "Create a ticket panel in the current channel. Users can click buttons to open tickets for different categories.".to_string(),
            usage: "`/panel` or `!panel`".to_string(),
            examples: "`/panel` - Creates a panel with all configured ticket categories".to_string(),
            permissions: "Administrator".to_string(),
        },
    );

    docs.insert(
        "close".to_string(),
        CommandDoc {
            name: "close".to_string(),
            description: "Close a ticket. Optionally provide a reason for closing.".to_string(),
            usage: "`/close [reason]` or `!close [reason]`".to_string(),
            examples: "`/close` - Close without reason\n`/close reason: Issue resolved` - Close with reason\n`!close Issue resolved` - Prefix version".to_string(),
            permissions: "Ticket owner or support role".to_string(),
        },
    );

    docs.insert(
        "claim".to_string(),
        CommandDoc {
            name: "claim".to_string(),
            description: "Claim a ticket to indicate you are handling it. Stops all escalations and handle notifications for this ticket.".to_string(),
            usage: "`/claim` or `!claim`".to_string(),
            examples: "`/claim` - Claim the current ticket\n`!claim` - Prefix version".to_string(),
            permissions: "Support role".to_string(),
        },
    );

    docs.insert(
        "assign".to_string(),
        CommandDoc {
            name: "assign".to_string(),
            description: "Assign a ticket to a specific support team member. The assigned user will receive a DM notification about the ticket assignment.".to_string(),
            usage: "`/assign <user>` or `!assign <user>`".to_string(),
            examples: "`/assign user: @Support1` - Assign ticket to @Support1\n`!assign @Support1` - Assign using prefix".to_string(),
            permissions: "Support role".to_string(),
        },
    );

    docs.insert(
        "escalate".to_string(),
        CommandDoc {
            name: "escalate".to_string(),
            description: "Escalate a ticket to notify all support staff. Only works if no support messages have been sent yet. Sends hourly DM reminders until claimed or closed.".to_string(),
            usage: "`/escalate` or `!escalate`".to_string(),
            examples: "`/escalate` - Start hourly reminders to all support staff\n`!escalate` - Prefix version".to_string(),
            permissions: "Ticket owner".to_string(),
        },
    );

    docs.insert(
        "handle".to_string(),
        CommandDoc {
            name: "handle".to_string(),
            description: "Immediately notify all support staff to respond to this ticket. Sends a one-time DM to all support members with claim instructions.".to_string(),
            usage: "`/handle` or `!handle`".to_string(),
            examples: "`/handle` - Send immediate notification to all support staff\n`!handle` - Prefix version".to_string(),
            permissions: "Ticket owner".to_string(),
        },
    );

    docs.insert(
        "priority".to_string(),
        CommandDoc {
            name: "priority".to_string(),
            description: "Set the priority level of a ticket (Low, Medium, High, Urgent).".to_string(),
            usage: "`/priority <level>` or `!priority <level>`".to_string(),
            examples: "`/priority level: High` - Set ticket to high priority\n`!priority Urgent` - Set to urgent using prefix".to_string(),
            permissions: "Support role".to_string(),
        },
    );

    docs.insert(
        "note".to_string(),
        CommandDoc {
            name: "note".to_string(),
            description: "Add an internal note to a ticket. Notes are only visible to support staff and are included in transcripts.".to_string(),
            usage: "`/note <content>` or `!note <content>`".to_string(),
            examples: "`/note content: User reported payment issue` - Add note via slash\n`!note User seems frustrated` - Add note via prefix".to_string(),
            permissions: "Support role".to_string(),
        },
    );

    docs.insert(
        "transcript".to_string(),
        CommandDoc {
            name: "transcript".to_string(),
            description: "Generate a transcript of the ticket conversation and send it to the configured transcript channel.".to_string(),
            usage: "`/transcript` or `!transcript`".to_string(),
            examples: "`/transcript` - Generate and send transcript\n`!transcript` - Prefix version".to_string(),
            permissions: "Support role".to_string(),
        },
    );

    docs.insert(
        "blacklist".to_string(),
        CommandDoc {
            name: "blacklist".to_string(),
            description: "Prevent a user from creating tickets. User will not be able to open new tickets.".to_string(),
            usage: "`/blacklist <user> [reason]` or `!blacklist <user> [reason]`".to_string(),
            examples: "`/blacklist user: @spammer reason: Spam` - Blacklist with reason\n`!blacklist @troll Abusive behavior` - Prefix version".to_string(),
            permissions: "Administrator".to_string(),
        },
    );

    docs.insert(
        "reminder".to_string(),
        CommandDoc {
            name: "reminder".to_string(),
            description: "Set a timed reminder for yourself. Bot will ping you and send a DM when the time is up.".to_string(),
            usage: "`!reminder <time> <reason>` or `!remind <time> <reason>`".to_string(),
            examples: "`!reminder 30m Check ticket status` - Remind in 30 minutes\n\
                `!remind 2h Close ticket` - Remind in 2 hours\n\
                `!remindme 1d Follow up with user` - Remind in 1 day\n\n\
                Time formats: s (seconds), m (minutes), h (hours), d (days), w (weeks)".to_string(),
            permissions: "Everyone".to_string(),
        },
    );

    docs.insert(
        "unblacklist".to_string(),
        CommandDoc {
            name: "unblacklist".to_string(),
            description: "Remove a user from the ticket blacklist, allowing them to create tickets again.".to_string(),
            usage: "`/unblacklist <user>` or `!unblacklist <user>`".to_string(),
            examples: "`/unblacklist user: @user` - Remove blacklist\n`!unblacklist @user` - Prefix version".to_string(),
            permissions: "Administrator".to_string(),
        },
    );

    docs.insert(
        "tag".to_string(),
        CommandDoc {
            name: "tag".to_string(),
            description: "Tag system for creating and managing server-wide text snippets. Tags are case-insensitive and can be used to store frequently used messages, rules, FAQs, and more.".to_string(),
            usage: "`!tag <name>` or `/tag use <name>` - Display a tag\n\
                `!tag create <name> <content>` - Create a new tag\n\
                `!tag edit <name> <content>` - Edit your tag\n\
                `!tag delete <name>` - Delete your tag\n\
                `!tag list` - List all tags\n\
                `!tag search <query>` - Search tags\n\
                `!tag info <name>` - View tag details\n\
                `!tag popular` - Show most used tags\n\
                `!tag raw <name>` - View raw content\n\
                `!tag rename <old> <new>` - Rename your tag".to_string(),
            examples: "`!tag create rules Server rules: 1. Be respectful 2. No spam` - Create a rules tag\n\
                `/tag create name: faq content: Check our FAQ at...` - Create FAQ tag (slash)\n\
                `!tag rules` - Display the rules tag\n\
                `/tag use name: faq` - Display FAQ tag (slash)\n\
                `!tag edit rules Updated rules: 1. Be respectful 2. No spam 3. Use appropriate channels` - Edit tag\n\
                `/tag edit name: rules new_content: Updated rules...` - Edit with slash\n\
                `!tag list` - Show all server tags\n\
                `!tag search help` - Find tags containing 'help'\n\
                `!tag info rules` - View who created the tag and usage stats\n\
                `!tag popular` - Show top 10 most used tags\n\
                `!tag raw rules` - View tag content in code block\n\
                `!tag rename rules server-rules` - Rename a tag\n\
                `!tag delete rules` - Delete your tag".to_string(),
            permissions: "Everyone can use tags and create their own tags. Only tag creators can edit, delete, or rename their own tags.".to_string(),
        },
    );

    docs
}
