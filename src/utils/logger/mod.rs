mod codes;

use std::{collections::HashMap, io};

use anyhow::Result;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider};
use tracing::Level;
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{format::FmtSpan, layer},
    layer::SubscriberExt,
    prelude::*,
    registry,
};

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
        let filter = EnvFilter::from_default_env().add_directive(self.level.into());

        let stdout_layer = layer()
            .with_writer(io::stdout)
            .with_ansi(self.dev_mode)
            .with_span_events(FmtSpan::CLOSE)
            .with_filter(filter);

        let env = app_env!();

        if let (Some(endpoint), Some(token), Some(stream)) = (
            env.otlp_endpoint.as_ref(),
            env.otlp_token.as_ref(),
            env.otlp_stream.as_ref(),
        ) && !self.dev_mode
        {
            let mut headers = HashMap::new();
            headers.insert(
                String::from("Authorization"),
                String::from(format!("Basic {}", token)),
            );
            headers.insert("stream-name".to_string(), stream.clone());

            let exporter = LogExporter::builder()
                .with_http()
                .with_protocol(Protocol::HttpBinary)
                .with_headers(headers)
                .with_endpoint(format!("{}/v1/logs", endpoint))
                .build()?;

            let resource = Resource::builder().with_service_name("api").build();

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
