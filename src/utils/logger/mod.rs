mod codes;
mod error;

use std::{
    fs::{create_dir_all, read_dir},
    io,
    path::PathBuf,
};

use chrono::Local;
use tracing::{Level, level_filters::LevelFilter};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, registry};

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

    pub fn init(self) -> Result<Option<WorkerGuard>, LoggerError> {
        let stdout_layer = fmt::layer()
            .with_writer(io::stdout)
            .with_ansi(self.dev_mode)
            .with_filter(LevelFilter::from_level(self.level));

        let guard = if self.log_to_file {
            create_dir_all(&self.log_dir).map_err(|e| LoggerError::FileCreation(e.to_string()))?;

            let file_appender = RollingFileAppender::builder()
                .rotation(Rotation::DAILY)
                .filename_suffix("api.log")
                .build(self.log_dir)
                .map_err(|e| LoggerError::Initialization(e.to_string()))?;
            let (file_writter, guard) = tracing_appender::non_blocking(file_appender);

            let file_layer = fmt::layer()
                .with_writer(file_writter)
                .with_ansi(false)
                .with_filter(LevelFilter::from_level(self.level));

            registry().with(stdout_layer).with(file_layer).init();

            Some(guard)
        } else {
            registry().with(stdout_layer).init();
            None
        };

        Ok(guard)
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
