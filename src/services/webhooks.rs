use std::sync::Arc;

use anyhow::Result;
use chrono::{Duration, Utc};
use mongodb::bson::DateTime;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    domain::models::{AchievementType, Bot, Provider, Vote, Webhook, WebhookData},
    managers::VotesWebhooksManager,
    repository::Repositories,
    utils::logger::LogCode,
};

#[derive(Clone)]
pub struct WebhooksService {
    repos: Repositories,
}

impl WebhooksService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn record_vote(
        &self,
        bot_id: &str,
        user_id: &str,
        provider: &str,
        vote_count: i32,
    ) -> Result<()> {
        let current_date = DateTime::now();

        let start_of_hour = DateTime::from_millis(
            current_date.timestamp_millis() - (current_date.timestamp_millis() % 3600000),
        );

        info!(
            code = %LogCode::Webhook,
            bot_id = %bot_id,
            user_id = %user_id,
            provider = %provider,
            vote_count = %vote_count,
            "Recording vote"
        );

        match self
            .repos
            .votes
            .find_by_date_and_provider(bot_id, &start_of_hour, provider)
            .await?
        {
            Some(_) => {
                self.repos
                    .votes
                    .increment_count(bot_id, &start_of_hour, provider, vote_count)
                    .await?;

                info!(
                    code = %LogCode::Webhook,
                    bot_id = %bot_id,
                    provider = %provider,
                    vote_count = %vote_count,
                    "Vote count updated for existing record"
                );
            }
            None => {
                let new_vote = Vote {
                    bot_id: bot_id.to_string(),
                    count: vote_count,
                    date: start_of_hour,
                    provider: provider.to_string(),
                };
                self.repos.votes.insert(&new_vote).await?;

                info!(
                    code = %LogCode::Webhook,
                    bot_id = %bot_id,
                    provider = %provider,
                    vote_count = %vote_count,
                    "New vote record created"
                );
            }
        }

        self.check_vote_achievements(bot_id).await?;

        info!(
            code = %LogCode::Webhook,
            bot_id = %bot_id,
            user_id = %user_id,
            provider = %provider,
            vote_count = %vote_count,
            "Vote received"
        );

        Ok(())
    }

    async fn check_vote_achievements(&self, bot_id: &str) -> Result<()> {
        let one_week_ago =
            DateTime::from_millis((Utc::now() - Duration::days(7)).timestamp_millis());

        let week_votes = self
            .repos
            .votes
            .count_votes_since(bot_id, &one_week_ago)
            .await?;

        info!(
            code = %LogCode::Webhook,
            bot_id = %bot_id,
            week_votes = %week_votes,
            "Checking vote achievements"
        );

        let achievements = self
            .repos
            .achievements
            .find_unachieved_by_bot(bot_id)
            .await?;

        for mut achievement in achievements {
            if achievement.objective.achievement_type != AchievementType::VotesCount {
                continue;
            }

            achievement.current = Some(week_votes);

            if achievement.current.unwrap_or(0) >= achievement.objective.value {
                achievement.achieved_on = Some(DateTime::now());
                info!(
                    code = %LogCode::Webhook,
                    bot_id = %bot_id,
                    achievement = ?achievement,
                    "Achievement unlocked"
                );
            }

            self.repos
                .achievements
                .update_progress(
                    bot_id,
                    achievement.id.to_hex().as_str(),
                    achievement.current,
                    achievement.achieved_on,
                )
                .await?;
        }

        Ok(())
    }

    pub async fn trigger_webhook_notification(
        &self,
        bot: &Bot,
        voter_id: &str,
        provider: &str,
        raw_data: Value,
        webhook_manager: &Arc<Mutex<VotesWebhooksManager>>,
    ) -> Result<()> {
        if let Some(webhook_config) = bot.webhooks_config.get(provider) {
            let webhook_url = match &webhook_config.webhook_url {
                Some(url) if !url.is_empty() => url.clone(),
                _ => {
                    info!(
                        code = %LogCode::Webhook,
                        bot_id = %bot.bot_id,
                        provider = %provider,
                        "Webhook URL not configured, skipping notification"
                    );
                    return Ok(());
                }
            };
            let webhook_secret = match &webhook_config.webhook_secret {
                Some(secret) if !secret.is_empty() => secret.clone(),
                _ => {
                    info!(
                        code = %LogCode::Webhook,
                        bot_id = %bot.bot_id,
                        provider = %provider,
                        "Webhook secret not configured, skipping notification"
                    );
                    return Ok(());
                }
            };
            let webhook = Webhook {
                webhook_url,
                data: WebhookData {
                    bot_id: bot.bot_id.clone(),
                    date: Utc::now(),
                    provider: Provider::parse_str(provider),
                    raw_data: Some(raw_data),
                    voter_id: voter_id.to_string(),
                },
                try_count: 0,
                webhook_secret,
            };

            let mut manager = webhook_manager.lock().await;
            manager.queue_webhook(webhook.clone());
            drop(manager);

            let manager_clone = webhook_manager.clone();
            tokio::spawn(async move {
                VotesWebhooksManager::send(manager_clone, webhook).await;
            });

            info!(
                code = %LogCode::Webhook,
                bot_id = %bot.bot_id,
                voter_id = %voter_id,
                provider = %provider,
                "Webhook queued and triggered"
            );
        }

        Ok(())
    }
}
