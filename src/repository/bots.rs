use std::collections::{HashMap, HashSet};

use chrono::{Duration, Utc};
use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, Document, doc},
    error::Result,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    results::{DeleteResult, InsertOneResult},
};

use crate::{
    domain::models::{Bot, WebhookConfig},
    utils::constants::BOTS_COLLECTION,
};

use super::common::{CollectionSpec, UpdateBuilder, ensure_collection};

#[derive(Clone, Default)]
pub struct BotUpdate {
    builder: UpdateBuilder,
}

impl BotUpdate {
    pub fn with_advanced_stats(mut self, advanced_stats: Option<bool>) -> Self {
        if let Some(advanced_stats) = advanced_stats {
            self.builder = self.builder.set(doc! { "advancedStats": advanced_stats });
        }
        self
    }

    pub fn with_avatar(mut self, avatar: String) -> Self {
        self.builder = self.builder.set(doc! { "avatar": avatar });
        self
    }

    pub fn with_custom_events_limit(mut self, custom_events_limit: Option<i32>) -> Self {
        if let Some(custom_events_limit) = custom_events_limit {
            self.builder = self
                .builder
                .set(doc! { "customEventsLimit": custom_events_limit });
        }
        self
    }

    pub fn with_framework(mut self, framework: String) -> Self {
        self.builder = self.builder.set(doc! { "framework": framework });
        self
    }

    pub fn with_goals_limit(mut self, goals_limit: Option<i32>) -> Self {
        if let Some(goals_limit) = goals_limit {
            self.builder = self.builder.set(doc! { "goalsLimit": goals_limit });
        }
        self
    }

    pub fn with_suspended(mut self, suspended: bool) -> Self {
        self.builder = self.builder.set(doc! { "suspended": suspended });
        self
    }

    pub fn with_team(mut self, team: Vec<String>) -> Self {
        self.builder = self.builder.set(doc! { "team": team });
        self
    }

    pub fn with_teammates_limit(mut self, teammates_limit: Option<i32>) -> Self {
        if let Some(teammates_limit) = teammates_limit {
            self.builder = self.builder.set(doc! { "teammatesLimit": teammates_limit });
        }
        self
    }

    pub fn with_team_member(mut self, user_id: &str) -> Self {
        self.builder = self.builder.add_to_set(doc! { "team": user_id });
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.builder = self.builder.set(doc! { "token": token });
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.builder = self.builder.set(doc! { "username": username });
        self
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.builder = self.builder.set(doc! { "version": version });
        self
    }

    pub fn with_warn_level(mut self, warn_level: i32) -> Self {
        self.builder = self.builder.set(doc! { "warnLevel": warn_level });
        self
    }

    pub fn with_webhook_config(mut self, provider: &str, config: WebhookConfig) -> Self {
        self.builder = self
            .builder
            .set(doc! { format!("webhooksConfig.{}", provider): doc! {
                "connectionId": config.connection_id,
                "webhookSecret": config.webhook_secret,
            }});
        self
    }

    pub fn with_webhook_url(mut self, webhook_url: Option<String>) -> Self {
        if let Some(url) = webhook_url {
            self.builder = self.builder.set(
                doc! { "webhooksConfig.webhookUrl": if url.is_empty() { None } else { Some(url) } },
            );
        }
        self
    }

    pub fn build(self) -> Document {
        self.builder.build()
    }
}

#[derive(Clone)]
pub struct BotsRepository {
    collection: Collection<Bot>,
}

impl BotsRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        Ok(Self {
            collection: ensure_collection(db, BOTS_COLLECTION, CollectionSpec::Standard).await?,
        })
    }

    pub async fn find_all(&self) -> Result<Vec<Bot>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_many_by_ids(
        &self,
        bot_ids: &HashSet<String>,
    ) -> Result<HashMap<String, Bot>> {
        let cursor = self
            .collection
            .find(doc! { "botId": { "$in": bot_ids } })
            .await?;
        let bots = cursor.try_collect::<Vec<_>>().await?;
        Ok(bots
            .into_iter()
            .map(|bot| (bot.bot_id.clone(), bot))
            .collect())
    }

    pub async fn find_not_configured(&self) -> Result<Vec<Bot>> {
        let cursor = self.collection.find(doc! { "framework": null }).await?;
        cursor.try_collect().await
    }

    pub async fn find_inactive(&self) -> Result<Vec<Bot>> {
        let difference = Utc::now() - Duration::weeks(1);
        let one_week_ago = DateTime::from_millis(difference.timestamp_millis());

        let cursor = self
            .collection
            .find(doc! {
              "$or": [
                { "lastPush": null },
                { "lastPush": { "$lt": one_week_ago } }
              ]
            })
            .await?;
        cursor.try_collect().await
    }

    pub async fn find_by_id(&self, bot_id: &str) -> Result<Option<Bot>> {
        self.collection.find_one(doc! { "botId": bot_id }).await
    }

    pub async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Bot>> {
        let cursor = self
            .collection
            .find(doc! {
              "$or": [
                { "ownerId": user_id },
                { "team": { "$in": [user_id] } }
              ]
            })
            .await?;
        cursor.try_collect().await
    }

    pub async fn find_by_owner_id(&self, owner_id: &str) -> Result<Vec<Bot>> {
        let cursor = self.collection.find(doc! { "ownerId": owner_id }).await?;
        cursor.try_collect().await
    }

    pub async fn count_by_user_id(&self, user_id: &str) -> Result<u64> {
        self.collection
            .count_documents(doc! { "ownerId": user_id })
            .await
    }

    pub async fn insert(&self, bot: &Bot) -> Result<InsertOneResult> {
        self.collection.insert_one(bot).await
    }

    pub async fn update(&self, bot_id: &str, updated_bot: BotUpdate) -> Result<Option<Bot>> {
        let updates = updated_bot.build();

        if updates.is_empty() {
            return Ok(None);
        }

        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(doc! { "botId": bot_id }, updates)
            .with_options(options)
            .await
    }

    pub async fn set_suspension_for_owner(&self, owner_id: &str, suspended: bool) -> Result<()> {
        let cursor = self.collection.find(doc! { "ownerId": owner_id }).await?;

        let bots: Vec<Bot> = cursor.try_collect().await?;

        for bot in bots {
            self.collection
                .find_one_and_update(
                    doc! { "botId": &bot.bot_id },
                    doc! { "$set": { "suspended": suspended } },
                )
                .await?;
        }

        Ok(())
    }

    pub async fn remove_user_from_team(&self, bot_id: &str, user_id: &str) -> Result<Option<Bot>> {
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(
                doc! { "botId": bot_id },
                doc! { "$pull": { "team": user_id } },
            )
            .with_options(options)
            .await
    }

    pub async fn remove_user_from_teams(&self, user_id: &str) -> Result<()> {
        self.collection
            .update_many(doc! {}, doc! { "$pull": { "team": user_id } })
            .await?;

        Ok(())
    }

    pub async fn remove_integration(&self, provider: &str, connection_id: &str) -> Result<()> {
        let provider_key = format!("webhooksConfig.{}", provider);
        let key = format!("{}.connectionId", provider_key);

        self.collection
            .update_one(
                doc! { key.as_str(): connection_id },
                doc! { "$unset": { provider_key.as_str(): "" } },
            )
            .await?;

        Ok(())
    }

    pub async fn delete(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_one(doc! { "botId": bot_id }).await
    }
}
