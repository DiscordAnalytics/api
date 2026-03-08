# ── Build Stage ──────────────────────────────────────────────────────────────
FROM rust:1-alpine AS build

RUN apk add --no-cache \
        pkgconfig \
        openssl-dev \
        build-base \
        clang \
        lld \
        musl-dev

WORKDIR /usr/src/discord-analytics-api

COPY Cargo.toml Cargo.lock ./
COPY src ./src

ARG BUILD_ARGS=""

RUN cargo build --release ${BUILD_ARGS}

#── Runtime Stage ─────────────────────────────────────────────────────────────
FROM alpine:3 AS final

LABEL maintainer="Discord Analytics"
LABEL org.opencontainers.image.title="Discord Analytics API"
LABEL org.opencontainers.image.description="Official DiscordAnalytics API docker image"
LABEL org.opencontainers.image.source="https://github.com/DiscordAnalytics/api"
LABEL org.opencontainers.image.vendor="Discord Analytics"
LABEL org.opencontainers.image.licenses="AGPL"

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=build /usr/src/discord-analytics-api/target/release/discord-analytics-api ./discord-analytics-api
# COPY ./templates ./templates

EXPOSE 3001

CMD ["./discord-analytics-api"]
