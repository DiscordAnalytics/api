use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::{Achievement, AchievementObjective};

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AchievementResponse {
    pub id: String,
    pub achieved_on: Option<String>,
    pub bot_id: Option<String>,
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

impl AchievementResponse {
    pub fn from_shared(achievement: Achievement, admin: bool) -> anyhow::Result<Self> {
        let mut res = Self::try_from(achievement)?;
        res.achieved_on = None;
        if !admin {
            res.bot_id = None;
        }
        res.current = None;
        Ok(res)
    }
}

impl TryFrom<Achievement> for AchievementResponse {
    type Error = anyhow::Error;

    fn try_from(achievement: Achievement) -> Result<Self, Self::Error> {
        Ok(Self {
            id: achievement.id.to_string(),
            achieved_on: achievement
                .achieved_on
                .map(|dt| dt.try_to_rfc3339_string())
                .transpose()?,
            bot_id: Some(achievement.bot_id),
            current: achievement.current,
            description: achievement.description,
            description_i18n: achievement.description_i18n,
            from: achievement.from,
            lang: achievement.lang,
            objective: achievement.objective,
            shared: achievement.shared,
            title: achievement.title,
            title_i18n: achievement.title_i18n,
            used_by: achievement.used_by,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AchievementCreationPayload {
    pub description: String,
    pub description_i18n: Option<String>,
    pub from: Option<String>,
    pub objective: AchievementObjective,
    pub shared: Option<bool>,
    pub title: String,
    pub title_i18n: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct AchievementUpdatePayload {
    pub id: String,
    pub description: Option<String>,
    pub lang: Option<String>,
    pub title: Option<String>,
    pub shared: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct DeleteAchievementQuery {
    pub id: String,
}
