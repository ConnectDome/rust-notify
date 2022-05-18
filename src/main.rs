use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, SmtpTransport, Transport};
use lettre_email::EmailBuilder;
use serde::Deserialize;
use serde_json::json;
use tera::{Context, Tera};
use tokio::time::{sleep, Duration};
use tracing_subscriber::EnvFilter;

#[derive(Clone, Debug, Deserialize)]
struct Config {
    discord: DiscordConfig,
    notion: NotionConfig,
    mail_simple: SimpleMailConfig,
    interval: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct DiscordConfig {
    webhook: String,
    username: Option<String>,
    avatar: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct NotionConfig {
    secret: String,
    database: String,
    api_version: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct SimpleMailConfig {
    to: String,
    from: String,
    domain: String,
    user: String,
    pass: String,
}

#[derive(Clone, Debug, Eq, Deserialize)]
struct Page {
    id: String,
    url: String,
}

impl PartialEq for Page {
    fn eq(&self, other: &Page) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Response {
    results: Vec<Page>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("RUST_NOTIFY_LOG"))
        .init();

    let filename =
        std::env::var("RUST_NOTIFY_CONFIG").unwrap_or_else(|_| String::from("config.toml"));
    let s = std::fs::read_to_string(filename)?;
    let config: Config = toml::from_str(&s)?;

    let client = reqwest::Client::new();

    let mut last = {
        let resp = client
            .post(format!(
                "https://api.notion.com/v1/databases/{}/query",
                config.notion.database
            ))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", config.notion.secret))
            .header(
                "Notion-Version",
                config.notion.api_version.as_deref().unwrap_or("2022-02-22"),
            )
            .send()
            .await?
            .json::<Response>()
            .await?;

        tracing::debug!("Fetched the pages for the first time");

        resp.results
    };

    loop {
        let resp = client
            .post(format!(
                "https://api.notion.com/v1/databases/{}/query",
                config.notion.database
            ))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", config.notion.secret))
            .header(
                "Notion-Version",
                config.notion.api_version.as_deref().unwrap_or("2022-02-22"),
            )
            .send()
            .await?
            .json::<Response>()
            .await?;

        tracing::debug!("Fetched latest notion pages");

        for page in &resp.results {
            if !last.contains(page) {
                tracing::info!("New page found, sending...");
                upload_to_discord(&client, page, &config).await?;
                send_email(&config, &page)?;
            }
        }

        last = resp.results;
        tracing::debug!("Set last pages");

        sleep(Duration::from_secs(config.interval)).await;
    }
}

async fn upload_to_discord(
    client: &reqwest::Client,
    page: &Page,
    config: &Config,
) -> anyhow::Result<()> {
    let body = json!({
        "content": format!("New post: {}", page.url), // TODO: make this configurable?
        "username": config.discord.username.as_ref(),
        "avatar_url": config.discord.avatar.as_ref(),
    });

    client
        .post(&config.discord.webhook)
        .json(&body)
        .send()
        .await?;

    tracing::debug!("Sent message to discord");

    Ok(())
}

fn send_email(config: &Config, page: &Page) -> anyhow::Result<()> {
    let mut ctx = Context::new();
    ctx.insert("url", &page.url);

    let raw = std::fs::read_to_string("email.html")?;
    let rendered = Tera::one_off(&raw, &ctx, true)?;

    let email = EmailBuilder::new()
        .to(&*config.mail_simple.to)
        .from(&*config.mail_simple.from)
        .subject("New post from ConnectDome") // TODO: make this configurable?
        .html(rendered)
        .build()?;

    let creds = Credentials::new(
        config.mail_simple.user.clone(),
        config.mail_simple.pass.clone(),
    );

    let smtp_client = SmtpClient::new_simple(&config.mail_simple.domain)?.credentials(creds);
    let mut mailer = SmtpTransport::new(smtp_client);

    mailer.send(email.into())?;

    Ok(())
}
