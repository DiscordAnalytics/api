# ── Chef Stage ────────────
FROM rust:1-alpine AS chef

RUN apk add --no-cache \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    build-base \
    clang \
    lld \
    musl-dev

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN cargo install --locked cargo-chef

WORKDIR /app

# ── Planning Stage ────────
FROM chef AS planner

COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock ./

RUN cargo chef prepare --recipe-path recipe.json

# ── Build Stage ───────────
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --all-features --recipe-path recipe.json

COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# ── Final Build Stage ─────
FROM builder AS final-build

ARG BUILD_ARGS="--features=full"
RUN cargo build --release ${BUILD_ARGS} --bin discord-analytics-api

# ── Runtime Stage ─────────
FROM alpine:3 AS runtime

LABEL maintainer="Discord Analytics"
LABEL org.opencontainers.image.title="Discord Analytics API"
LABEL org.opencontainers.image.description="Official DiscordAnalytics API docker image"
LABEL org.opencontainers.image.source="https://github.com/DiscordAnalytics/api"
LABEL org.opencontainers.image.vendor="Discord Analytics"
LABEL org.opencontainers.image.licenses="AGPL"

RUN apk add --no-cache libssl3 ca-certificates curl

WORKDIR /app

COPY --from=final-build /app/target/release/discord-analytics-api /usr/local/bin

EXPOSE 3001

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["sh", "-c", "curl -fs http://localhost:3001/health | grep -qv 'degraded'"]

CMD ["/usr/local/bin/discord-analytics-api"]
