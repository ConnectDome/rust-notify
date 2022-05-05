extern crate dotenv;
extern crate webhook;
use webhook::{
    client::{WebhookClient, WebhookResult},
};

const IMAGE_URL: &'static str = "https://cdn.discordapp.com/avatars/525730915675275296/3a8c31e9f8da1e7e101cbc28764cb4f3.webp?size=160";

#[tokio::main]
async fn main() -> WebhookResult<()> {
    dotenv::dotenv()?;
    let webhook_url = dotenv::var("WEBHOOK_URL")?;
    let client = WebhookClient::new(&webhook_url);
    let webhook_info = client.get_info().await?;
    println!("{:?}", webhook_info);

    client.send(|message| message
        .username("Rust Notify")
        .avatar_url(IMAGE_URL)
        .content("An update from ConnectDome Product Blog")

    ).await?;

    Ok(())
}