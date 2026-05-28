//! `TelemetryModule` must not be imported without `Telemetry::init` first — that
//! would register no-op telemetry providers and drop traces/metrics silently, so
//! it panics at boot instead. This runs as its own test binary so no sibling test
//! initialises telemetry and sets the global flag.

use nestrs_core::{Container, Module};
use nestrs_telemetry::TelemetryModule;

#[test]
#[should_panic(expected = "without calling `Telemetry::init`")]
fn importing_the_module_without_init_panics() {
    let _ = TelemetryModule::register(Container::builder());
}
