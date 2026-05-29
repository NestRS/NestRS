use std::sync::Arc;

use nestrs_http::{controller, crud};

use crate::authn::AuthGuard;
use crate::authz::AppAbilityGuard;
use crate::orgs::entity::{self, CreateOrgInput, Org, UpdateOrgInput};
use crate::orgs::service::OrgsService;

#[controller(path = "/orgs")]
#[use_guards(AuthGuard, AppAbilityGuard)]
pub struct OrgsController {
    #[inject]
    svc: Arc<OrgsService>,
}

#[crud(
    service = svc,
    entity = entity::Entity,
    output = Org,
    create = CreateOrgInput,
    update = UpdateOrgInput,
    paginate = cursor,
)]
impl OrgsController {}
