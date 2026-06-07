use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse, ResolvedValue,
};
use songbird::input::YoutubeDl;

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
                CreateInteractionResponseMessage::new()
                    .add_embed(CreateEmbed::new().description("Cloud not get channel info!")),
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
            EditInteractionResponse::new().add_embed(CreateEmbed::new().description(format!(
                "Started playing {}!",
                output.title.clone().unwrap_or(String::from("?"))
            ))),
        )
        .await;
}
