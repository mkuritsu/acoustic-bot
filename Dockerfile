FROM ubuntu:26.04 AS builder

RUN apt-get update && apt-get install -y libssl-dev pkg-config libopus-dev cargo

WORKDIR /app

COPY Cargo.toml Cargo.lock .
RUN mkdir -p ./src && touch src/lib.rs
RUN cargo build --release --locked

COPY src ./src
RUN cargo build --release --locked

FROM ubuntu:26.04

RUN apt-get update && apt-get install -y yt-dlp ffmpeg

COPY --from=builder /app/target/release/acoustic-bot /usr/bin/acoustic-bot

CMD ["acoustic-bot"]
