use std::{fmt::Write, time::Duration};

use serenity::all::{
    Color, CommandInteraction, Context, CreateCommand, CreateEmbed,
    CreateInteractionResponseMessage,
};

use crate::{
    commands::check_user_channel,
    queue_store,
};

pub fn create() -> CreateCommand {
    CreateCommand::new("queue").description("display the current song queue")
}

pub async fn execute(ctx: &Context, cmd: &CommandInteraction) -> anyhow::Result<()> {
    let (guild_id, _) = check_user_channel(ctx, cmd).await?;

    let tracks = queue_store::get(guild_id);

    let embed = if tracks.is_empty() {
        CreateEmbed::new()
            .title("Queue")
            .description("Queue is empty")
            .color(Color::from_rgb(34, 255, 253))
    } else {
        let mut description = String::new();
        for (i, item) in tracks.iter().enumerate() {
            if i == 0 {
                description.push_str("**Now Playing**\n");
            } else if i == 1 {
                description.push_str("\n**Up Next**\n");
            }

            let display_name = item
                .user
                .global_name
                .as_deref()
                .unwrap_or(&item.user.name);
            let _ = writeln!(
                description,
                "{}. {} ({}) — {display_name}",
                i + 1,
                item.title,
                format_duration(item.duration.as_ref()),
            );
        }

        let thumbnail = tracks[0].user.face();

        CreateEmbed::new()
            .title("Queue")
            .description(description)
            .color(Color::from_rgb(34, 255, 253))
            .thumbnail(thumbnail)
    };

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

fn format_duration(dur: Option<&Duration>) -> String {
    match dur {
        Some(d) => {
            let s = d.as_secs();
            let hours = s / 3600;
            let minutes = (s % 3600) / 60;
            let seconds = s % 60;
            if hours > 0 {
                format!("{hours}:{minutes:02}:{seconds:02}")
            } else {
                format!("{minutes}:{seconds:02}")
            }
        }
        None => "Unknown".to_string(),
    }
}
