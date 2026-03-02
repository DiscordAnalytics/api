use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{Bson, DateTime, Document, doc},
    error::Result,
    options::{
        FindOneAndUpdateOptions, FindOptions, ReturnDocument, TimeseriesGranularity,
        TimeseriesOptions,
    },
    results::{DeleteResult, InsertOneResult},
};

use crate::{
    domain::models::{BotStats, Guild, Interaction},
    utils::constants::BOT_STATS_COLLECTION,
};

#[derive(Clone, Default)]
pub struct BotStatsUpdate {
    updates: Document,
}

impl BotStatsUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_added_guilds(mut self, added_guilds: i32) -> Self {
        self.merge_inc(doc! { "addedGuilds": added_guilds });
        self
    }

    pub fn with_custom_event(mut self, event_key: &str, count: i32) -> Self {
        self.merge_inc(doc! { format!("customEvents.{}", event_key): count });
        self
    }

    pub fn with_guilds(mut self, guilds: &[Guild]) -> Self {
        if guilds.is_empty() {
            return self;
        }

        let new_guilds = guilds
            .iter()
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

        let update_doc = doc! {
            "guilds": {
                "$let": {
                    "vars": { "guilds": { "$ifNull": [ "$guilds", [] ] } },
                    "in": {
                        "$reduce": {
                            "input": new_guilds,
                            "initialValue": "$$guilds",
                            "in": {
                                "$let": {
                                    "vars": { "existing": "$$value" },
                                    "in": {
                                        "$cond": [
                                            { "$in": [ "$$this.guildId", "$$existing.guildId" ] },
                                            {
                                                "$map": {
                                                    "input": "$$existing",
                                                    "as": "g",
                                                    "in": {
                                                        "$cond": [
                                                            { "$eq": [ "$$g.guildId", "$$this.guildId" ] },
                                                            {
                                                                "guildId": "$$g.guildId",
                                                                "name": "$$this.name",
                                                                "icon": "$$this.icon",
                                                                "members": "$$this.members",
                                                                "interactions": { "$add": [ "$$g.interactions", "$$this.interactions" ] }
                                                            },
                                                            "$$g"
                                                        ]
                                                    }
                                                }
                                            },
                                            {
                                                "$concatArrays": [ "$$existing", [ "$$this" ] ]
                                            }
                                        ]
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        self.merge_set(update_doc);

        self
    }

    pub fn with_guild_count(mut self, guild_count: i32) -> Self {
        self.updates.insert("guildCount", guild_count);
        self
    }

    pub fn with_guild_locales(mut self, locales: &[(&str, i32)]) -> Self {
        let update_doc = Self::build_locale_update("guildLocales", locales);
        self.merge_set(update_doc);
        self
    }

    pub fn with_guild_member(mut self, bucket: &str, count: i32) -> Self {
        self.merge_inc(doc! { format!("guildMembers.{}", bucket): count });
        self
    }

    pub fn with_interactions(mut self, interactions: &[Interaction]) -> Self {
        if interactions.is_empty() {
            return self;
        }

        let new_interactions = interactions
            .iter()
            .map(|interaction| {
                doc! {
                    "commandType": interaction.command_type,
                    "name": &interaction.name,
                    "number": interaction.number,
                    "type": interaction.type_,
                }
            })
            .collect::<Vec<_>>();

        let update_doc = doc! {
            "interactions": {
                "$let": {
                  "vars": { "interactions": { "$ifNull": [ "$interactions", [] ] } },
                  "in": {
                      "$reduce": {
                          "input": new_interactions,
                          "initialValue": "$$interactions",
                          "in": {
                              "$let": {
                                  "vars": { "existing": "$$value" },
                                  "in": {
                                      "$cond": [
                                          { "$in": [ "$$this.name", "$$existing.name" ] },
                                          {
                                              "$map": {
                                                  "input": "$$existing",
                                                  "as": "i",
                                                  "in": {
                                                      "$cond": [
                                                          { "$eq": [ "$$i.name", "$$this.name" ] },
                                                          {
                                                              "commandType": "$$this.commandType",
                                                              "name": "$$i.name",
                                                              "number": { "$add": [ "$$i.number", "$$this.number" ] },
                                                              "type": "$$i.type"
                                                          },
                                                          "$$i"
                                                      ]
                                                  }
                                              }
                                          },
                                          {
                                              "$concatArrays": [ "$$existing", [ "$$this" ] ]
                                          }
                                      ]
                                  }
                              }
                          }
                      }
                  }
                }
            }
        };

        self.merge_set(update_doc);

        self
    }

    pub fn with_interactions_locales(mut self, locales: &[(&str, i32)]) -> Self {
        let update_doc = Self::build_locale_update("interactionsLocales", locales);
        self.merge_set(update_doc);
        self
    }

    pub fn with_removed_guilds(mut self, removed_guilds: i32) -> Self {
        self.merge_inc(doc! { "removedGuilds": removed_guilds });
        self
    }

    pub fn with_user_count(mut self, user_count: i32) -> Self {
        self.updates.insert("userCount", user_count);
        self
    }

    pub fn with_user_install_count(mut self, user_install_count: i32) -> Self {
        self.updates.insert("userInstallCount", user_install_count);
        self
    }

    pub fn with_user_type(mut self, user_type: &str, count: i32) -> Self {
        self.merge_inc(doc! { format!("usersType.{}", user_type): count });
        self
    }

    fn build_locale_update(field: &str, updates: &[(&str, i32)]) -> Document {
        let new_locales = updates
            .iter()
            .map(|(locale, number)| {
                doc! {
                    "locale": locale,
                    "number": number,
                }
            })
            .collect::<Vec<_>>();

        doc! {
            field: {
                "$let": {
                    "vars": { "locales": { "$ifNull": [ format!("${field}"), [] ] } },
                    "in": {
                        "$reduce": {
                            "input": new_locales,
                            "initialValue": "$$locales",
                            "in": {
                                "$let": {
                                    "vars": { "existing": "$$value" },
                                    "in": {
                                        "$cond": [
                                            { "$in": [ "$$this.locale", "$$existing.locale" ] },
                                            {
                                                "$map": {
                                                    "input": "$$existing",
                                                    "as": "l",
                                                    "in": {
                                                        "$cond": [
                                                            { "$eq": [ "$$l.locale", "$$this.locale" ] },
                                                            {
                                                                "locale": "$$l.locale",
                                                                "number": { "$add": [ "$$l.number", "$$this.number" ] }
                                                            },
                                                            "$$l"
                                                        ]
                                                    }
                                                }
                                            },
                                            {
                                                "$concatArrays": [ "$$existing", [ "$$this" ] ]
                                            }
                                        ]
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn merge_set(&mut self, doc: Document) {
        let set_doc = self
            .updates
            .entry("$set")
            .or_insert_with(|| Bson::Document(doc! {}));

        if let Bson::Document(existing) = set_doc {
            existing.extend(doc);
        }
    }

    fn merge_inc(&mut self, doc: Document) {
        let inc_doc = self
            .updates
            .entry("$inc")
            .or_insert_with(|| Bson::Document(doc! {}));

        if let Bson::Document(existing) = inc_doc {
            existing.extend(doc);
        }
    }

    pub fn build(self) -> Document {
        self.updates
    }
}

#[derive(Clone)]
pub struct BotStatsRepository {
    collection: Collection<BotStats>,
}

impl BotStatsRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        if !db
            .list_collection_names()
            .await?
            .contains(&BOT_STATS_COLLECTION.to_string())
        {
            let ts_opts = TimeseriesOptions::builder()
                .time_field("date")
                .granularity(Some(TimeseriesGranularity::Hours))
                .build();
            db.create_collection(BOT_STATS_COLLECTION)
                .timeseries(ts_opts)
                .await?;
        }

        Ok(Self {
            collection: db.collection(BOT_STATS_COLLECTION),
        })
    }

    pub async fn ping(&self) -> Result<()> {
        self.collection.find_one(doc! {}).await?;
        Ok(())
    }

    pub async fn find_by_bot_id(&self, bot_id: &str) -> Result<Vec<BotStats>> {
        let cursor = self.collection.find(doc! { "botId": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_date(&self, bot_id: &str, date: &DateTime) -> Result<Option<BotStats>> {
        self.collection
            .find_one(doc! { "botId": bot_id, "date": date })
            .await
    }

    pub async fn find_from_date_range(
        &self,
        bot_id: &str,
        start_date: &DateTime,
        end_date: &DateTime,
    ) -> Result<Vec<BotStats>> {
        let options = FindOptions::builder().sort(doc! { "date": 1 }).build();

        let cursor = self
            .collection
            .find(doc! {
                "botId": bot_id,
                "date": {
                    "$gte": start_date,
                    "$lte": end_date
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
            .find_one_and_update(
                doc! { "botId": bot_id, "date": date },
                doc! { "$set": updates },
            )
            .with_options(options)
            .await
    }

    pub async fn delete_by_date(&self, bot_id: &str, date: &DateTime) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "botId": bot_id, "date": date })
            .await
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_many(doc! { "botId": bot_id }).await
    }
}
