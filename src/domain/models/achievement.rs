use apistos::ApiComponent;
use mongodb::bson::{DateTime, oid::ObjectId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub achieved_on: Option<DateTime>,
    pub bot_id: String,
    pub current: Option<i64>,
    pub description: String,
    pub description_i18n: Option<String>,
    pub from: Option<String>,
    pub lang: Option<String>,
    pub objective: AchievementObjective,
    pub shared: bool,
    pub title: String,
    pub title_i18n: Option<String>,
    pub used_by: i64,
}

impl Achievement {
    pub fn new(
        bot_id: &str,
        description: &str,
        title: &str,
        objective: AchievementObjective,
    ) -> Self {
        Self {
            id: ObjectId::new(),
            achieved_on: None,
            bot_id: bot_id.to_owned(),
            current: None,
            description: description.to_owned(),
            description_i18n: None,
            from: None,
            lang: None,
            objective,
            shared: false,
            title: title.to_owned(),
            title_i18n: None,
            used_by: 0,
        }
    }

    pub fn with_description_i18n(mut self, description_i18n: &str) -> Self {
        self.description_i18n = Some(description_i18n.to_owned());
        self
    }

    pub fn with_from(mut self, from: &str) -> Self {
        self.from = Some(from.to_owned());
        self
    }

    pub fn with_title_i18n(mut self, title_i18n: &str) -> Self {
        self.title_i18n = Some(title_i18n.to_owned());
        self
    }

    pub fn defaults(bot_id: &str) -> Vec<Self> {
        vec![
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Be on 75 servers.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.1.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::GuildCount,
                    value: 75,
                },
                shared: true,
                title: "#RoadToCertification".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.1.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Have an average of at least 10 interactions over the 7 days."
                    .to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.2.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::InteractionAverageWeek,
                    value: 50,
                },
                shared: true,
                title: "Get started with interactions".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.2.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Have at least 10% of French-speaking users.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.3.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::FrenchPercentage,
                    value: 10,
                },
                shared: true,
                title: "French power 🥖".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.3.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Be on 300 servers.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.4.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::GuildCount,
                    value: 300,
                },
                shared: true,
                title: "Medium bot".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.4.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Use Discord Analytics for at least a year.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.5.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::JoinedDa,
                    value: 31556952000,
                },
                shared: true,
                title: "Old member".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.5.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Have users who speak at least 15 different languages.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.6.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::UsersLocales,
                    value: 15,
                },
                shared: true,
                title: "Diversified users".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.6.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Be on 1k servers.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.7.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::GuildCount,
                    value: 1000,
                },
                shared: true,
                title: "Big bot".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.7.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Have an average of at least 1k interactions over the 7 days."
                    .to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.8.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::InteractionAverageWeek,
                    value: 1000,
                },
                shared: true,
                title: "In search of popularity".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.8.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Have at least 100k users.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.9.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::UserCount,
                    value: 100000,
                },
                shared: true,
                title: "Oh! Big servers".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.9.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Have at least 1M users.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.10.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::UserCount,
                    value: 1000000,
                },
                shared: true,
                title: "Big servers + {username} = ♥".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.10.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Be on 10k servers.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.11.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::GuildCount,
                    value: 10000,
                },
                shared: true,
                title: "Huge bot".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.11.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Have an average of at least 20k interactions over the 7 days."
                    .to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.12.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::InteractionAverageWeek,
                    value: 20000,
                },
                shared: true,
                title: "Too popular for you".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.12.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Have an average of at least 50k interactions over the 7 days."
                    .to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.13.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::UserCount,
                    value: 50000,
                },
                shared: true,
                title: "Explosion of interactions".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.13.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Have at least 10k users.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.14.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::UserCount,
                    value: 10000,
                },
                shared: true,
                title: "Just a little for me".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.14.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Finish Discord Analytics' configuration.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.15.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::BotConfigured,
                    value: 1,
                },
                shared: true,
                title: "All the stats for me".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.15.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Use Discord Analytics for at least a month.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.16.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::JoinedDa,
                    value: 2629746000,
                },
                shared: true,
                title: "Oh! That's nice".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.16.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Be on 10 servers.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.17.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::GuildCount,
                    value: 10,
                },
                shared: true,
                title: "Bot on the move".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.17.title".to_owned(),
                ),
                used_by: 0,
            },
            Self {
                id: ObjectId::new(),
                achieved_on: None,
                bot_id: bot_id.to_owned(),
                current: Some(0),
                description: "Have received at least 20 votes over the 30 days.".to_owned(),
                description_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.18.description"
                        .to_owned(),
                ),
                from: Some("DiscordAnalytics".to_owned()),
                lang: None,
                objective: AchievementObjective {
                    achievement_type: AchievementType::VotesCount,
                    value: 20,
                },
                shared: true,
                title: "Votes operation".to_owned(),
                title_i18n: Some(
                    "pages.dashboard.bots.achievements.default_achievements.18.title".to_owned(),
                ),
                used_by: 0,
            },
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct AchievementObjective {
    #[serde(rename = "type")]
    pub achievement_type: AchievementType,
    pub value: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ApiComponent, JsonSchema)]
pub enum AchievementType {
    BotConfigured,
    FrenchPercentage,
    GuildCount,
    InteractionAverageWeek,
    JoinedDa,
    UserCount,
    UsersLocales,
    VotesCount,
}
