use std::{env, net::Ipv4Addr, sync::OnceLock};

use anyhow::{Error, Result};
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
    pub database_name: String,

    // Tokens
    pub discord_token: String,
    pub jwt_secret: String,
    pub enable_registrations: bool,

    // Linked Roles
    pub client_secret: String,
    pub client_id: String,

    // OpenTelemetry
    #[cfg(feature = "otel")]
    pub otlp_endpoint: String,
    #[cfg(feature = "otel")]
    pub otlp_token: String,
    #[cfg(feature = "otel")]
    pub otlp_stream: String,

    // Mail
    #[cfg(feature = "mails")]
    pub smtp: String,
    #[cfg(feature = "mails")]
    pub smtp_mail: String,
    #[cfg(feature = "mails")]
    pub smtp_user: String,
    #[cfg(feature = "mails")]
    pub smtp_password: String,

    // R2
    #[cfg(feature = "reports")]
    pub r2_bucket_name: String,
    #[cfg(feature = "reports")]
    pub r2_account_id: String,
    #[cfg(feature = "reports")]
    pub r2_public_bucket_endpoint: String,
    #[cfg(feature = "reports")]
    pub cloudflare_id: String,
    #[cfg(feature = "reports")]
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
    let api_url =
        env::var("API_URL").unwrap_or_else(|_| format!("{}:{}", Ipv4Addr::UNSPECIFIED, port));
    let client_url = get_var("CLIENT_URL")?;
    let admins = env::var("ADMINS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let database_url = get_var("DATABASE_URL")?;
    let database_name = get_var("DATABASE_NAME")?;

    let discord_token = get_var("DISCORD_TOKEN")?;
    let jwt_secret = get_var("JWT_SECRET")?;
    let enable_registrations = env::var("ENABLE_REGISTRATIONS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(true);

    let client_secret = get_var("CLIENT_SECRET")?;
    let client_id = get_var("CLIENT_ID")?;

    #[cfg(feature = "otel")]
    let otlp_endpoint = get_var("OTLP_ENDPOINT")?;
    #[cfg(feature = "otel")]
    let otlp_token = get_var("OTLP_TOKEN")?;
    #[cfg(feature = "otel")]
    let otlp_stream = get_var("OTLP_STREAM")?;

    #[cfg(feature = "mails")]
    let smtp = get_var("SMTP")?;
    #[cfg(feature = "mails")]
    let smtp_mail = get_var("SMTP_MAIL")?;
    #[cfg(feature = "mails")]
    let smtp_user = get_var("SMTP_USER")?;
    #[cfg(feature = "mails")]
    let smtp_password = get_var("SMTP_PASSWORD")?;

    #[cfg(feature = "reports")]
    let r2_bucket_name = get_var("R2_BUCKET_NAME")?;
    #[cfg(feature = "reports")]
    let r2_account_id = get_var("R2_ACCOUNT_ID")?;
    #[cfg(feature = "reports")]
    let r2_public_bucket_endpoint = get_var("R2_PUBLIC_BUCKET_ENDPOINT")?;
    #[cfg(feature = "reports")]
    let cloudflare_id = get_var("CLOUDFLARE_ID")?;
    #[cfg(feature = "reports")]
    let cloudflare_token = get_var("CLOUDFLARE_TOKEN")?;

    Ok(ENV.get_or_init(|| EnvConfig {
        port,
        api_url,
        client_url,
        admins,
        database_url,
        database_name,
        discord_token,
        jwt_secret,
        enable_registrations,
        client_secret,
        client_id,
        #[cfg(feature = "otel")]
        otlp_endpoint,
        #[cfg(feature = "otel")]
        otlp_token,
        #[cfg(feature = "otel")]
        otlp_stream,
        #[cfg(feature = "mails")]
        smtp,
        #[cfg(feature = "mails")]
        smtp_mail,
        #[cfg(feature = "mails")]
        smtp_user,
        #[cfg(feature = "mails")]
        smtp_password,
        #[cfg(feature = "reports")]
        r2_bucket_name,
        #[cfg(feature = "reports")]
        r2_account_id,
        #[cfg(feature = "reports")]
        r2_public_bucket_endpoint,
        #[cfg(feature = "reports")]
        cloudflare_id,
        #[cfg(feature = "reports")]
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
