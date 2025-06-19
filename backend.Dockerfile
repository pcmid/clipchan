FROM rust:slim-bookworm AS backend-builder

WORKDIR /build

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libgstreamer-plugins-base1.0-dev && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY bilive/ bilive/

RUN cargo build --release

FROM debian:bookworm-slim AS server

WORKDIR /

RUN sed -i 's/Components: main/Components: main contrib non-free/' /etc/apt/sources.list.d/debian.sources && \
    apt-get update && \
    apt-get install -y openssl ca-certificates nginx ffmpeg \
     gstreamer1.0-plugins-good \
     gstreamer1.0-plugins-bad \
     gstreamer1.0-plugins-ugly \
     gstreamer1.0-libav \
     gstreamer1.0-fdkaac && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean &&  \
    apt-get autoclean && \
    apt-get autoremove -y

COPY --from=backend-builder /build/target/release/clipchan /
RUN mkdir -p /data /data/temp /config
COPY clipchan.toml /config
EXPOSE 3000

# 启动应用
CMD ["/clipchan", "-c", "/config/clipchan.toml"]
