mod codes;

use std::io;

use anyhow::Result;
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, registry};

pub struct Logger {
    level: Level,
    dev_mode: bool,
}

pub use codes::LogCode;

impl Logger {
    pub fn new() -> Self {
        Self {
            level: Level::INFO,
            dev_mode: cfg!(debug_assertions),
        }
    }

    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub fn init(self) -> Result<()> {
        let stdout_layer = fmt::layer()
            .with_writer(io::stdout)
            .with_ansi(self.dev_mode)
            .with_filter(LevelFilter::from_level(self.level));

        registry().with(stdout_layer).init();

        Ok(())
    }
}
