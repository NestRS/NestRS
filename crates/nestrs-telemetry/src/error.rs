#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("telemetry init failed: {0}")]
    Init(String),
    #[cfg(feature = "otlp")]
    #[error("OTLP exporter build failed: {0}")]
    Otlp(String),
}
