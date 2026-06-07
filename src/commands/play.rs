use serenity::all::{
    Color, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse, ResolvedValue,
};
use songbird::input::YoutubeDl;
use tracing::trace;

use crate::HttpKey;

pub fn create() -> CreateCommand {
    CreateCommand::new("play")
        .description("play some music!")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "url", "the youtube url")
                .required(true),
        )
}

pub async fn execute(ctx: &Context, cmd: &CommandInteraction) {
    let manager = songbird::get(ctx)
        .await
        .expect("Should have songbird instance");
    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Should have httpclient")
    };

    let Some(guild_id) = cmd.guild_id else {
        let _ = send_response(
            cmd,
            ctx,
            error_embed("You need to be on a server to use this command!"),
        )
        .await;
        return;
    };

    let Ok(voice_state) = guild_id.get_user_voice_state(&ctx.http, cmd.user.id).await else {
        let _ = send_response(
            cmd,
            ctx,
            error_embed("You need to be connected to a voice channel to use this command!"),
        )
        .await;
        return;
    };
    let Some(channel_id) = voice_state.channel_id else {
        let _ = send_response(
            cmd,
            ctx,
            error_embed("You need to be connected to a voice channel to use this command!"),
        )
        .await;
        return;
    };

    let options = cmd.data.options();
    let Some(url) = options.first() else {
        let _ = send_response(cmd, ctx, error_embed("You need to provide an url!")).await;
        return;
    };
    let ResolvedValue::String(url) = url.value else {
        let _ = send_response(cmd, ctx, error_embed("Invalid url type!")).await;
        return;
    };
    trace!("Received play for {url} in {channel_id}");
    let _ = send_defer_response(cmd, ctx).await;

    let Ok(handler_lock) = manager.join(guild_id, channel_id).await else {
        let _ = send_response(cmd, ctx, error_embed("Could not get channel info!")).await;
        return;
    };

    let mut handler = handler_lock.lock().await;
    let mut yt_source = YoutubeDl::new(http_client, String::from(url));

    let Ok(outputs) = yt_source.query(1).await else {
        let _ = send_response(cmd, ctx, error_embed("Couldn't find the requested song!")).await;
        return;
    };
    let Some(output) = outputs.first() else {
        let _ = send_response(cmd, ctx, error_embed("Couldn't find the requested song!")).await;
        return;
    };

    let _ = handler.play_input(yt_source.clone().into());
    let _ = send_edit_response(
        cmd,
        ctx,
        success_embed(
            "Now Playing!",
            format!(
                "I will now play: {}",
                output.title.clone().unwrap_or(String::from("<untitled>"))
            ),
        ),
    )
    .await;
}

fn success_embed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(Color::DARK_GREEN)
}

fn error_embed(description: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title("ERROR")
        .description(description)
        .color(Color::RED)
}

async fn send_response(
    cmd: &CommandInteraction,
    ctx: &Context,
    embed: CreateEmbed,
) -> serenity::Result<()> {
    cmd.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().add_embed(embed),
        ),
    )
    .await?;
    Ok(())
}

async fn send_edit_response(
    cmd: &CommandInteraction,
    ctx: &Context,
    embed: CreateEmbed,
) -> serenity::Result<()> {
    cmd.edit_response(&ctx.http, EditInteractionResponse::new().add_embed(embed))
        .await?;
    Ok(())
}

async fn send_defer_response(cmd: &CommandInteraction, ctx: &Context) -> serenity::Result<()> {
    cmd.create_response(
        &ctx.http,
        CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
    )
    .await?;
    Ok(())
}
