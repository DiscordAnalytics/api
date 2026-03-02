# ── Build Stage ──────────────────────────────────────────────────────────────
FROM rust:1-bullseye AS build

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        g++ \
        lld \
        clang \
        musl-tools \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

COPY . .

ARG BUILD_ARGS=""

RUN cargo build --release --target x86_64-unknown-linux-musl ${BUILD_ARGS}

FROM alpine:3 AS final

LABEL maintainer="Discord Analytics"
LABEL org.opencontainers.image.title="Discord Analytics API"
LABEL org.opencontainers.image.description="Official DiscordAnalytics API docker image"
LABEL org.opencontainers.image.source="https://github.com/DiscordAnalytics/api"
LABEL org.opencontainers.image.vendor="Discord Analytics"
LABEL org.opencontainers.image.licenses="AGPL"

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=build /usr/src/app/target/x86_64-unknown-linux-musl/release/discord-analytics-api ./discord-analytics-api
# COPY ./templates ./templates

EXPOSE 3001

CMD ["./discord-analytics-api"]