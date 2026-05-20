//! Named middleware vocabulary for nestrs.
//!
//! Poem ships a single `Middleware` trait — powerful but type-erased of
//! intent. nestrs layers three named categories on top, mirroring the
//! NestJS mental model:
//!
//! - [`Interceptor`] — wraps handler execution (logging, metrics, response
//!   transformation). The closest equivalent to a Poem middleware.
//! - [`Guard`] — pre-handler authorization. Short-circuits with a
//!   [`Response`](poem::Response) when access is denied.
//! - [`Filter`] — converts errors returned by inner endpoints into
//!   responses (exception handling).
//!
//! All three plug into a route via the [`EndpointExt`] extension trait:
//!
//! ```ignore
//! use nestrs_middleware::EndpointExt;
//!
//! Route::new()
//!     .nest("/", controller)
//!     .interceptor(MyInterceptor::new())
//!     .guard(MyGuard::new())
//!     .filter(MyFilter::new());
//! ```
//!
//! For escape-hatch needs that don't fit any category, the raw
//! [`poem::Middleware`] remains available via Poem's `.with()`.

mod ext;
mod filter;
mod guard;
mod interceptor;

pub use ext::EndpointExt;
pub use filter::{Filter, RequestSnapshot};
pub use guard::Guard;
pub use interceptor::{Interceptor, Next};
