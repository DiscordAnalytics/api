use std::collections::HashMap;

use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, Document, doc},
    error::Result,
    options::{FindOneAndUpdateOptions, FindOneOptions, FindOptions, ReturnDocument},
    results::{DeleteResult, InsertOneResult},
};

use crate::{
    domain::models::{BotStats, Guild, Interaction, Locale},
    utils::constants::BOT_STATS_COLLECTION,
};

use super::common::{UpdateBuilder, ensure_collection};

#[derive(Clone, Default)]
pub struct BotStatsUpdate {
    builder: UpdateBuilder,
}

impl BotStatsUpdate {
    pub fn with_added_guilds(mut self, added_guilds: i32) -> Self {
        self.builder = self.builder.inc(doc! { "addedGuilds": added_guilds });
        self
    }

    pub fn with_custom_event(mut self, event_key: &str, count: i32) -> Self {
        self.builder = self
            .builder
            .set(doc! { format!("customEvents.{}", event_key): count });
        self
    }

    pub fn with_guilds(mut self, guilds: &[Guild], existing_guilds: &[Guild]) -> Self {
        if guilds.is_empty() && existing_guilds.is_empty() {
            return self;
        }

        let mut map: HashMap<String, Guild> = HashMap::new();

        for g in existing_guilds {
            map.insert(g.guild_id.clone(), g.clone());
        }

        for g in guilds {
            map.entry(g.guild_id.clone())
                .and_modify(|existing| {
                    existing.interactions += g.interactions;
                    existing.members = g.members;
                    existing.name = g.name.clone();
                    existing.icon = g.icon.clone();
                })
                .or_insert_with(|| g.clone());
        }

        let merged = map
            .into_values()
            .map(|guild| {
                doc! {
                    "guildId": &guild.guild_id,
                    "icon": &guild.icon,
                    "interactions": guild.interactions,
                    "members": guild.members,
                    "name": &guild.name,
                }
            })
            .collect::<Vec<_>>();

        self.builder = self.builder.set(doc! { "guilds": merged });

        self
    }

    pub fn with_guild_count(mut self, guild_count: i32) -> Self {
        self.builder = self.builder.set(doc! { "guildCount": guild_count });
        self
    }

    pub fn with_guild_locales(mut self, locales: &[(&str, i32)], existing: &[Locale]) -> Self {
        let update_doc = Self::build_locale_update("guildLocales", locales, existing);
        self.builder = self.builder.set(update_doc);
        self
    }

    pub fn with_guild_member(mut self, bucket: &str, count: i32) -> Self {
        self.builder = self
            .builder
            .inc(doc! { format!("guildMembers.{}", bucket): count });
        self
    }

    pub fn with_interactions(
        mut self,
        interactions: &[Interaction],
        existing_interactions: &[Interaction],
    ) -> Self {
        if interactions.is_empty() && existing_interactions.is_empty() {
            return self;
        }

        let mut map: HashMap<(Option<i32>, String, i32), Interaction> = HashMap::new();

        for i in existing_interactions {
            map.insert((i.command_type, i.name.clone(), i.type_), i.clone());
        }

        for i in interactions {
            let key = (i.command_type, i.name.clone(), i.type_);
            map.entry(key)
                .and_modify(|existing| existing.number += i.number)
                .or_insert_with(|| i.clone());
        }

        let merged = map
            .into_values()
            .map(|interaction| {
                doc! {
                    "commandType": interaction.command_type,
                    "name": &interaction.name,
                    "number": interaction.number,
                    "type": interaction.type_,
                }
            })
            .collect::<Vec<_>>();

        self.builder = self.builder.set(doc! { "interactions": merged });

        self
    }

    pub fn with_interactions_locales(
        mut self,
        locales: &[(&str, i32)],
        existing: &[Locale],
    ) -> Self {
        let update_doc = Self::build_locale_update("interactionsLocales", locales, existing);
        self.builder = self.builder.set(update_doc);
        self
    }

    pub fn with_removed_guilds(mut self, removed_guilds: i32) -> Self {
        self.builder = self.builder.inc(doc! { "removedGuilds": removed_guilds });
        self
    }

    pub fn with_user_count(mut self, user_count: i32) -> Self {
        self.builder = self.builder.set(doc! { "userCount": user_count });
        self
    }

    pub fn with_user_install_count(mut self, user_install_count: i32) -> Self {
        self.builder = self
            .builder
            .set(doc! { "userInstallCount": user_install_count });
        self
    }

    pub fn with_user_type(mut self, user_type: &str, count: i32) -> Self {
        self.builder = self
            .builder
            .inc(doc! { format!("usersType.{}", user_type): count });
        self
    }

    fn build_locale_update(field: &str, updates: &[(&str, i32)], existing: &[Locale]) -> Document {
        let mut map: HashMap<String, i32> = HashMap::new();

        for l in existing {
            map.insert(l.locale.clone(), l.number);
        }

        for (locale, number) in updates {
            map.entry(locale.to_string())
                .and_modify(|n| *n += *number)
                .or_insert(*number);
        }

        doc! {
            field: map
                .into_iter()
                .map(|(locale, number)| {
                    doc! {
                        "locale": locale,
                        "number": number,
                    }
                })
                .collect::<Vec<_>>(),
        }
    }

    pub fn build(self) -> Document {
        self.builder.build()
    }
}

#[derive(Clone)]
pub struct BotStatsRepository {
    collection: Collection<BotStats>,
}

impl BotStatsRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        Ok(Self {
            collection: ensure_collection(db, BOT_STATS_COLLECTION).await?,
        })
    }

    pub async fn find_last(&self, bot_id: &str) -> Result<Option<BotStats>> {
        let options = FindOneOptions::builder().sort(doc! { "date": -1 }).build();
        self.collection
            .find_one(doc! { "botId": bot_id })
            .with_options(options)
            .await
    }

    pub async fn find_by_date(&self, bot_id: &str, date: &DateTime) -> Result<Option<BotStats>> {
        self.collection
            .find_one(doc! { "botId": bot_id, "date": date })
            .await
    }

    pub async fn find_from_date_range(
        &self,
        bot_id: &str,
        from: &DateTime,
        to: &DateTime,
    ) -> Result<Vec<BotStats>> {
        let options = FindOptions::builder().sort(doc! { "date": 1 }).build();

        let cursor = self
            .collection
            .find(doc! {
                "botId": bot_id,
                "date": {
                    "$gte": from,
                    "$lte": to
                }
            })
            .with_options(options)
            .await?;
        cursor.try_collect().await
    }

    pub async fn insert(&self, bot_stats: &BotStats) -> Result<InsertOneResult> {
        self.collection.insert_one(bot_stats).await
    }

    pub async fn update(
        &self,
        bot_id: &str,
        date: &DateTime,
        updated_bot_stats: BotStatsUpdate,
    ) -> Result<Option<BotStats>> {
        let updates = updated_bot_stats.build();

        if updates.is_empty() {
            return Ok(None);
        }

        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(doc! { "botId": bot_id, "date": date }, updates)
            .with_options(options)
            .await
    }

    pub async fn remove_event_from_stats(&self, bot_id: &str, event_key: &str) -> Result<()> {
        let field = format!("customEvents.{}", event_key);

        self.collection
            .update_many(doc! { "botId": bot_id }, doc! { "$unset": { field: "" } })
            .await?;

        Ok(())
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_many(doc! { "botId": bot_id }).await
    }
}
