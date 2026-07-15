use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    time::Duration,
};

use serenity::all::{GuildId, User};

#[derive(Clone)]
pub struct TrackMetadata {
    pub title: String,
    pub duration: Option<Duration>,
    pub user: User,
}

static QUEUE_STORE: LazyLock<Mutex<HashMap<GuildId, Vec<TrackMetadata>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn push(guild_id: GuildId, metadata: TrackMetadata) {
    QUEUE_STORE
        .lock()
        .expect("queue store lock poisoned")
        .entry(guild_id)
        .or_default()
        .push(metadata);
}

pub fn pop_front(guild_id: GuildId) {
    QUEUE_STORE
        .lock()
        .expect("queue store lock poisoned")
        .entry(guild_id)
        .and_modify(|v| {
            if !v.is_empty() {
                v.remove(0);
            }
        });
}

pub fn keep_first(guild_id: GuildId) {
    QUEUE_STORE
        .lock()
        .expect("queue store lock poisoned")
        .entry(guild_id)
        .and_modify(|v| {
            if v.len() > 1 {
                v.truncate(1);
            }
        });
}

pub fn remove(guild_id: GuildId) {
    QUEUE_STORE
        .lock()
        .expect("queue store lock poisoned")
        .remove(&guild_id);
}

pub fn get(guild_id: GuildId) -> Vec<TrackMetadata> {
    QUEUE_STORE
        .lock()
        .expect("queue store lock poisoned")
        .get(&guild_id)
        .cloned()
        .unwrap_or_default()
}
