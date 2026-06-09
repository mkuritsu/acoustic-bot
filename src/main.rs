use serenity::{
    Client,
    all::{Context, EventHandler, GatewayIntents, Ready},
    async_trait,
};
use songbird::SerenityInit;
use tracing_subscriber::EnvFilter;

#[cfg(debug_assertions)]
use crate::dotenv::load_dotenv_vars;
use crate::{
    context::SerenityHttpClientExt,
    handlers::{command_handler::CommandHandler, voice_handler::VoiceHandler},
};

mod commands;
mod context;
mod handlers;

#[cfg(debug_assertions)]
mod dotenv;

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
        .register_http_client()
        .event_handler(CommandHandler)
        .event_handler(StartupHandler)
        .event_handler(VoiceHandler)
        .await?;
    client.start().await?;
    Ok(())
}
