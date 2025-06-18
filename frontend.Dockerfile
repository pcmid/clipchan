FROM node:24-bookworm-slim AS frontend-builder

WORKDIR /build
COPY web/ ./
RUN yarn install && yarn add -D vite
RUN npx vite build

FROM debian:bookworm-slim AS web

RUN apt-get update && \
    apt-get install -y ca-certificates nginx && \
    rm -rf /var/lib/apt/lists/*

COPY --from=frontend-builder /build/dist /var/www/html

RUN echo 'server { \
    listen 80; \
    root /var/www/html; \
    index index.html; \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
}' > /etc/nginx/sites-available/default

# 暴露端口
EXPOSE 80

# 启动应用
CMD ["nginx", "-g", "daemon off;"]
