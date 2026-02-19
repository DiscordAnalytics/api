use std::{env, sync::OnceLock};

use anyhow::{Error, Result, anyhow};
use dotenvy::dotenv;

#[derive(Debug)]
pub struct EnvConfig {
    // Misc
    pub port: u16,
    pub api_url: String,
    pub client_url: String,
    pub admins: Vec<String>,

    // Database
    pub database_url: String,

    // OpenTelemetry
    pub otlp_endpoint: Option<String>,
    pub otlp_token: Option<String>,
    pub otlp_stream: Option<String>,

    // Tokens
    pub discord_token: String,
    pub jwt_secret: String,

    // Linked Roles
    pub client_secret: String,
    pub client_id: String,

    // Mail
    pub smtp: String,
    pub smtp_mail: String,
    pub smtp_user: String,
    pub smtp_password: String,

    // R2
    pub r2_bucket_name: String,
    pub r2_account_id: String,
    pub r2_public_bucket_endpoint: String,
    pub cloudflare_id: String,
    pub cloudflare_token: String,
}

pub static ENV: OnceLock<EnvConfig> = OnceLock::new();

fn get_var(key: &str) -> Result<String> {
    env::var(key)
        .map_err(|e| Error::new(e).context(format!("Environment variable {} not set", key)))
}

pub fn init_env() -> Result<&'static EnvConfig> {
    dotenv().ok();

    if let Some(config) = ENV.get() {
        return Ok(config);
    }

    let port = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3001);
    let api_url = env::var("API_URL").unwrap_or_else(|_| format!("http://localhost:{}", port));
    let client_url = get_var("CLIENT_URL")?;
    let admins = env::var("ADMINS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let database_url = get_var("DATABASE_URL")?;

    let (otlp_endpoint, otlp_token, otlp_stream) = match (
        get_var("OTLP_ENDPOINT"),
        get_var("OTLP_TOKEN"),
        get_var("OTLP_STREAM"),
    ) {
        (Ok(endpoint), Ok(token), Ok(stream)) => (Some(endpoint), Some(token), Some(stream)),
        (Err(_), Err(_), Err(_)) => (None, None, None),
        _ => {
            return Err(anyhow!(
                "One of these env vars are missing: OTLP_ENDPOINT, OTLP_TOKEN or OTLP_STREAM"
            ));
        }
    };

    let discord_token = get_var("DISCORD_TOKEN")?;
    let jwt_secret = get_var("JWT_SECRET")?;

    let client_secret = get_var("CLIENT_SECRET")?;
    let client_id = get_var("CLIENT_ID")?;

    let smtp = get_var("SMTP")?;
    let smtp_mail = get_var("SMTP_MAIL")?;
    let smtp_user = get_var("SMTP_USER")?;
    let smtp_password = get_var("SMTP_PASSWORD")?;

    let r2_bucket_name = get_var("R2_BUCKET_NAME")?;
    let r2_account_id = get_var("R2_ACCOUNT_ID")?;
    let r2_public_bucket_endpoint = get_var("R2_PUBLIC_BUCKET_ENDPOINT")?;
    let cloudflare_id = get_var("CLOUDFLARE_ID")?;
    let cloudflare_token = get_var("CLOUDFLARE_TOKEN")?;

    Ok(ENV.get_or_init(|| EnvConfig {
        port,
        api_url,
        client_url,
        admins,
        database_url,
        otlp_endpoint,
        otlp_token,
        otlp_stream,
        discord_token,
        jwt_secret,
        client_secret,
        client_id,
        smtp,
        smtp_mail,
        smtp_user,
        smtp_password,
        r2_bucket_name,
        r2_account_id,
        r2_public_bucket_endpoint,
        cloudflare_id,
        cloudflare_token,
    }))
}

#[macro_export]
macro_rules! app_env {
    () => {
        $crate::config::env::ENV
            .get()
            .expect("Environment not initialized. Call init_env() first.")
    };
}
