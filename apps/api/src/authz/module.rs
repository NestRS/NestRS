use nestrs_authz_http::AbilityGuard;
use nestrs_core::module;

use crate::authz::ability::AppAbility;

pub type AppAbilityGuard = AbilityGuard<AppAbility>;

#[module(providers = [AppAbility, AppAbilityGuard])]
pub struct AuthzModule;
