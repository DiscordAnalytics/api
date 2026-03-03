use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::Duration,
};

use actix_web::rt;
use anyhow::Result;
use regex::Regex;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use tokio::{sync::Mutex, time::sleep};
use tracing::info;

use crate::{
    domain::models::{Provider, Webhook, WebhookSendData},
    utils::{constants::MAX_WEBHOOK_RETRIES, logger::LogCode},
};

static DISCORD_WEBHOOK_REGEX: OnceLock<Regex> = OnceLock::new();

pub struct VotesWebhooksManager {
    pub waitlist: HashMap<String, Webhook>,
    client: Client,
}

impl Default for VotesWebhooksManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VotesWebhooksManager {
    pub fn new() -> Self {
        Self {
            waitlist: HashMap::new(),
            client: Client::new(),
        }
    }

    pub fn queue_webhook(&mut self, webhook: Webhook) {
        let key = Self::build_key(&webhook);
        self.waitlist.insert(key, webhook);
    }

    fn is_discord_webhook(url: &str) -> bool {
        DISCORD_WEBHOOK_REGEX
            .get_or_init(|| {
                Regex::new(r"^https:\/\/([a-z]+\.)?discord(app)?\.com\/api(\/v\d+)?\/webhooks\/\d+\/[\w-]+$")
                    .expect("Invalid Discord webhook regex")
            })
            .is_match(url)
    }

    fn build_payload(webhook: &Webhook) -> Result<(WebhookSendData<'_>, HeaderMap)> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&webhook.webhook_secret)?,
        );

        let provider_str = webhook.data.provider.to_str();

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

        let data = WebhookSendData {
            bot_id: &webhook.data.bot_id,
            voter_id: &webhook.data.voter_id,
            provider: provider_str,
            date: webhook.data.date,
            raw_data: webhook.data.raw_data.as_ref(),
            content,
        };

        Ok((data, headers))
    }

    fn build_key(webhook: &Webhook) -> String {
        format!(
            "{}:{}:{}",
            webhook.webhook_url,
            webhook.data.voter_id,
            webhook.data.provider.to_str()
        )
    }

    pub async fn send(manager: Arc<Mutex<Self>>, webhook: Webhook) {
        let (client, payload, headers) = {
            let manager = manager.lock().await;
            let (payload, headers) =
                Self::build_payload(&webhook).expect("Failed to build webhook payload");
            (manager.client.clone(), payload, headers)
        };

        let result = client
            .post(&webhook.webhook_url)
            .json(&payload)
            .headers(headers)
            .send()
            .await;

        let mut mgr = manager.lock().await;
        match result {
            Ok(res) if res.status().is_success() => {
                info!(
                    code = %LogCode::Request,
                    "Vote webhook of bot {} for provider {} has been sent",
                    webhook.data.bot_id.as_str(),
                    webhook.data.provider.to_str()
                );
                let key = Self::build_key(&webhook);
                mgr.waitlist.remove(&key);
            }
            _ => {
                info!(
                    code = %LogCode::Request,
                    "Vote webhook of bot {} for provider {} has failed to be sent",
                    webhook.data.bot_id.as_str(),
                    webhook.data.provider.to_str()
                );

                if let Some(delay) = mgr.retry(&webhook) {
                    drop(mgr);
                    Self::schedule_retry(manager.clone(), webhook, delay);
                }
            }
        }
    }

    fn retry(&mut self, webhook: &Webhook) -> Option<Duration> {
        let key = Self::build_key(webhook);

        let entry = self.waitlist.entry(key).or_insert_with(|| {
            let mut w = webhook.clone();
            w.try_count = 0;
            w
        });

        entry.try_count = entry.try_count.saturating_add(1);

        if entry.try_count > MAX_WEBHOOK_RETRIES {
            self.waitlist.remove(&Self::build_key(webhook));
            return None;
        }

        let exp = (entry.try_count.min(6)) as u32;
        let delay = Duration::from_secs(2u64.pow(exp));

        Some(delay)
    }

    fn schedule_retry(manager: Arc<Mutex<Self>>, webhook: Webhook, delay: Duration) {
        rt::spawn(async move {
            sleep(delay).await;
            Self::send(manager, webhook).await;
        });
    }
}
