use serenity::{
    all::{Context, EventHandler, VoiceState},
    async_trait,
};
use tracing::info;

use crate::queue_store;

pub struct VoiceHandler;

#[async_trait]
impl EventHandler for VoiceHandler {
    async fn voice_state_update(&self, ctx: Context, _: Option<VoiceState>, new: VoiceState) {
        let bot_id = ctx.cache.current_user().id;
        if new.user_id != bot_id {
            return;
        }

        let Some(guild_id) = new.guild_id else {
            return;
        };

        let manager = songbird::get(&ctx)
            .await
            .expect("Should have songbird instance");
        if new.channel_id.is_none() && manager.get(guild_id).is_some() {
            manager.remove(guild_id).await.ok();
            queue_store::remove(guild_id);
            info!("removing manager for guild {guild_id}");
        }
    }
}
