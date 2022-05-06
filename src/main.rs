use serde::Deserialize;
use serde_json::json;

#[derive(Clone, Debug, Deserialize)]
struct Config {
    discord: DiscordConfig,
    notion: NotionConfig,
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

#[derive(Clone, Debug, Deserialize)]
struct Page {
    id: String,
    last_edited_time: String,
    url: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Response {
    results: Vec<Page>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let s = std::fs::read_to_string("config.toml")?; // todo: make this configurable
    let config: Config = toml::from_str(&s)?;

    let body = json!({
        "content": "Hello, world!"
    });

    let client = reqwest::Client::new();

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

    //client
    //    .post(config.discord.webhook)
    //    .json(&body)
    //    .send()
    //    .await?;

    dbg!(&resp);

    Ok(())
}
