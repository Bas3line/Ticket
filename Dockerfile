FROM rust:1-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY templates ./templates

RUN cargo build --release && \
    strip target/release/ticket-bot

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/ticket-bot /app/ticket-bot
COPY --from=builder /app/migrations /app/migrations

ENV RUST_LOG=info

EXPOSE 8080

CMD ["/app/ticket-bot"]
