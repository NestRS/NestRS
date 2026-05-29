use std::sync::Arc;

use nestrs_authz::{Create, Read};
use nestrs_authz_http::{Authorize, Bind};
use nestrs_http::{controller, routes, Valid};
use poem::web::Json;
use poem::Result;

use crate::authn::AuthGuard;
use crate::authz::AppAbilityGuard;
use crate::orgs::entity::{self, CreateOrgInput, Org};
use crate::orgs::service::OrgsService;

#[controller(path = "/orgs")]
pub struct OrgsController {
    #[inject]
    svc: Arc<OrgsService>,
}

#[routes]
impl OrgsController {
    #[get("/")]
    #[use_guards(AuthGuard, AppAbilityGuard)]
    #[api(summary = "List organizations the caller may see", tags("Orgs"))]
    async fn list(&self, _authz: Authorize<Read, entity::Entity>) -> Result<Json<Vec<Org>>> {
        // The ambient ability scopes the `Repo` read: an admin lists every org, a
        // tenant member only its own.
        let rows = self.svc.list().await?;
        Ok(Json(rows.iter().map(Org::from).collect()))
    }

    #[get("/:id")]
    #[use_guards(AuthGuard, AppAbilityGuard)]
    #[api(summary = "Fetch an organization by id", tags("Orgs"))]
    async fn get(
        &self,
        _authz: Authorize<Read, entity::Entity>,
        org: Bind<entity::Entity, Read>,
    ) -> Result<Json<Org>> {
        // `Bind` loaded the org and ran the instance check (404 if absent, 403 if
        // the caller may not read it) before this handler.
        Ok(Json(Org::from(&org.into_inner())))
    }

    #[post("/")]
    #[use_guards(AuthGuard, AppAbilityGuard)]
    #[api(summary = "Create an organization", tags("Orgs"))]
    async fn create(
        &self,
        _authz: Authorize<Create, entity::Entity>,
        body: Valid<Json<CreateOrgInput>>,
    ) -> Result<Json<Org>> {
        let row = self.svc.create(body.into_inner()).await?;
        Ok(Json(Org::from(&row)))
    }
}
