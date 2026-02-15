use mongodb::bson::DateTime;

use crate::utils::constants::DISCORD_EPOCH;

pub fn get_user_creation_date(id: &str) -> Option<DateTime> {
    let snowflake = id.parse::<i64>().ok()?;
    let timestamp = (snowflake >> 22) + DISCORD_EPOCH;
    Some(DateTime::from_millis(timestamp))
}

pub fn is_valid_snowflake(id: &str) -> bool {
    id.parse::<i64>().is_ok()
}
