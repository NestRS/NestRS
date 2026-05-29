//! The app's authentication guard — `nestrs_auth::AuthGuard` bound to this app's
//! [`AppJwtStrategy`](crate::authn::strategy::AppJwtStrategy). Bind it on routes
//! with `#[use_guards(AuthGuard, …)]`; an `AbilityGuard` follows it to authorize.

use crate::authn::strategy::AppJwtStrategy;

pub type AuthGuard = nestrs_auth::AuthGuard<AppJwtStrategy>;
