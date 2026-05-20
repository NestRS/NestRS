use std::env;

/// Configuration for [`crate::Telemetry::init`].
///
/// Construct via [`TelemetryConfig::new`] (just a service name) or
/// [`TelemetryConfig::from_env`] (also reads the standard `OTEL_*` /
/// `RUST_LOG` env vars). The OTel exporter is wired only when
/// [`Self::otlp_endpoint`] is `Some`; otherwise telemetry stays console-only.
#[derive(Clone, Debug)]
pub struct TelemetryConfig {
    /// `service.name` resource attribute.
    pub service_name: String,
    /// `service.version`. Defaults to `CARGO_PKG_VERSION` of the app crate
    /// when unspecified (see [`Self::from_env`]).
    pub service_version: Option<String>,
    /// `deployment.environment`. Free-form (`prod`, `staging`, `dev`).
    pub deployment_environment: Option<String>,
    /// `service.instance.id`. Defaults to a fresh UUID v7 per process — so
    /// restarts produce distinct identities in the backend.
    pub service_instance_id: Option<String>,
    /// `RUST_LOG`-style filter applied to the console layer **and** the OTel
    /// log appender — same gate for both.
    pub log_filter: String,
    /// OTLP base endpoint (e.g. `http://localhost:4318`). The exporter
    /// appends `/v1/traces`, `/v1/metrics`, `/v1/logs` per signal.
    pub otlp_endpoint: Option<String>,
    /// Head-based sample ratio in `[0.0, 1.0]`. `1.0` keeps every trace;
    /// pick `0.05` or `0.1` in prod. Wrapped in `ParentBased` so child
    /// spans inherit the parent's sampling decision.
    pub trace_sample_ratio: f64,
}

impl TelemetryConfig {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            service_version: None,
            deployment_environment: None,
            service_instance_id: None,
            log_filter: "info".into(),
            otlp_endpoint: None,
            trace_sample_ratio: 1.0,
        }
    }

    /// Pull standard env vars: `OTEL_SERVICE_NAME`, `OTEL_SERVICE_VERSION`,
    /// `OTEL_DEPLOYMENT_ENVIRONMENT`, `OTEL_SERVICE_INSTANCE_ID`, `RUST_LOG`,
    /// `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_TRACES_SAMPLER_ARG`.
    pub fn from_env(service_name: impl Into<String>) -> Self {
        let mut cfg = Self::new(service_name);
        if let Ok(v) = env::var("OTEL_SERVICE_NAME") {
            cfg.service_name = v;
        }
        cfg.service_version = env::var("OTEL_SERVICE_VERSION").ok();
        cfg.deployment_environment = env::var("OTEL_DEPLOYMENT_ENVIRONMENT").ok();
        cfg.service_instance_id = env::var("OTEL_SERVICE_INSTANCE_ID").ok();
        if let Ok(v) = env::var("RUST_LOG") {
            cfg.log_filter = v;
        }
        cfg.otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
        if let Ok(ratio) = env::var("OTEL_TRACES_SAMPLER_ARG") {
            if let Ok(r) = ratio.parse::<f64>() {
                cfg.trace_sample_ratio = r.clamp(0.0, 1.0);
            }
        }
        cfg
    }

    pub fn with_log_filter(mut self, filter: impl Into<String>) -> Self {
        self.log_filter = filter.into();
        self
    }

    pub fn with_otlp_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.otlp_endpoint = Some(endpoint.into());
        self
    }

    pub fn with_service_version(mut self, version: impl Into<String>) -> Self {
        self.service_version = Some(version.into());
        self
    }

    pub fn with_deployment_environment(mut self, env: impl Into<String>) -> Self {
        self.deployment_environment = Some(env.into());
        self
    }

    pub fn with_trace_sample_ratio(mut self, ratio: f64) -> Self {
        self.trace_sample_ratio = ratio.clamp(0.0, 1.0);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_sample_everything() {
        let cfg = TelemetryConfig::new("svc");
        assert_eq!(cfg.trace_sample_ratio, 1.0);
        assert!(cfg.otlp_endpoint.is_none());
        assert_eq!(cfg.log_filter, "info");
    }

    #[test]
    fn ratio_is_clamped() {
        let cfg = TelemetryConfig::new("svc").with_trace_sample_ratio(2.5);
        assert_eq!(cfg.trace_sample_ratio, 1.0);
        let cfg = TelemetryConfig::new("svc").with_trace_sample_ratio(-1.0);
        assert_eq!(cfg.trace_sample_ratio, 0.0);
    }
}
