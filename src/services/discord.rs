use anyhow::{Result, anyhow};
use reqwest::Client;
use tracing::error;

use crate::{
    app_env,
    openapi::schemas::{DiscordBot, DiscordOAuthUser, DiscordTokenResponse},
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

    pub async fn get_bot(&self, bot_id: &str) -> Result<DiscordBot> {
        let response = self
            .client
            .get(format!("https://discord.com/api/users/{}", bot_id))
            .header("Authorization", format!("Bot {}", app_env!().discord_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!(
                code = %LogCode::Auth,
                error = %error_text,
                "Failed to fetch Discord bot"
            );
            return Err(anyhow!("Failed to fetch Discord bot: {}", error_text));
        }

        Ok(response.json().await?)
    }

    pub async fn exchange_linked_roles_code(&self, code: &str) -> Result<DiscordTokenResponse> {
        let redirect_uri = format!("{}/auth/linkedroles", app_env!().api_url);
        let params = [
            ("client_id", app_env!().client_id.as_str()),
            ("client_secret", app_env!().client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
            ("code", code),
            ("scope", "role_connections.write identify"),
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
                "Failed to exchange linked roles OAuth code"
            );
            return Err(anyhow!(
                "Discord linked roles OAuth token exchange failed: {}",
                error_text
            ));
        }

        Ok(response.json().await?)
    }

    pub async fn update_role_connection(
        &self,
        token_type: &str,
        access_token: &str,
        bot_count: i32,
    ) -> Result<()> {
        let response = self
            .client
            .put(format!(
                "https://discord.com/api/users/@me/applications/{}/role-connection",
                app_env!().client_id
            ))
            .header("Authorization", format!("{} {}", token_type, access_token))
            .json(&serde_json::json!({
                "platform_name": "Discord Analytics",
                "metadata": {
                    "botcount": bot_count.to_string()
                }
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!(
                code = %LogCode::Auth,
                error = %error_text,
                "Failed to update Discord role connection"
            );
            return Err(anyhow!(
                "Failed to update Discord role connection: {}",
                error_text
            ));
        }

        Ok(())
    }
}
