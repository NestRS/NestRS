//! `api` is a resource server: it only **verifies** bearer tokens (the `auth` app
//! issues them). So this module wires just the framework's JWT strategy over the
//! app's `Claims` and the guard that runs it — no token-issuing controller/service.

use nestrs_core::module;

use crate::authn::guard::AuthGuard;
use crate::authn::strategy::AppJwtStrategy;

#[module(providers = [AppJwtStrategy, AuthGuard])]
pub struct AuthnModule;
