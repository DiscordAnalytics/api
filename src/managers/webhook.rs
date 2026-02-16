use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;
use reqwest::{
    Client, StatusCode,
    header::{HeaderMap, HeaderValue},
};
use tracing::info;

use crate::{
    domain::models::{Provider, Webhook, WebhookSendData},
    utils::logger::LogCode,
};

static DISCORD_WEBHOOK_REGEX: OnceLock<Regex> = OnceLock::new();

pub struct VotesWebhooksManager {
    pub waitlist: Vec<Webhook>,
}

impl VotesWebhooksManager {
    pub fn new() -> Self {
        Self {
            waitlist: Vec::new(),
        }
    }

    fn is_discord_webhook(url: &str) -> bool {
        DISCORD_WEBHOOK_REGEX.get_or_init(|| {
            Regex::new(
                r"^https:\/\/([a-z]+\.)?discord\.com\/api\/webhooks\/\d+\/[\w-]+$"
            ).expect("Invalid Discord webhook regex")
        })
        .is_match(url)
    }

    pub fn increment_tries(&mut self, webhook: Webhook) {
        let entry = self.waitlist.iter_mut().find(|w| {
            w.data.bot_id == webhook.data.bot_id
                && w.data.voter_id == webhook.data.voter_id
                && w.data.provider == webhook.data.provider
                && w.data.date == webhook.data.date
                && w.data.raw_data == webhook.data.raw_data
        });

        if let Some(w) = entry {
            w.try_count += 1;
        } else {
            self.waitlist.push(Webhook {
                try_count: 1,
                ..webhook
            });
        }
    }

    pub async fn send_webhook(&mut self, webhook: Webhook) -> Result<()> {
        let client = Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&webhook.webhook_secret)?,
        );

        let provider_str = webhook.data.provider.as_str();

        let content = if Self::is_discord_webhook(&webhook.webhook_url) {
            match &webhook.data.provider {
                Provider::Test => Some(format!(
                    "Test received for <@{}> ({}) from Discord Analytics dashboard.",
                    webhook.data.bot_id, webhook.data.bot_id,
                )),
                _ => Some(format!(
                    "User <@{}> ({}) voted for <@{}> ({}) on {} at <t:{}>",
                    webhook.data.voter_id,
                    webhook.data.voter_id,
                    webhook.data.bot_id,
                    webhook.data.bot_id,
                    provider_str,
                    webhook.data.date.timestamp(),
                )),
            }
        } else {
            None
        };

        let res = client
            .post(&webhook.webhook_url)
            .json(&WebhookSendData {
                bot_id: webhook.data.bot_id.clone(),
                voter_id: webhook.data.voter_id.clone(),
                provider: provider_str.to_string(),
                date: webhook.data.date,
                raw_data: webhook.data.raw_data.clone(),
                content,
            })
            .headers(headers)
            .send()
            .await;

        match res {
            Ok(res) => match res.status() {
                StatusCode::OK => {
                    info!(
                        "[{}] Vote webhook of bot {} for provider {} has been sent",
                        LogCode::Request,
                        webhook.data.bot_id.as_str(),
                        webhook.data.provider.as_str()
                    );
                    self.waitlist.retain(|w| *w != webhook);
                }
                StatusCode::NO_CONTENT => {
                    info!(
                        "[{}] Vote webhook of bot {} for provider {} has been sent to a discord webhook",
                        LogCode::Request,
                        webhook.data.bot_id.as_str(),
                        webhook.data.provider.as_str()
                    );
                    self.waitlist.retain(|w| *w != webhook);
                }
                _ => {
                    info!(
                        "[{}] Vote webhook of bot {} for provider {} did not return a successful status code",
                        LogCode::Request,
                        webhook.data.bot_id.as_str(),
                        webhook.data.provider.as_str()
                    );
                    self.increment_tries(webhook)
                }
            },
            Err(_) => {
                info!(
                    "[{}] Vote webhook of bot {} for provider {} has failed to be sent",
                    LogCode::Request,
                    webhook.data.bot_id.as_str(),
                    webhook.data.provider.as_str()
                );
                self.increment_tries(webhook)
            }
        }

        Ok(())
    }
}
