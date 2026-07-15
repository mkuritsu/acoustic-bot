use anyhow::Context as _;
use serenity::all::{ChannelId, CommandInteraction, Context, GuildId};

pub mod clear_queue;
pub mod play;
pub mod queue;
pub mod skip;

async fn check_user_channel(
    ctx: &Context,
    cmd: &CommandInteraction,
) -> anyhow::Result<(GuildId, ChannelId)> {
    let guild_id = cmd
        .guild_id
        .context("You need to be on a server to use this command!")?;

    let channel_id = guild_id
        .get_user_voice_state(&ctx.http, cmd.user.id)
        .await
        .ok()
        .and_then(|voice_state| voice_state.channel_id)
        .context("You need to be connected to a voice channel to use this command!")?;

    Ok((guild_id, channel_id))
}
