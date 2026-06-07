use reqwest::Client as HttpClient;
use serenity::{
    Client,
    all::{Context, EventHandler, GatewayIntents, Ready},
    async_trait,
    prelude::TypeMapKey,
};
use songbird::SerenityInit;
use tracing_subscriber::EnvFilter;

use crate::{commands::CommandHandler, dotenv::load_dotenv_vars};

mod commands;
mod dotenv;

struct HttpKey;
impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

struct StartupHandler;

#[async_trait]
impl EventHandler for StartupHandler {
    async fn ready(&self, _: Context, _: Ready) {
        tracing::info!("Acoustic is now running!");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    #[cfg(debug_assertions)]
    load_dotenv_vars()?;

    let Ok(token) = std::env::var("BOT_TOKEN") else {
        anyhow::bail!("BOT_TOKEN environment variable not set!");
    };

    let intents = GatewayIntents::non_privileged();
    let mut client = Client::builder(&token, intents)
        .register_songbird()
        .event_handler(CommandHandler)
        .event_handler(StartupHandler)
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await?;
    client.start().await?;
    Ok(())
}
