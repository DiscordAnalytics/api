use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
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
    pub shared: Option<bool>,
    pub title: String,
    pub title_i18n: Option<String>,
    pub used_by: Option<i64>,
}

impl Achievement {
    pub fn with_achieved_on(mut self, achieved_on: Option<DateTime>) -> Self {
        self.achieved_on = achieved_on;
        self
    }

    pub fn with_bot_id(mut self, bot_id: String) -> Self {
        self.bot_id = bot_id;
        self
    }

    pub fn with_current(mut self, current: Option<i64>) -> Self {
        self.current = current;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_description_i18n(mut self, description_i18n: Option<String>) -> Self {
        self.description_i18n = description_i18n;
        self
    }

    pub fn with_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn with_from(mut self, from: Option<String>) -> Self {
        self.from = from;
        self
    }

    pub fn with_lang(mut self, lang: Option<String>) -> Self {
        self.lang = lang;
        self
    }

    pub fn with_objective(mut self, objective: AchievementObjective) -> Self {
        self.objective = objective;
        self
    }

    pub fn with_shared(mut self, shared: Option<bool>) -> Self {
        self.shared = shared;
        self
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn with_title_i18n(mut self, title_i18n: Option<String>) -> Self {
        self.title_i18n = title_i18n;
        self
    }

    pub fn with_used_by(mut self, used_by: Option<i64>) -> Self {
        self.used_by = used_by;
        self
    }

    pub fn is_achieved(&self) -> bool {
        match self.current {
            Some(current) => current >= self.objective.value,
            None => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
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
