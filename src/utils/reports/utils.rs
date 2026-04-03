use mongodb::bson::DateTime;

use crate::domain::models::BotStats;

fn days_between(from: DateTime, to: DateTime) -> Option<usize> {
    let ms_per_day = 24 * 60 * 60 * 1000;
    let diff = to.timestamp_millis() - from.timestamp_millis();
    if diff < 0 {
        None
    } else {
        Some((diff / ms_per_day) as usize)
    }
}

pub fn compute_stats(
    stats: Vec<BotStats>,
    from: DateTime,
    days_count: usize,
) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut interactions_count = vec![0f32; days_count];
    let mut guilds_count = vec![0f32; days_count];
    let mut users_count = vec![0f32; days_count];

    for stat in stats {
        if let Some(day_index) = days_between(from, stat.date)
            && day_index < days_count
        {
            interactions_count[day_index] += stat
                .interactions
                .iter()
                .map(|i| i.number as f32)
                .sum::<f32>();
            guilds_count[day_index] += stat.guild_count as f32;
            users_count[day_index] += stat.user_count as f32;
        }
    }

    (interactions_count, guilds_count, users_count)
}
