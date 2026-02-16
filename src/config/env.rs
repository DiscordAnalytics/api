use std::{env, sync::OnceLock};

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
    pub admin_token: String,
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

    // Octokit
    pub github_client_id: String,
    pub github_app_private_key: String,
    pub github_install_id: String,

    // R2
    pub r2_bucket_name: String,
    pub r2_account_id: String,
    pub r2_public_bucket_endpoint: String,
    pub cloudflare_id: String,
    pub cloudflare_token: String,
}

pub static ENV: OnceLock<EnvConfig> = OnceLock::new();

fn get_var(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("Environment variable {} not set", key))
}

pub fn init_env() -> Result<&'static EnvConfig, String> {
    dotenv().map_err(|e| format!("Failed to load .env file: {}", e))?;

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

    let otlp_endpoint = get_var("OTLP_ENDPOINT").ok();
    let otlp_token = get_var("OTLP_TOKEN").ok();
    let otlp_stream = get_var("OTLP_STREAM").ok();
    // Throw error if not all vars are defined
    if !((otlp_endpoint.is_some() && otlp_token.is_some() && otlp_stream.is_some())
        && (otlp_endpoint.is_none() && otlp_token.is_none() && otlp_stream.is_none()))
    {
        return Err(
            "One of these env vars are missing: OTLP_ENDPOINT, OTLP_TOKEN or OTLP_STREAM"
                .to_string(),
        );
    }

    let admin_token = get_var("ADMIN_TOKEN")?;
    let discord_token = get_var("DISCORD_TOKEN")?;
    let jwt_secret = get_var("JWT_SECRET")?;

    let client_secret = get_var("CLIENT_SECRET")?;
    let client_id = get_var("CLIENT_ID")?;

    let smtp = get_var("SMTP")?;
    let smtp_mail = get_var("SMTP_MAIL")?;
    let smtp_user = get_var("SMTP_USER")?;
    let smtp_password = get_var("SMTP_PASSWORD")?;

    let github_client_id = get_var("GITHUB_CLIENT_ID")?;
    let github_app_private_key = get_var("GITHUB_APP_PRIVATE_KEY")?;
    let github_install_id = get_var("GITHUB_INSTALL_ID")?;

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
        admin_token,
        discord_token,
        jwt_secret,
        client_secret,
        client_id,
        smtp,
        smtp_mail,
        smtp_user,
        smtp_password,
        github_client_id,
        github_app_private_key,
        github_install_id,
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
