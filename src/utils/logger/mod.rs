mod codes;
mod error;

use std::collections::HashMap;
use std::{
    fs::{create_dir_all, read_dir},
    io,
    path::PathBuf,
};

use anyhow::Result;
use chrono::Local;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider};
use tracing::{Level, level_filters::LevelFilter};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, registry};

use crate::app_env;

pub struct Logger {
    level: Level,
    log_to_file: bool,
    log_dir: PathBuf,
    dev_mode: bool,
}
pub use codes::LogCode;
pub use error::LoggerError;

impl Logger {
    pub fn new() -> Self {
        Self {
            level: Level::INFO,
            log_to_file: true,
            log_dir: PathBuf::from("logs/"),
            dev_mode: cfg!(debug_assertions),
        }
    }

    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub fn init(self) -> Result<()> {
        let stdout_layer = fmt::layer()
            .with_writer(io::stdout)
            .with_ansi(self.dev_mode)
            .with_filter(LevelFilter::from_level(self.level));

        if app_env!().otlp_endpoint.is_some() {
            let mut headers = HashMap::new();
            headers.insert(
                String::from("Authorization"),
                String::from(format!(
                    "Basic {}",
                    app_env!().otlp_token.clone().unwrap_or("".to_string())
                )),
            );
            headers.insert(
                "stream-name".to_string(),
                app_env!().otlp_stream.clone().unwrap_or("".to_string()),
            );

            let exporter = opentelemetry_otlp::LogExporter::builder()
                .with_http()
                .with_protocol(Protocol::HttpBinary)
                .with_headers(headers)
                .with_endpoint(format!(
                    "{}/v1/logs",
                    app_env!().otlp_endpoint.clone().unwrap_or("".to_string())
                ))
                .build()?;

            let resource = Resource::builder()
                .with_service_name(if self.dev_mode { "api-dev" } else { "api" })
                .build();

            let provider = SdkLoggerProvider::builder()
                .with_batch_exporter(exporter)
                .with_resource(resource)
                .build();
            let otlp_layer = OpenTelemetryTracingBridge::new(&provider);

            registry().with(stdout_layer).with(otlp_layer).init();
        } else {
            registry().with(stdout_layer).init()
        }

        Ok(())
    }
}

pub fn get_log_file_for_date(date: &str, log_dir: &str) -> PathBuf {
    PathBuf::from(log_dir).join(format!("{}.log", date))
}

pub fn get_current_log_file(log_dir: &str) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d").to_string();
    get_log_file_for_date(&date, log_dir)
}

pub fn list_log_files(log_dir: &str) -> io::Result<Vec<PathBuf>> {
    let mut log_files = Vec::new();
    for entry in read_dir(log_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            log_files.push(entry.path());
        }
    }
    log_files.sort();
    Ok(log_files)
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub code: String,
    pub message: String,
}

fn parse_log_entries(contents: &str) -> Vec<LogEntry> {
    contents
        .lines()
        .filter_map(|line| parse_log_entry(line))
        .collect()
}

fn parse_log_entry(line: &str) -> Option<LogEntry> {
    let parts: Vec<&str> = line.splitn(4, "]").collect();
    if parts.len() < 4 {
        return None;
    }

    let timestamp = parts[0].trim_start_matches('[').to_string();
    let level = parts[1].trim_start_matches('[').to_string();
    let code = parts[2].trim_start_matches('[').to_string();
    let message = parts[3].trim().to_string();

    Some(LogEntry {
        timestamp,
        level,
        code,
        message,
    })
}
