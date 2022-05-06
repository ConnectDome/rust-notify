use serde::Deserialize;
use serde_json::json;

#[derive(Clone, Debug, Deserialize)]
struct Config {
    discord: DiscordConfig,
}

#[derive(Clone, Debug, Deserialize)]
struct DiscordConfig {
    webhook: String,
    username: Option<String>,
    avatar: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let s = std::fs::read_to_string("config.toml")?; // todo: make this configurable
    let config: Config = toml::from_str(&s)?;

    let body = json!({
        "content": "Hello, world!"
    });

    let client = reqwest::Client::new();
    client
        .post(config.discord.webhook)
        .json(&body)
        .send()
        .await?;

    Ok(())
}
