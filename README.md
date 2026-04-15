# DiscordAnalytics API

Official REST API for [DiscordAnalytics](https://discordanalytics.xyz) — a platform for tracking and analyzing Discord bot statistics, votes, users, and teams.

Built with [Rust](https://www.rust-lang.org/), [Actix-Web](https://actix.rs/), and [MongoDB](https://www.mongodb.com/).

---

## Table of Contents

- [Requirements](#requirements)
- [Getting Started](#getting-started)
- [Running the API](#running-the-api)
- [Authentication](#authentication)

---

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [MongoDB](https://www.mongodb.com/) instance
- A Discord application (for OAuth2)
- A Redis server (for rate limiting)
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

## Authentication

The API uses three types of authentication, passed via the `Authorization` header:

| Type    | Header format             | Description                                     |
|---------|---------------------------|-------------------------------------------------|
| `Admin` | `Admin <jwt_access_token>`| Admin user (must be listed in `ADMINS` env var) |
| `User`  | `User <jwt_access_token>` | Authenticated dashboard user                    |
| `Bot`   | `Bot <bot_token>`         | A registered Discord bot                        |

Access tokens expire after **30 minutes**. Refresh tokens expire after **30 days** and can be exchanged for a new access token at `POST /auth/refresh`.

---
