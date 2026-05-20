//! Telemetry for nestrs applications.
//!
//! Single entry point: [`Telemetry::init`] sets up `tracing` (console fmt
//! always; OTLP exporter when the `otlp` feature is on and
//! `OTEL_EXPORTER_OTLP_ENDPOINT` is set). The returned [`TelemetryGuard`]
//! flushes pending spans on drop, so it must outlive `main`'s work.
//!
//! The [`OtelHttp`] interceptor bridges incoming W3C `traceparent` headers
//! into per-request `tracing` spans, so traces stitch together across
//! services.
//!
//! The W3C Server-Timing response header lives in a separate crate,
//! `nestrs-server-timing`, because its responsibility (browser-visible
//! per-request cost) is unrelated to OpenTelemetry export.

mod config;
mod error;
mod interceptor;
mod module;
#[cfg(feature = "otlp")]
mod otlp;
mod telemetry;

pub use config::TelemetryConfig;
pub use error::TelemetryError;
pub use interceptor::OtelHttp;
#[cfg(feature = "otlp")]
pub use module::Meter;
pub use module::TelemetryModule;
pub use telemetry::Telemetry;
