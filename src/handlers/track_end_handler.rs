use std::sync::Arc;

use serenity::{all::GuildId, async_trait};
use songbird::{
    Event, EventContext, EventHandler as SongbirdEventHandler, Songbird, tracks::PlayMode,
};
use tracing::info;

pub struct TrackEndHandler {
    pub guild_id: GuildId,
    pub manager: Arc<Songbird>,
}

#[async_trait]
impl SongbirdEventHandler for TrackEndHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(tracks) = ctx {
            let all_ended = tracks
                .iter()
                .all(|(state, _)| state.playing == PlayMode::End);
            if all_ended {
                // TODO: would be better to wait for some time before disconnecting, still need to figure out how to do that
                info!("All tracks have ended, disconnecting...");
                self.manager.remove(self.guild_id).await.ok();
            }
        }
        None
    }
}
