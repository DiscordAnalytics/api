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
    GuildCount,
    InteractionAverageWeek,
    FrenchPercentage,
    JoinedDa,
    UsersLocales,
    UserCount,
    BotConfigured,
    VotesCount,
}
