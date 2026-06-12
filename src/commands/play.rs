use std::time::Duration;

use anyhow::Context as _;
use serenity::all::{
    ChannelId, Color, CommandInteraction, CommandOptionType, Context, CreateCommand,
    CreateCommandOption, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse, GuildId, ResolvedValue,
};
use songbird::{
    Event, TrackEvent,
    input::{AuxMetadata, Compose, YoutubeDl},
};
use tracing::trace;

use crate::{
    commands::check_user_channel, context::ContextHttpClientExt,
    handlers::track_end_handler::TrackEndHandler,
};

pub fn create() -> CreateCommand {
    CreateCommand::new("play")
        .description("play some music!")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "query",
                "youtube url or search query",
            )
            .required(true),
        )
}

pub async fn execute(ctx: &Context, cmd: &CommandInteraction) -> anyhow::Result<()> {
    let (guild_id, channel_id) = check_user_channel(ctx, cmd).await?;

    let query = cmd
        .data
        .options()
        .first()
        .cloned()
        .context("You need to provide a query!")?;
    let ResolvedValue::String(query) = query.value else {
        anyhow::bail!("Invalid query type!");
    };
    trace!("Received play for {query} in {channel_id}");

    let _ = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
        )
        .await;

    let metadata = start_playback(ctx, query, guild_id, channel_id).await?;
    send_reply(ctx, cmd, metadata).await;

    Ok(())
}

async fn start_playback(
    ctx: &Context,
    query: &str,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> anyhow::Result<AuxMetadata> {
    let manager = songbird::get(ctx)
        .await
        .expect("Should have songbird instance");
    let http_client = ctx.get_http_client().await;

    let handler = manager
        .join(guild_id, channel_id)
        .await
        .context("Could not get channel info!")?;
    let mut handler = handler.lock().await;
    handler.deafen(true).await.ok();

    let mut yt_source = if query.starts_with("https://") || query.starts_with("http://") {
        YoutubeDl::new(http_client, String::from(query))
    } else {
        YoutubeDl::new_search(http_client, String::from(query))
    };
    let metadata = yt_source.aux_metadata().await.context("Song not found!")?;
    let track = handler.enqueue_with_preload(yt_source.into(), Some(Duration::from_secs(20)));
    track
        .add_event(
            Event::Track(TrackEvent::End),
            TrackEndHandler {
                guild_id,
                manager: manager.clone(),
            },
        )
        .ok();

    Ok(metadata)
}

async fn send_reply(ctx: &Context, cmd: &CommandInteraction, metadata: AuxMetadata) {
    let song_title = metadata
        .title
        .unwrap_or_else(|| "<unknown title>".to_string());

    let song_artist = metadata
        .artist
        .unwrap_or_else(|| "<unknown artist>".to_string());

    let source_url = metadata.source_url;
    let thumbnail_url = metadata.thumbnail;

    let message = source_url.map_or_else(
        || format!("**{song_title}** by **{song_artist}**"),
        |url| format!("[**{song_title}**]({url}) by **{song_artist}**"),
    );
    let embed = CreateEmbed::new()
        .title("🎶  Now Playing  🎶")
        .description(message)
        .color(Color::from_rgb(34, 255, 253))
        .thumbnail(thumbnail_url.unwrap_or_default());

    let _ = cmd
        .edit_response(&ctx.http, EditInteractionResponse::new().add_embed(embed))
        .await;
}
