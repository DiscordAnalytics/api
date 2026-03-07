# DiscordAnalytics API

Official REST API for [DiscordAnalytics](https://discordanalytics.xyz) — a platform for tracking and analyzing Discord bot statistics, votes, users, and teams.

Built with [Rust](https://www.rust-lang.org/), [Actix-Web](https://actix.rs/), and [MongoDB](https://www.mongodb.com/).

---

## Table of Contents

- [Requirements](#requirements)
- [Getting Started](#getting-started)
- [Environment Variables](#environment-variables)
- [Running the API](#running-the-api)
- [Architecture](#architecture)
- [Authentication](#authentication)

---

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [MongoDB](https://www.mongodb.com/) instance
- A Discord application (for OAuth2)
- An SMTP server (optional, for email notifications)
- A Cloudflare R2 bucket (optional, for file storage)
- An OpenTelemetry collector (optional, for telemetry data)

---

## Getting Started

**1. Clone the repository**

```sh
git clone https://github.com/DiscordAnalytics/api.git
cd api
```

**2. Copy the environment file and fill in the values**

```sh
cp .env.example .env
```

**3. Install dependencies and build**

```sh
cargo build
```

---

## Environment Variables

| Variable                    | Required | Default           | Description                                                              |
|-----------------------------|----------|-------------------|--------------------------------------------------------------------------|
| `PORT`                      | No       | `3001`            | Port the API will listen on                                              |
| `API_URL`                   | No       | `0.0.0.0:{PORT}`  | Public URL of the API                                                    |
| `CLIENT_URL`                | **Yes**  | —                 | URL of the frontend client (used for CORS and OAuth redirects)           |
| `ADMINS`                    | No       | —                 | Comma-separated list of Discord user IDs with admin privileges           |
| `DATABASE_URL`              | **Yes**  | —                 | MongoDB connection string                                                |
| `OTLP_ENDPOINT`             | No       | —                 | OpenTelemetry collector endpoint (all three OTLP vars must be set)       |
| `OTLP_TOKEN`                | No       | —                 | OpenTelemetry authentication token                                       |
| `OTLP_STREAM`               | No       | —                 | OpenTelemetry stream name                                                |
| `DISCORD_TOKEN`             | **Yes**  | —                 | Discord bot token                                                        |
| `JWT_SECRET`                | **Yes**  | —                 | Secret used to sign JWT tokens                                           |
| `ENABLE_REGISTRATIONS`      | No       | `true`            | Whether new user registrations are allowed (`true` or `1` to enable)     |
| `CLIENT_SECRET`             | **Yes**  | —                 | Discord OAuth2 client secret                                             |
| `CLIENT_ID`                 | **Yes**  | —                 | Discord OAuth2 client ID                                                 |
| `SMTP`                      | No       | —                 | SMTP server address                                                      |
| `SMTP_MAIL`                 | No       | —                 | Sender email address                                                     |
| `SMTP_USER`                 | No       | —                 | SMTP username                                                            |
| `SMTP_PASSWORD`             | No       | —                 | SMTP password                                                            |
| `R2_BUCKET_NAME`            | No       | —                 | Cloudflare R2 bucket name                                                |
| `R2_ACCOUNT_ID`             | No       | —                 | Cloudflare account ID                                                    |
| `R2_PUBLIC_BUCKET_ENDPOINT` | No       | —                 | Public URL of the R2 bucket                                              |
| `CLOUDFLARE_ID`             | No       | —                 | Cloudflare API ID                                                        |
| `CLOUDFLARE_TOKEN`          | No       | —                 | Cloudflare API token                                                     |

> **Note 1:** Required when using the `otel` feature, if any one of `OTLP_ENDPOINT`, `OTLP_TOKEN`, or `OTLP_STREAM` is set, all three must be provided.
> **Note 2:** Required when using the `mails` feature, if any of the `SMTP` variables are set, all of them must be provided to enable email notifications.
> **Note 3:** Required when using the `reports` feature, if any of the `R2` or `CLOUDFLARE` variables are set, all of them must be provided to enable Cloudflare R2 integration.

---

## Running the API

**Development**

The following `cargo` commands run the API with different feature sets. You can also combine features as needed (e.g. `cargo run --features "mails otel"`).

```sh
cargo minimal    # Runs the api without any features
cargo mails      # Runs the API with email notifications enabled
cargo otel       # Runs the API with OpenTelemetry enabled
cargo reports    # Runs the API with the reports feature enabled
cargo mails-otel # Runs the API with both email notifications and OpenTelemetry enabled
cargo full       # Runs the API with all features enabled
```

**Production**

```sh
cargo build --release
./target/release/discord-analytics-api
```

The API will start on `http://0.0.0.0:3001` by default. The OpenAPI specification is available at `/openapi.json`.

---

## Architecture

```
src/
├── api/
│   ├── middleware/       # Auth middleware and request extractors
│   └── routes/           # Route handlers grouped by resource
│       ├── achievements/
│       ├── auth/         # OAuth2 callback, token refresh, session management
│       ├── bots/         # Bot CRUD operations
│       ├── health/       # Health check endpoint
│       ├── invitations/  # Team invitations
│       ├── stats/        # Global statistics
│       ├── users/        # User CRUD and management
│       └── websocket/    # WebSocket endpoint
├── config/               # Environment configuration
├── domain/
│   ├── auth/             # JWT, token generation, auth context
│   ├── error.rs          # API error types
│   └── models/           # MongoDB document models
├── managers/
│   ├── chat.rs           # WebSocket chat server
│   └── webhook.rs        # Vote webhook delivery manager
├── openapi/              # OpenAPI spec builder
├── repository/           # MongoDB repository layer
├── services/             # Business logic layer
└── utils/                # Constants, logger, Discord utilities
```

---

## Authentication

The API uses three types of authentication, passed via the `Authorization` header:

| Type    | Header format             | Description                                     |
|---------|---------------------------|-------------------------------------------------|
| `Admin` | `Admin <jwt_access_token>`| Admin user (must be listed in `ADMINS` env var) |
| `User`  | `User <jwt_access_token>` | Authenticated dashboard user                    |
| `Bot`   | `Bot <bot_token>`         | A registered Discord bot                        |

Access tokens expire after **30 minutes**. Refresh tokens expire after **30 days** and can be exchanged for a new access token at `POST /auth/refresh`.

---
