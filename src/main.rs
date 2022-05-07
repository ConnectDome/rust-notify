use serde::Deserialize;
use serde_json::json;
use tokio::time::{sleep, Duration};
use tracing_subscriber::EnvFilter;

#[derive(Clone, Debug, Deserialize)]
struct Config {
    discord: DiscordConfig,
    notion: NotionConfig,
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
            .header("Notion-Version", "2022-02-22")
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
            .header("Notion-Version", "2022-02-22")
            .send()
            .await?
            .json::<Response>()
            .await?;

        tracing::debug!("Fetched latest notion pages");

        for page in &resp.results {
            if !last.contains(page) {
                tracing::info!("New page found, sending...");
                upload_to_discord(&client, page, &config).await?;
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
        "content": format!("New post: {}", page.url), // this can be changed
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
