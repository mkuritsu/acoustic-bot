use anyhow::Context as _;
use serenity::all::{
    Color, CommandInteraction, Context, CreateCommand, CreateEmbed,
    CreateInteractionResponseMessage,
};

use crate::{commands::check_user_channel, queue_store};

pub fn create() -> CreateCommand {
    CreateCommand::new("clearqueue").description("skip to the next song in the queue")
}

pub async fn execute(ctx: &Context, cmd: &CommandInteraction) -> anyhow::Result<()> {
    let (guild_id, _) = check_user_channel(ctx, cmd).await?;

    let handler = songbird::get(ctx)
        .await
        .expect("Should have songbird instance")
        .get(guild_id)
        .context("I'm currently not playing in any voice channel!")?;
    let handler = handler.lock().await;

    let queue = handler.queue();
    while queue.len() > 1 {
        let song = queue.dequeue(queue.len() - 1);
        song.and_then(|s| s.stop().ok());
    }

    queue_store::keep_first(guild_id);

    let embed = CreateEmbed::new()
        .description("Queue has been cleared")
        .color(Color::from_rgb(34, 255, 253));
    let _ = cmd
        .create_response(
            &ctx,
            serenity::all::CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().add_embed(embed),
            ),
        )
        .await;

    Ok(())
}
