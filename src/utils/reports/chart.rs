use anyhow::Result;
use charts_rs::{Box, LegendCategory, LineChart, Series, svg_to_png};
use chrono::{Duration, Local};
use hex::encode;
use ring::rand::{SecureRandom, SystemRandom};

pub fn draw_chart(
    previous_data: Vec<f32>,
    current_data: Vec<f32>,
    days_count: usize,
) -> Result<(String, Vec<u8>)> {
    let min = previous_data
        .iter()
        .chain(current_data.iter())
        .copied()
        .filter(|v| !v.is_nan())
        .fold(f32::INFINITY, f32::min);

    let period = if days_count == 7 { "week" } else { "month" };
    let mut chart = LineChart::new_with_theme(
        vec![
            Series::new(format!("Last {period}"), previous_data),
            Series::new(format!("Current {period}"), current_data),
        ],
        get_rotated_weekdays(days_count),
        "shadcn",
    );

    chart.legend_category = LegendCategory::Circle;
    chart.series_smooth = true;
    chart.margin = Box {
        bottom: 20.0,
        left: 20.0,
        right: 20.0,
        top: 20.0,
    };

    chart.y_axis_configs[0].axis_min = Some(if min.is_finite() {
        (min - 10.0).max(0.0)
    } else {
        0.0
    });

    let file_path = format!("reports/charts/{}.png", generate_random_id());

    let svg = chart.svg()?;
    let png_bytes = svg_to_png(&svg)?;

    Ok((file_path, png_bytes))
}

fn get_rotated_weekdays(days_count: usize) -> Vec<String> {
    let today = Local::now().date_naive();

    (0..days_count)
        .rev()
        .map(|i| today - Duration::days(i as i64))
        .map(|date| date.format("%d %b").to_string())
        .collect()
}

fn generate_random_id() -> String {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes)
        .expect("Failed to generate random bytes");
    encode(bytes)
}
