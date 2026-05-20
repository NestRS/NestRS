use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

#[cfg(feature = "otlp")]
use tracing_subscriber::Layer;

use crate::config::TelemetryConfig;
use crate::error::TelemetryError;

/// Active telemetry instance. Returned by [`Telemetry::init`] and dropped at
/// the end of `main` — Drop synchronously flushes pending traces, metrics and
/// logs so trailing telemetry isn't lost on shutdown.
///
/// Keep the binding alive for the whole program: `let _telemetry =
/// Telemetry::init("api")?;`.
pub struct Telemetry {
    #[cfg(feature = "otlp")]
    tracer_provider: Option<opentelemetry_sdk::trace::SdkTracerProvider>,
    #[cfg(feature = "otlp")]
    meter_provider: Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
    #[cfg(feature = "otlp")]
    logger_provider: Option<opentelemetry_sdk::logs::SdkLoggerProvider>,
}

impl Telemetry {
    /// Shortcut: reads env (`RUST_LOG`, `OTEL_EXPORTER_OTLP_ENDPOINT`,
    /// `OTEL_SERVICE_NAME`, …) and wires console logs plus the three OTLP
    /// exporters when an endpoint is set.
    pub fn init(service_name: impl Into<String>) -> Result<Self, TelemetryError> {
        Self::init_with(TelemetryConfig::from_env(service_name))
    }

    pub fn init_with(config: TelemetryConfig) -> Result<Self, TelemetryError> {
        let filter =
            EnvFilter::try_new(&config.log_filter).unwrap_or_else(|_| EnvFilter::new("info"));
        let fmt_layer = tracing_subscriber::fmt::layer();

        #[cfg(feature = "otlp")]
        {
            if let Some(endpoint) = config.otlp_endpoint.clone() {
                let exporters = crate::otlp::build(&config, &endpoint)?;
                let appender_filter = EnvFilter::try_new(&config.log_filter)
                    .unwrap_or_else(|_| EnvFilter::new("info"));
                let appender =
                    opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(
                        &exporters.logger_provider,
                    );
                Registry::default()
                    .with(filter)
                    .with(fmt_layer)
                    .with(tracing_opentelemetry::layer().with_tracer(exporters.tracer))
                    .with(appender.with_filter(appender_filter))
                    .try_init()
                    .map_err(|e| TelemetryError::Init(e.to_string()))?;
                tracing::info!(
                    service = %config.service_name,
                    endpoint = %endpoint,
                    sample_ratio = config.trace_sample_ratio,
                    "telemetry initialised (console + OTLP traces/metrics/logs)"
                );
                return Ok(Telemetry {
                    tracer_provider: Some(exporters.tracer_provider),
                    meter_provider: Some(exporters.meter_provider),
                    logger_provider: Some(exporters.logger_provider),
                });
            }
        }

        Registry::default()
            .with(filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| TelemetryError::Init(e.to_string()))?;
        tracing::info!(service = %config.service_name, "telemetry initialised (console only)");

        Ok(Telemetry {
            #[cfg(feature = "otlp")]
            tracer_provider: None,
            #[cfg(feature = "otlp")]
            meter_provider: None,
            #[cfg(feature = "otlp")]
            logger_provider: None,
        })
    }
}

impl Drop for Telemetry {
    fn drop(&mut self) {
        #[cfg(feature = "otlp")]
        {
            if let Some(p) = self.tracer_provider.take() {
                let _ = p.shutdown();
            }
            if let Some(p) = self.meter_provider.take() {
                let _ = p.shutdown();
            }
            if let Some(p) = self.logger_provider.take() {
                let _ = p.shutdown();
            }
        }
    }
}
