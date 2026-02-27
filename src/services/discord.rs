use anyhow::{Result, anyhow};
use reqwest::Client;
use tracing::error;

use crate::{
    app_env,
    openapi::schemas::{DiscordOAuthUser, DiscordTokenResponse},
    utils::logger::LogCode,
};

#[derive(Clone)]
pub struct DiscordService {
    client: Client,
}

impl DiscordService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        redirect_uri: &str,
        scopes: &str,
    ) -> Result<DiscordTokenResponse> {
        let params = [
            ("client_id", app_env!().client_id.as_str()),
            ("client_secret", app_env!().client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri),
            ("code", code),
            ("scope", scopes),
        ];

        let response = self
            .client
            .post("https://discord.com/api/oauth2/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!(
                code = %LogCode::Auth,
                error = %error_text,
                "Failed to exchange OAuth code"
            );
            return Err(anyhow!(
                "Discord OAuth token exchange failed: {}",
                error_text
            ));
        }

        Ok(response.json().await?)
    }

    pub async fn get_user(&self, token_type: &str, access_token: &str) -> Result<DiscordOAuthUser> {
        let response = self
            .client
            .get("https://discord.com/api/users/@me")
            .header("Authorization", format!("{} {}", token_type, access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!(
                code = %LogCode::Auth,
                error = %error_text,
                "Failed to fetch Discord user"
            );
            return Err(anyhow!("Failed to fetch Discord user: {}", error_text));
        }

        Ok(response.json().await?)
    }
}
