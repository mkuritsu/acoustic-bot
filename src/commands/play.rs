use std::time::Duration;

use anyhow::Context as _;
use serenity::all::{
    ChannelId, Color, CommandInteraction, CommandOptionType, Context, CreateCommand,
    CreateCommandOption, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse, GuildId, ResolvedValue, User,
};
use songbird::{
    Event, TrackEvent,
    input::{AuxMetadata, Compose, YoutubeDl},
};
use tracing::{info, trace};

use crate::{
    commands::check_user_channel, context::ContextHttpClientExt,
    handlers::track_end_handler::TrackEndHandler,
    queue_store::{self, TrackMetadata},
};

pub fn create() -> CreateCommand {
    CreateCommand::new("play")
        .description("play some music! supports youtube urls, search, and playlists")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "query",
                "youtube url, search query, or playlist url",
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

    let (metadata, started_now) = if is_playlist_url(query) {
        start_playlist(ctx, query, guild_id, channel_id, cmd.user.clone()).await?
    } else {
        start_single(ctx, query, guild_id, channel_id, cmd.user.clone()).await?
    };
    send_reply(ctx, cmd, metadata, started_now).await;

    Ok(())
}

fn is_playlist_url(query: &str) -> bool {
    query.contains("list=") || query.contains("/playlist")
}

async fn start_playlist(
    ctx: &Context,
    url: &str,
    guild_id: GuildId,
    channel_id: ChannelId,
    user: User,
) -> anyhow::Result<(AuxMetadata, bool)> {
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

    let will_play_now = handler.queue().is_empty() && handler.queue().current().is_none();

    let items = fetch_playlist_items(url).await?;
    let total = items.len();
    let mut first_meta = None;

    for (i, item) in items.iter().enumerate() {
        let video_url = format!("https://www.youtube.com/watch?v={}", item.id);
        let mut yt_source = YoutubeDl::new(http_client.clone(), video_url);

        let (track_title, track_duration, track_thumbnail) = if i == 0 {
            let meta = yt_source.aux_metadata().await?;
            first_meta = Some(meta.clone());
            let title = meta
                .title
                .clone()
                .unwrap_or_else(|| item.title.clone());
            let duration = meta.duration.or(item.duration.map(Duration::from_secs_f64));
            let thumbnail = meta.thumbnail.clone().or_else(|| item.thumbnail.clone());
            (title, duration, thumbnail)
        } else {
            let duration = item.duration.map(Duration::from_secs_f64);
            (item.title.clone(), duration, item.thumbnail.clone())
        };

        let track = if i == 0 {
            handler.enqueue_with_preload(yt_source.into(), Some(Duration::from_secs(20)))
        } else {
            handler.enqueue(yt_source.into()).await
        };
        track
            .add_event(
                Event::Track(TrackEvent::End),
                TrackEndHandler {
                    guild_id,
                    manager: manager.clone(),
                },
            )
            .ok();

        queue_store::push(
            guild_id,
            TrackMetadata {
                title: track_title,
                duration: track_duration,
                thumbnail: track_thumbnail,
                user: user.clone(),
            },
        );

        info!("enqueued ({i}/{total}): {}", item.title);
    }

    Ok((first_meta.expect("playlist had at least one item"), will_play_now))
}

async fn start_single(
    ctx: &Context,
    query: &str,
    guild_id: GuildId,
    channel_id: ChannelId,
    user: User,
) -> anyhow::Result<(AuxMetadata, bool)> {
    let manager = songbird::get(ctx)
        .await
        .expect("Should have songbird instance");
    let http_client = ctx.get_http_client().await;

    let mut yt_source = if query.starts_with("https://") || query.starts_with("http://") {
        YoutubeDl::new(http_client, String::from(query))
    } else {
        YoutubeDl::new_search(http_client, String::from(query))
    };

    // run VC join and yt-dlp metadata fetch in parallel
    let join_fut = manager.join(guild_id, channel_id);
    let meta_fut = yt_source.aux_metadata();
    let (join_result, meta_result) = tokio::join!(join_fut, meta_fut);

    let handler_arc = join_result.context("Could not get channel info!")?;
    let mut handler = handler_arc.lock().await;
    handler.deafen(true).await.ok();

    let will_play_now = handler.queue().is_empty() && handler.queue().current().is_none();

    let metadata = meta_result.context("Song not found!")?;
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

    queue_store::push(
        guild_id,
        TrackMetadata {
            title: metadata
                .title
                .clone()
                .unwrap_or_else(|| "<unknown title>".to_string()),
            duration: metadata.duration,
            thumbnail: metadata.thumbnail.clone(),
            user,
        },
    );

    Ok((metadata, will_play_now))
}

async fn fetch_playlist_items(url: &str) -> anyhow::Result<Vec<PlaylistItem>> {
    let output = tokio::process::Command::new("yt-dlp")
        .args(["--flat-playlist", "-j"])
        .arg(url)
        .output()
        .await
        .context("Failed to run yt-dlp — is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp failed to fetch playlist: {stderr}");
    }

    let stdout = String::from_utf8(output.stdout)?;
    let mut items = Vec::new();

    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_str(line).context("Failed to parse yt-dlp output")?;

        let id = value["id"]
            .as_str()
            .context("Video missing id in yt-dlp output")?
            .to_string();
        let title = value["title"].as_str().unwrap_or("<unknown>").to_string();
        let duration = value["duration"].as_f64();
        let thumbnail = value["thumbnail"].as_str().map(String::from);

        items.push(PlaylistItem {
            id,
            title,
            duration,
            thumbnail,
        });
    }

    anyhow::ensure!(!items.is_empty(), "No videos found in playlist");

    Ok(items)
}

struct PlaylistItem {
    id: String,
    title: String,
    duration: Option<f64>,
    thumbnail: Option<String>,
}

async fn send_reply(
    ctx: &Context,
    cmd: &CommandInteraction,
    metadata: AuxMetadata,
    started_now: bool,
) {
    let song_title = metadata
        .title
        .unwrap_or_else(|| "<unknown title>".to_string());

    let song_artist = metadata
        .artist
        .unwrap_or_else(|| "<unknown artist>".to_string());

    let source_url = metadata.source_url;
    let thumbnail_url = metadata.thumbnail;

    let embed_title = if started_now {
        "🎶  Now Playing  🎶"
    } else {
        "🎶  Queued  🎶"
    };

    let message = source_url.map_or_else(
        || format!("**{song_title}** by **{song_artist}**"),
        |url| format!("[**{song_title}**]({url}) by **{song_artist}**"),
    );

    let embed = CreateEmbed::new()
        .title(embed_title)
        .description(message)
        .color(Color::from_rgb(34, 255, 253))
        .thumbnail(thumbnail_url.unwrap_or_default());

    let _ = cmd
        .edit_response(&ctx.http, EditInteractionResponse::new().add_embed(embed))
        .await;
}
