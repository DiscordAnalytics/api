use apistos::ApiComponent;
use mongodb::bson::{DateTime, oid::ObjectId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub achieved_on: Option<DateTime>,
    pub bot_id: String,
    pub current: Option<i64>,
    pub description: String,
    pub description_i18n: Option<String>,
    pub editable: bool,
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
        editable: bool,
        objective: AchievementObjective,
    ) -> Self {
        Self {
            id: None,
            achieved_on: None,
            bot_id: bot_id.to_string(),
            current: None,
            description: description.to_string(),
            description_i18n: None,
            editable,
            from: None,
            lang: None,
            objective,
            shared: false,
            title: title.to_string(),
            title_i18n: None,
            used_by: 0,
        }
    }

    pub fn with_description_i18n(mut self, description_i18n: &str) -> Self {
        self.description_i18n = Some(description_i18n.to_string());
        self
    }

    pub fn with_from(mut self, from: &str) -> Self {
        self.from = Some(from.to_string());
        self
    }

    pub fn with_title_i18n(mut self, title_i18n: &str) -> Self {
        self.title_i18n = Some(title_i18n.to_string());
        self
    }

    pub fn is_achieved(&self) -> bool {
        match self.current {
            Some(current) => current >= self.objective.value,
            None => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct AchievementObjective {
    pub value: i64,
    #[serde(rename = "type")]
    pub achievement_type: AchievementType,
}

impl AchievementObjective {
    pub fn with_value(mut self, value: i64) -> Self {
        self.value = value;
        self
    }

    pub fn with_achievement_type(mut self, achievement_type: AchievementType) -> Self {
        self.achievement_type = achievement_type;
        self
    }
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
