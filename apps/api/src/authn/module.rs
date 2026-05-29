use nestrs_core::module;

use crate::authn::guard::AuthGuard;
use crate::authn::strategy::AppJwtStrategy;

#[module(providers = [AppJwtStrategy, AuthGuard])]
pub struct AuthnModule;
