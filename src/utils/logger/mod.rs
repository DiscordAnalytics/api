mod codes;
mod error;

use std::{
    fs::{create_dir_all, read_dir},
    io,
    path::PathBuf,
};

use chrono::Local;
use fern::{
    Dispatch,
    colors::{Color, ColoredLevelConfig},
    log_file,
};
use log::LevelFilter;

pub struct Logger {
    level: LevelFilter,
    log_to_file: bool,
    log_dir: PathBuf,
    dev_mode: bool,
}

pub use codes::LogCode;
pub use error::LoggerError;

impl Logger {
    pub fn new() -> Self {
        Self {
            level: LevelFilter::Info,
            log_to_file: true,
            log_dir: PathBuf::from("logs"),
            dev_mode: cfg!(debug_assertions),
        }
    }

    pub fn level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    pub fn log_to_file(mut self, log_to_file: bool) -> Self {
        self.log_to_file = log_to_file;
        self
    }

    pub fn log_dir(mut self, log_dir: PathBuf) -> Self {
        self.log_dir = log_dir;
        self
    }

    pub fn dev_mode(mut self, dev_mode: bool) -> Self {
        self.dev_mode = dev_mode;
        self
    }

    fn get_log_file_path(&self) -> PathBuf {
        let date = Local::now().format("%Y-%m-%d").to_string();
        self.log_dir.join(format!("{}.log", date))
    }

    pub fn init(self) -> Result<(), LoggerError> {
        let colors = ColoredLevelConfig::new()
            .error(Color::Red)
            .warn(Color::Yellow)
            .info(Color::Green)
            .debug(Color::Blue)
            .trace(Color::BrightBlack);

        let mut base_config = Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "[{}] [{}] {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    colors.color(record.level()),
                    message
                ))
            })
            .level(self.level)
            .level_for("actix_web", LevelFilter::Warn)
            .level_for("actix_server", LevelFilter::Warn)
            .level_for("mongodb", LevelFilter::Warn)
            .level_for("reqwest", LevelFilter::Warn)
            .level_for("rustls", LevelFilter::Warn);

        let stdout_config = Dispatch::new()
            .filter(move |metadata| {
                if self.dev_mode {
                    metadata.level() <= self.level
                } else {
                    metadata.level() <= LevelFilter::Warn
                }
            })
            .chain(io::stdout());
        base_config = base_config.chain(stdout_config);

        if self.log_to_file {
            create_dir_all(&self.log_dir).map_err(|e| LoggerError::FileCreation(e.to_string()))?;

            let log_file_path = self.get_log_file_path();

            let file_config = Dispatch::new().chain(
                log_file(&log_file_path).map_err(|e| LoggerError::FileCreation(e.to_string()))?,
            );
            base_config = base_config.chain(file_config);
        }

        base_config
            .apply()
            .map_err(|e| LoggerError::Initialization(e.to_string()))?;

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
