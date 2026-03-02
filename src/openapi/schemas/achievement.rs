use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::{Achievement, AchievementObjective};

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AchievementResponse {
    pub id: String,
    pub achieved_on: Option<String>,
    pub current: Option<i64>,
    pub description: String,
    pub description_i18n: Option<String>,
    pub editable: bool,
    pub lang: Option<String>,
    pub objective: AchievementObjective,
    pub shared: bool,
    pub title: String,
    pub title_i18n: Option<String>,
    pub used_by: i64,
}

impl TryFrom<Achievement> for AchievementResponse {
    type Error = anyhow::Error;

    fn try_from(achievement: Achievement) -> Result<Self, Self::Error> {
        Ok(Self {
            id: achievement
                .id
                .ok_or_else(|| anyhow::anyhow!("Achievement ID is missing"))?
                .to_string(),
            achieved_on: achievement
                .achieved_on
                .map(|dt| dt.try_to_rfc3339_string())
                .transpose()?,
            current: achievement.current,
            description: achievement.description,
            description_i18n: achievement.description_i18n,
            editable: achievement.editable,
            lang: achievement.lang,
            objective: achievement.objective,
            shared: achievement.shared,
            title: achievement.title,
            title_i18n: achievement.title_i18n,
            used_by: achievement.used_by,
        })
    }
}
