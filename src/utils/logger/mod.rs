mod codes;

#[cfg(feature = "otel")]
use std::collections::HashMap;
use std::io;

use anyhow::Result;
use tracing::level_filters::LevelFilter;
#[cfg(not(feature = "otel"))]
use tracing_subscriber::layer::Identity;
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{format::FmtSpan, layer},
    layer::SubscriberExt,
    prelude::*,
    registry,
};
#[cfg(feature = "otel")]
use {
    opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge,
    opentelemetry_otlp::{LogExporter, Protocol, WithExportConfig, WithHttpConfig},
    opentelemetry_sdk::{
        Resource,
        logs::{SdkLogger, SdkLoggerProvider},
    },
};

#[cfg(feature = "otel")]
use crate::app_env;

pub struct Logger {
    dev_mode: bool,
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

pub use codes::LogCode;

impl Logger {
    pub fn new() -> Self {
        Self {
            dev_mode: cfg!(debug_assertions),
        }
    }

    pub fn init(self) -> Result<()> {
        let filter = EnvFilter::try_from_default_env().unwrap_or(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        );

        let stdout_layer = layer()
            .with_writer(io::stdout)
            .with_ansi(self.dev_mode)
            .with_span_events(FmtSpan::CLOSE)
            .with_filter(filter);

        #[cfg(feature = "otel")]
        let otel_layer = (!self.dev_mode).then(|| self.otel_layer()).transpose()?;
        #[cfg(not(feature = "otel"))]
        let otel_layer: Option<Identity> = None;

        registry().with(stdout_layer).with(otel_layer).init();

        Ok(())
    }

    #[cfg(feature = "otel")]
    fn otel_layer(&self) -> Result<OpenTelemetryTracingBridge<SdkLoggerProvider, SdkLogger>> {
        let env = app_env!();

        let mut headers = HashMap::new();
        headers.insert(
            String::from("Authorization"),
            format!("Bearer {}", env.otlp_token),
        );
        headers.insert("stream-name".to_string(), env.otlp_stream.clone());

        let exporter = LogExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_headers(headers)
            .with_endpoint(env.otlp_endpoint.clone())
            .build()?;

        let resource = Resource::builder().with_service_name("api").build();

        let provider = SdkLoggerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(resource)
            .build();
        let otel_layer = OpenTelemetryTracingBridge::new(&provider);

        Ok(otel_layer)
    }
}
