# --- Stage 1: Builder ---
FROM rust:1.91-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release --bin pd

# --- Stage 2: Runtime ---
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pd /usr/local/bin/pd

RUN mkdir -p /app/downloads /root/.config/pd

EXPOSE 9090

ENV PD__SERVER_ADDR=0.0.0.0
ENV PD__DEFAULT_DIR=/app/downloads

CMD ["pd", "start"]
