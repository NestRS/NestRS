//! This app's authentication strategy. `api` is a resource server, so it reuses
//! the framework's bearer-JWT strategy ([`JwtStrategy<C>`](nestrs_auth::JwtStrategy))
//! over the shared [`Claims`] contract — there is no hand-written strategy. A
//! custom scheme would implement `nestrs_auth::Strategy` here instead.

use identity::Claims;
use nestrs_auth::JwtStrategy;

/// Verifies the `Bearer` JWT into the shared [`Claims`] (which *are* the caller).
pub type AppJwtStrategy = JwtStrategy<Claims>;
