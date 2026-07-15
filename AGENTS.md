# Acoustic Bot

Discord music bot using serenity 0.12 + songbird 0.6. Plays YouTube audio via yt-dlp.

## Commands

slash commands, registered globally on `ready`:
- `/play <query>` — YouTube URL or search query; joins VC, enqueues track
- `/skip` — skip current track
- `/clearqueue` — clear queue (current track keeps playing)

## Build & Run

```sh
cargo build
cargo run
docker build -t acoustic-bot .
```

System deps: `libopus`, `openssl`, `pkg-config`, `ffmpeg`, `yt-dlp`

## Repo structure

```
src/
  main.rs          — entrypoint, tracing, client setup
  dotenv.rs        — loads .env (debug builds only)
  context.rs       — reqwest HttpClient in serenity type map
  handlers/
    command_handler.rs  — slash command dispatch
    voice_handler.rs    — cleanup songbird manager on bot VC leave
    track_end_handler.rs — songbird track-end event (no-op, TODO)
  commands/
    mod.rs  — shared check_user_channel helper
    play.rs
    skip.rs
    clear_queue.rs
```

## Key facts

- `.env` loading is `#[cfg(debug_assertions)]` only — release builds must set `BOT_TOKEN` via env
- `clearqueue` stops all but the currently playing track (dequeues from tail)
- `TrackEndHandler.act` is a no-op (disconnect logic commented out)
- No tests — no `tests/` dir or test modules
- No CI workflows
- `Rust edition 2024`, clippy pedantic + `unwrap_used = warn`
- `tokio` multi-thread runtime, graceful shutdown on `ctrl+c`
