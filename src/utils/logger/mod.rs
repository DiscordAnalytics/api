mod codes;

#[cfg(feature = "otel")]
use std::collections::HashMap;
use std::io;

use anyhow::Result;
#[cfg(feature = "otel")]
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
#[cfg(feature = "otel")]
use opentelemetry_otlp::{LogExporter, Protocol, WithExportConfig, WithHttpConfig};
#[cfg(feature = "otel")]
use opentelemetry_sdk::{
    Resource,
    logs::{SdkLogger, SdkLoggerProvider},
};
use tracing::Level;
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{format::FmtSpan, layer},
    layer::SubscriberExt,
    prelude::*,
    registry,
};

#[cfg(feature = "otel")]
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

        #[cfg(feature = "otel")]
        {
            let otel_layer = self.otel_layer();
            registry().with(stdout_layer).with(otel_layer).init();
        }
        #[cfg(not(feature = "otel"))]
        {
            registry().with(stdout_layer).init();
        }

        Ok(())
    }

    #[cfg(feature = "otel")]
    fn otel_layer(&self) -> OpenTelemetryTracingBridge<SdkLoggerProvider, SdkLogger> {
        let env = app_env!();

        let mut headers = HashMap::new();
        headers.insert(
            String::from("Authorization"),
            String::from(format!("Basic {}", env.otlp_token)),
        );
        headers.insert("stream-name".to_string(), env.otlp_stream.clone());

        let exporter = LogExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_headers(headers)
            .with_endpoint(format!("{}/v1/logs", env.otlp_endpoint))
            .build()
            .expect("Failed to create OTLP log exporter");

        let resource = Resource::builder().with_service_name("api").build();

        let provider = SdkLoggerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(resource)
            .build();
        let otel_layer = OpenTelemetryTracingBridge::new(&provider);

        otel_layer
    }
}
