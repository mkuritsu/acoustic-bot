use anyhow::Context as _;
use serenity::all::{
    Color, CommandInteraction, Context, CreateCommand, CreateEmbed,
    CreateInteractionResponseMessage,
};

use crate::commands::check_user_channel;

pub fn create() -> CreateCommand {
    CreateCommand::new("skip").description("skip to the next song in the queue")
}

pub async fn execute(ctx: &Context, cmd: &CommandInteraction) -> anyhow::Result<()> {
    let (guild_id, _) = check_user_channel(ctx, cmd).await?;
    println!("skip received");

    let handler = songbird::get(ctx)
        .await
        .expect("Should have songbird instance")
        .get(guild_id)
        .context("I'm currently not playing in any voice channel!")?;
    let handler = handler.lock().await;
    println!("locked");

    let queue = handler.queue();

    queue.current().context("No track playing!")?;
    queue.skip().context("Failed to skip song!")?;

    println!("called skip");

    let embed = CreateEmbed::new()
        .description("Song has been skipped")
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
