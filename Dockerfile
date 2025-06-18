FROM rust:slim-bookworm AS backend-builder

WORKDIR /build

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libgstreamer-plugins-base1.0-dev && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY bilive/ bilive/

RUN cargo build --release

FROM node:24-bookworm-slim AS frontend-builder

WORKDIR /build
COPY web/ ./
RUN yarn install && yarn add -D vite
RUN npx vite build

FROM debian:bookworm-slim AS final

WORKDIR /

RUN apt-get update && \
    apt-get install -y openssl ca-certificates nginx ffmpeg \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev && \
    rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /build/target/release/clipchan /
COPY --from=frontend-builder /build/dist /var/www/html

RUN mkdir -p /data/clips /data/temp /config
COPY clipchan.toml /config

RUN echo 'server { \
    listen 80; \
    root /var/www/html; \
    index index.html; \
    location /api/ { \
        proxy_pass http://localhost:3000/; \
        proxy_set_header Host $host; \
        proxy_set_header X-Real-IP $remote_addr; \
    } \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
}' > /etc/nginx/sites-available/default

RUN echo '#!/bin/bash\n\
service nginx start\n\
./clipchan -c config/clipchan.toml\n\
' > /start.sh && chmod +x /start.sh

ENV RUST_LOG="info,sea_orm=warn,sqlx=warn"

# 暴露端口
EXPOSE 80 3000

# 启动应用
CMD ["/start.sh"]
