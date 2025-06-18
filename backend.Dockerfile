FROM rust:slim-bookworm AS backend-builder

WORKDIR /app

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libgstreamer-plugins-base1.0-dev && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY bilive/ bilive/

RUN cargo build --release

FROM debian:bookworm-slim AS server

WORKDIR /app

RUN apt-get update && \
    apt-get install -y openssl ca-certificates nginx ffmpeg \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev && \
    rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/target/release/clipchan /app/
RUN mkdir -p /data /data/temp /config
COPY clipchan.toml /config
EXPOSE 3000

# 启动应用
CMD ["/app/clipchan", "-c", "/config/clipchan.toml"]
