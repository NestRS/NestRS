use nestrs_core::module;

use crate::authn::guard::AuthGuard;

#[module(providers = [AuthGuard])]
pub struct AuthnModule;
