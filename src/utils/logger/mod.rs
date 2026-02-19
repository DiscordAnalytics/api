mod codes;

use std::{collections::HashMap, io};

use anyhow::Result;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider};
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, registry};

use crate::app_env;

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
