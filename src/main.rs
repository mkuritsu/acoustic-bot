use std::fs;

use reqwest::Client as HttpClient;
use serenity::{
    Client,
    all::{
        Command, CommandOptionType, Context, CreateCommand, CreateCommandOption, CreateEmbed,
        CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse,
        EventHandler, GatewayIntents, Interaction, Ready, ResolvedValue,
    },
    async_trait,
    prelude::TypeMapKey,
};
use songbird::{SerenityInit, input::YoutubeDl};
use tracing_subscriber::EnvFilter;

const DOTENV_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.env");

fn load_env_vars() -> anyhow::Result<()> {
    fs::read_to_string(DOTENV_PATH)?
        .lines()
        .filter(|line| !line.trim().is_empty()) // filter blank lines
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .for_each(|(key, value)| {
            unsafe { std::env::set_var(key, value) };
        });
    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        let play_command = CreateCommand::new("play")
            .description("play some music!")
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "url", "the youtube url")
                    .required(true),
            );

        let commands = vec![play_command];
        if let Err(err) = Command::set_global_commands(&ctx.http, commands).await {
            eprintln!("{err:?}");
        }
        println!("Bot ready!");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(cmd) = interaction
            && cmd.data.name == "play"
        {
            let http_client = {
                let data = ctx.data.read().await;
                data.get::<HttpKey>().cloned().expect("Should always exist")
            };

            let Some(guild_id) = cmd.guild_id else {
                let response = CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_embed(CreateEmbed::new().description("No guild id!")),
                );
                let _ = cmd.create_response(&ctx.http, response).await;
                return;
            };

            let user_id = cmd.user.id;
            let Ok(voice_state) = guild_id.get_user_voice_state(&ctx.http, user_id).await else {
                let response = CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_embed(CreateEmbed::new().description("Failed to get voice state!")),
                );
                let _ = cmd.create_response(&ctx.http, response).await;
                return;
            };

            let Some(channel_id) = voice_state.channel_id else {
                let response = CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_embed(CreateEmbed::new().description("Not in VC!")),
                );
                let _ = cmd.create_response(&ctx.http, response).await;
                return;
            };

            let options = cmd.data.options();
            let url = options.first().unwrap();
            let ResolvedValue::String(url) = url.value else {
                let response = CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_embed(CreateEmbed::new().description("Not resolved to string!")),
                );
                let _ = cmd.create_response(&ctx.http, response).await;
                return;
            };

            let manager = songbird::get(&ctx).await.expect("Should have instance");

            println!("Received play for {url} in {channel_id}");

            let handler_lock = match manager.join(guild_id, channel_id).await {
                Ok(handler) => handler,
                Err(err) => {
                    eprintln!("Failed to join VC: {err:?}");
                    let response = CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().add_embed(
                            CreateEmbed::new().description("Cloud not get channel info!"),
                        ),
                    );
                    let _ = cmd.create_response(&ctx.http, response).await;
                    return;
                }
            };

            let _ = cmd
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
                )
                .await;

            let mut handler = handler_lock.lock().await;
            let mut yt_source = YoutubeDl::new_search(http_client, String::from(url));
            let outputs = yt_source.query(1).await.unwrap();
            let output = outputs.first().unwrap();
            let track_handle = handler.play_input(yt_source.clone().into());

            let _ = cmd
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new().add_embed(CreateEmbed::new().description(
                        format!(
                            "Started playing {}!",
                            output.title.clone().unwrap_or(String::from("?"))
                        ),
                    )),
                )
                .await;
        }
    }
}

struct HttpKey;
impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    load_env_vars()?;
    let Ok(token) = std::env::var("BOT_TOKEN") else {
        anyhow::bail!("BOT_TOKEN environment variable not set!");
    };

    let intents = GatewayIntents::non_privileged();
    let mut client = Client::builder(&token, intents)
        .register_songbird()
        .event_handler(Handler)
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await?;
    client.start().await?;
    Ok(())
}
