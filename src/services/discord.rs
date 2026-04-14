use anyhow::{Result, anyhow};
use reqwest::Client;
use tracing::{debug, error, info};

use crate::{
    app_env,
    openapi::schemas::{DiscordTokenResponse, DiscordUser},
    utils::{
        discord::{DiscordEmbed, DmChannel},
        logger::LogCode,
    },
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

    pub async fn get_user(&self, token_type: &str, access_token: &str) -> Result<DiscordUser> {
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

    pub async fn get_bot(&self, bot_id: &str) -> Result<DiscordUser> {
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

    pub async fn send_dm(&self, user_id: &str, embeds: Option<Vec<DiscordEmbed>>) -> Result<()> {
        if cfg!(debug_assertions) {
            debug!(
                code = %LogCode::Mail,
                "Skipping DM send in debug mode"
            );
            return Ok(());
        }

        let bot_token = &app_env!().discord_token;

        let dm_channel = self.create_dm_channel(user_id, bot_token).await?;

        let mut payload = serde_json::json!({});

        if let Some(embeds) = embeds {
            payload["embeds"] = serde_json::json!(embeds);
        }

        let response = self
            .client
            .post(format!(
                "https://discord.com/api/channels/{}/messages",
                dm_channel.id
            ))
            .header("Authorization", format!("Bot {}", bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            error!(
                code = %LogCode::Mail,
                status = %status,
                error = %error_text,
                "Failed to send DM message"
            );
            return Err(anyhow!(
                "Failed to send DM message: {} - {}",
                status,
                error_text
            ));
        }

        info!(
            code = %LogCode::Mail,
            user_id = %user_id,
            "Successfully sent DM message"
        );

        Ok(())
    }

    async fn create_dm_channel(&self, user_id: &str, bot_token: &str) -> Result<DmChannel> {
        let response = self
            .client
            .post("https://discord.com/api/users/@me/channels")
            .header("Authorization", format!("Bot {}", bot_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "recipient_id": user_id
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            error!(
                code = %LogCode::Mail,
                status = %status,
                error = %error_text,
                "Failed to create DM channel"
            );
            return Err(anyhow!(
                "Failed to create DM channel: {} - {}",
                status,
                error_text
            ));
        }

        let dm_channel: DmChannel = response.json().await?;

        info!(
            code = %LogCode::Mail,
            user_id = %user_id,
            channel_id = %dm_channel.id,
            "Successfully created DM channel"
        );

        Ok(dm_channel)
    }
}
