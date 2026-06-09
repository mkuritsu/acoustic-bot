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

use crate::{context::ContextHttpClientExt, handlers::track_end_handler::TrackEndHandler};

pub fn create() -> CreateCommand {
    CreateCommand::new("play")
        .description("play some music!")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "url", "the youtube url")
                .required(true),
        )
}

pub async fn execute(ctx: &Context, cmd: &CommandInteraction) -> anyhow::Result<()> {
    let guild_id = cmd
        .guild_id
        .context("You need to be on a server to use this command!")?;

    let channel_id = guild_id
        .get_user_voice_state(&ctx.http, cmd.user.id)
        .await
        .ok()
        .and_then(|voice_state| voice_state.channel_id)
        .context("You need to be connected to a voice channel to use this command!")?;

    let url = cmd
        .data
        .options()
        .first()
        .cloned()
        .context("You need to provide an url!")?;
    let ResolvedValue::String(url) = url.value else {
        anyhow::bail!("Invalid url type!");
    };
    trace!("Received play for {url} in {channel_id}");

    let _ = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
        )
        .await;

    let metadata = start_playback(ctx, url, guild_id, channel_id).await?;
    send_reply(ctx, cmd, metadata).await;

    Ok(())
}

async fn start_playback(
    ctx: &Context,
    url: &str,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> anyhow::Result<Option<AuxMetadata>> {
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

    let mut yt_source = YoutubeDl::new(http_client, String::from(url));
    let track = handler.play_only_input(yt_source.clone().into());
    track
        .add_event(
            Event::Track(TrackEvent::End),
            TrackEndHandler {
                guild_id,
                manager: manager.clone(),
            },
        )
        .ok();

    let meta = yt_source.aux_metadata().await.ok();
    Ok(meta)
}

async fn send_reply(ctx: &Context, cmd: &CommandInteraction, metadata: Option<AuxMetadata>) {
    let song_title = metadata
        .as_ref()
        .and_then(|m| m.title.as_deref())
        .unwrap_or("<unknown title>");

    let song_artist = metadata
        .as_ref()
        .and_then(|m| m.artist.as_deref())
        .unwrap_or("<unknown artist>");

    let embed = CreateEmbed::new()
        .title("Now Playing!")
        .description(format!(
            "I will now play: **{song_title}** by **{song_artist}**"
        ))
        .color(Color::DARK_GREEN);

    let _ = cmd
        .edit_response(&ctx.http, EditInteractionResponse::new().add_embed(embed))
        .await;
}
