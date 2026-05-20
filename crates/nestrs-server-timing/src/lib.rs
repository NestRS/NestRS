//! W3C [Server-Timing] interceptor for nestrs.
//!
//! Adds a `Server-Timing` response header so Chrome DevTools (and every
//! other modern browser) renders per-request server cost natively in the
//! Network panel. Independent of OpenTelemetry — this is purely a W3C HTTP
//! concern.
//!
//! ```ignore
//! use nestrs_middleware::EndpointExt;
//! use nestrs_server_timing::ServerTiming;
//!
//! Route::new()
//!     .nest("/", controller)
//!     .interceptor(ServerTiming::new());
//! ```
//!
//! Handlers can record sub-step durations by pulling the [`Timings`]
//! accumulator out of request extensions and calling [`Timings::record`].
//!
//! [Server-Timing]: https://www.w3.org/TR/server-timing/

mod entry;
mod format;
mod interceptor;

pub use entry::{Entry, Timings};
pub use interceptor::ServerTiming;
