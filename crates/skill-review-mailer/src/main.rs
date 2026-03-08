use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "skill-review-mailer")]
#[command(about = "Send skill review summaries and check for approval replies via Gmail")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send an email via Gmail SMTP
    Send {
        /// Recipient email address
        #[arg(long)]
        to: String,
        /// Email subject line
        #[arg(long)]
        subject: String,
        /// Path to file containing the email body (plain text or HTML)
        #[arg(long)]
        body: PathBuf,
    },
    /// Check Gmail IMAP for approve/decline replies
    CheckReplies {
        /// Only check emails received after this ISO 8601 datetime
        #[arg(long)]
        since: String,
    },
}

#[derive(Debug, Serialize)]
struct ReplyResponse {
    id: String,
    action: String, // "approve" or "decline"
    received_at: String,
}

fn get_credentials() -> Result<(String, String)> {
    let _ = dotenvy::dotenv();
    let user = std::env::var("GMAIL_USER").context("GMAIL_USER env var not set")?;
    let password =
        std::env::var("GMAIL_APP_PASSWORD").context("GMAIL_APP_PASSWORD env var not set")?;
    Ok((user, password))
}

fn send_email(to: &str, subject: &str, body_path: &PathBuf) -> Result<()> {
    let (user, password) = get_credentials()?;
    let body_content =
        std::fs::read_to_string(body_path).context("failed to read body file")?;

    use lettre::message::header::ContentType;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    let lower = body_content.to_lowercase();
    let content_type = if ["<html", "<body", "<p>", "<div", "<br", "<h1"]
        .iter()
        .any(|tag| lower.contains(tag))
    {
        ContentType::TEXT_HTML
    } else {
        ContentType::TEXT_PLAIN
    };

    let email = Message::builder()
        .from(user.parse().context("invalid sender address")?)
        .to(to.parse().context("invalid recipient address")?)
        .subject(subject)
        .header(content_type)
        .body(body_content)
        .context("failed to build email")?;

    let creds = Credentials::new(user, password);

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .context("failed to create SMTP relay")?
        .credentials(creds)
        .build();

    mailer.send(&email).context("failed to send email")?;

    eprintln!("Email sent to {to}");
    Ok(())
}

fn check_replies(since: &str) -> Result<()> {
    let (user, password) = get_credentials()?;
    let since_dt: DateTime<Utc> = since
        .parse()
        .context("invalid --since datetime, expected ISO 8601")?;

    let tls = native_tls::TlsConnector::builder()
        .build()
        .context("failed to build TLS connector")?;

    let client = imap::connect(("imap.gmail.com", 993), "imap.gmail.com", &tls)
        .context("failed to connect to Gmail IMAP")?;

    let mut session = client
        .login(&user, &password)
        .map_err(|e| anyhow::anyhow!("IMAP login failed: {}", e.0))?;

    session.select("INBOX").context("failed to select INBOX")?;

    // Search for emails since the given date
    let imap_date = since_dt.format("%d-%b-%Y").to_string();
    let search_query = format!("SINCE {imap_date} SUBJECT \"Skill Review Summary\"");
    let message_ids = session
        .search(&search_query)
        .context("IMAP search failed")?;

    let mut responses: Vec<ReplyResponse> = Vec::new();

    if !message_ids.is_empty() {
        let id_list: Vec<String> = message_ids.iter().map(|id| id.to_string()).collect();
        let fetch_range = id_list.join(",");
        let messages = session
            .fetch(&fetch_range, "BODY[TEXT]")
            .context("failed to fetch messages")?;

        for msg in messages.iter() {
            let body = msg.text().unwrap_or_default();
            let body_str = std::str::from_utf8(body).unwrap_or_default();

            for line in body_str.lines() {
                let trimmed = line.trim().to_lowercase();
                if let Some(rest) = trimmed.strip_prefix("approve ") {
                    let id = rest.trim().to_uppercase();
                    if id.starts_with("SR-") {
                        responses.push(ReplyResponse {
                            id,
                            action: "approve".to_string(),
                            received_at: Utc::now().to_rfc3339(),
                        });
                    }
                } else if let Some(rest) = trimmed.strip_prefix("decline ") {
                    let id = rest.trim().to_uppercase();
                    if id.starts_with("SR-") {
                        responses.push(ReplyResponse {
                            id,
                            action: "decline".to_string(),
                            received_at: Utc::now().to_rfc3339(),
                        });
                    }
                }
            }
        }
    }

    session.logout().context("IMAP logout failed")?;

    let json = serde_json::to_string_pretty(&responses)
        .context("failed to serialize responses")?;
    println!("{json}");

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Send { to, subject, body } => {
            send_email(&to, &subject, &body)?;
        }
        Commands::CheckReplies { since } => {
            check_replies(&since)?;
        }
    }

    Ok(())
}
