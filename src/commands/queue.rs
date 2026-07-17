use std::time::Duration;

use serenity::all::{
    Color, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateEmbedAuthor,
    CreateInteractionResponseMessage,
};

use crate::{commands::check_user_channel, queue_store};

const MAX_EMBEDS: usize = 10;

pub fn create() -> CreateCommand {
    CreateCommand::new("queue").description("display the current song queue")
}

pub async fn execute(ctx: &Context, cmd: &CommandInteraction) -> anyhow::Result<()> {
    let (guild_id, _) = check_user_channel(ctx, cmd).await?;

    let tracks = queue_store::get(guild_id);

    let mut resp = CreateInteractionResponseMessage::new();

    if tracks.is_empty() {
        resp = resp.add_embed(
            CreateEmbed::new()
                .title("Queue")
                .description("Queue is empty")
                .color(Color::from_rgb(34, 255, 253)),
        );
    } else {
        for (i, item) in tracks.iter().enumerate().take(MAX_EMBEDS) {
            let display_name = item
                .user
                .global_name
                .as_deref()
                .unwrap_or(&item.user.name);

            let mut embed = CreateEmbed::new()
                .author(CreateEmbedAuthor::new(display_name).icon_url(item.user.face()))
                .description(format!(
                    "**{}** ({})",
                    item.title,
                    format_duration(item.duration.as_ref()),
                ))
                .color(Color::from_rgb(34, 255, 253));

            if i == 0 && let Some(ref thumb) = item.thumbnail {
                embed = embed.thumbnail(thumb);
            }

            resp = resp.add_embed(embed);
        }

        if tracks.len() > MAX_EMBEDS {
            resp = resp.add_embed(
                CreateEmbed::new()
                    .description(format!("... and {} more", tracks.len() - MAX_EMBEDS))
                    .color(Color::from_rgb(34, 255, 253)),
            );
        }
    }

    let _ = cmd
        .create_response(
            &ctx,
            serenity::all::CreateInteractionResponse::Message(resp),
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
