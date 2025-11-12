use crate::models::TicketMessage;
use anyhow::Result;
use askama::Template;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;

#[derive(Template)]
#[template(path = "transcript.html")]
struct TranscriptTemplate {
    ticket_number: i32,
    owner_name: String,
    created_at: String,
    closed_at: Option<String>,
    claimed_by: Option<String>,
    messages: Vec<TicketMessage>,
    generated_at: String,
}

pub async fn generate_transcript(
    ticket_number: i32,
    owner_name: String,
    created_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
    claimed_by: Option<String>,
    messages: Vec<TicketMessage>,
) -> Result<String> {
    let template = TranscriptTemplate {
        ticket_number,
        owner_name,
        created_at: created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        closed_at: closed_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
        claimed_by,
        messages,
        generated_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    };

    let html = template.render()?;
    Ok(html)
}

pub async fn save_transcript(guild_id: i64, ticket_number: i32, html: String) -> Result<String> {
    let dir = Path::new("transcripts");
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    let filename = format!("transcript-{}-{}.html", guild_id, ticket_number);
    let filepath = dir.join(&filename);

    fs::write(&filepath, html)?;

    Ok(filepath.to_string_lossy().to_string())
}

pub async fn delete_transcript(filepath: &str) -> Result<()> {
    if Path::new(filepath).exists() {
        fs::remove_file(filepath)?;
    }
    Ok(())
}
