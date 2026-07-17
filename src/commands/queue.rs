use std::{fmt::Write, time::Duration};

use serenity::all::{
    Color, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateEmbedAuthor,
    CreateInteractionResponseMessage,
};

use crate::{commands::check_user_channel, queue_store};

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
        let first = &tracks[0];
        let display_name = first
            .user
            .global_name
            .as_deref()
            .unwrap_or(&first.user.name);

        let mut description = String::new();
        description.push_str("**Now Playing**\n");

        for (i, item) in tracks.iter().enumerate() {
            if i > 0 && i == 1 {
                description.push_str("\n**Up Next**\n");
            }

            let submitter = item
                .user
                .global_name
                .as_deref()
                .unwrap_or(&item.user.name);

            let _ = writeln!(
                description,
                "{}. {} ({}) — {submitter}",
                i + 1,
                item.title,
                format_duration(item.duration.as_ref()),
            );
        }

        let mut embed = CreateEmbed::new()
            .author(CreateEmbedAuthor::new(display_name).icon_url(first.user.face()))
            .description(description)
            .color(Color::from_rgb(34, 255, 253));

        if let Some(ref thumb) = first.thumbnail {
            embed = embed.thumbnail(thumb);
        }

        embed
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
