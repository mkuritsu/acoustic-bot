use std::{sync::Arc, time::Duration};

use serenity::{all::GuildId, async_trait};
use songbird::{Event, EventContext, EventHandler as SongbirdEventHandler, Songbird};
use tracing::info;

use crate::queue_store;

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
                .all(|(state, _)| state.playing == songbird::tracks::PlayMode::End);
            if all_ended {
                queue_store::pop_front(self.guild_id);

                let handler_lock = self.manager.get(self.guild_id)?;
                let handler = handler_lock.lock().await;
                if handler.queue().is_empty() && handler.queue().current().is_none() {
                    info!("queue empty, will disconnect after 30s of silence");
                    drop(handler);
                    let manager = self.manager.clone();
                    let guild_id = self.guild_id;
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        if let Some(handler_lock) = manager.get(guild_id) {
                            let handler = handler_lock.lock().await;
                            if handler.queue().is_empty() && handler.queue().current().is_none() {
                                info!("no new tracks after 30s, disconnecting");
                                drop(handler);
                                manager.remove(guild_id).await.ok();
                            }
                        }
                    });
                }
            }
        }
        None
    }
}
