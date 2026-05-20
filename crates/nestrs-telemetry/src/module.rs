use nestrs_core::{container::ContainerBuilder, module::Module};

/// Telemetry module. Compose with `#[module(imports = [TelemetryModule, ...])]`.
///
/// When the `otlp` feature is on, registers the global OTel [`Meter`] as a
/// provider so services can `#[inject]` it directly:
///
/// ```ignore
/// #[injectable]
/// pub struct UserService {
///     #[inject] meter: std::sync::Arc<nestrs_telemetry::Meter>,
/// }
/// ```
///
/// Without the `otlp` feature, this module is a no-op marker.
///
/// **Ordering:** [`crate::Telemetry::init`] must run before the module is
/// registered, so the global meter provider is installed first.
pub struct TelemetryModule;

impl Module for TelemetryModule {
    fn register(builder: ContainerBuilder) -> ContainerBuilder {
        #[cfg(feature = "otlp")]
        {
            let meter = opentelemetry::global::meter("nestrs");
            return builder.provide_arc(std::sync::Arc::new(Meter(meter)));
        }
        #[allow(unreachable_code)]
        builder
    }
}

/// Wrapper around the global OTel meter so it can be registered as a typed
/// provider in the nestrs container.
#[cfg(feature = "otlp")]
pub struct Meter(pub opentelemetry::metrics::Meter);

#[cfg(feature = "otlp")]
impl std::ops::Deref for Meter {
    type Target = opentelemetry::metrics::Meter;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
