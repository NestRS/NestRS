use identity::Claims;
use nestrs_core::module;
use nestrs_graphql::OperationGuard;

use crate::authn::AuthnModule;
use crate::authz::ability::AppAbility;
use crate::authz::guard::{AppAbilityGuard, GraphqlAuthGuard};

#[module(
    imports = [AuthnModule],
    providers = [AppAbility, AppAbilityGuard, GraphqlAuthGuard as dyn OperationGuard],
)]
pub struct AuthzModule;

nestrs_graphql::forward_principal!(Claims);
