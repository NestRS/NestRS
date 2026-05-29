use std::sync::Arc;

use nestrs_authz::{Create, Read};
use nestrs_authz_http::{Authorize, Bind};
use nestrs_http::{controller, routes, Ctx, Valid};
use poem::web::Json;
use poem::Result;

use crate::authn::{AuthGuard, AuthUser};
use crate::authz::AppAbilityGuard;
use crate::users::entity::{self, CreateUserInput};
use crate::users::service::UsersService;

#[controller(path = "/users")]
pub struct UsersController {
    #[inject]
    svc: Arc<UsersService>,
}

#[routes]
impl UsersController {
    #[get("/")]
    #[use_guards(AuthGuard, AppAbilityGuard)]
    #[api(summary = "List users in the caller's org", tags("Users"))]
    async fn list(
        &self,
        _authz: Authorize<Read, entity::Entity>,
    ) -> Result<Json<Vec<entity::Model>>> {
        Ok(Json(self.svc.list().await?))
    }

    #[get("/:id")]
    #[use_guards(AuthGuard, AppAbilityGuard)]
    #[api(
        summary = "Fetch a user by id (scoped to the caller's org)",
        tags("Users")
    )]
    async fn get(
        &self,
        _authz: Authorize<Read, entity::Entity>,
        user: Bind<entity::Entity, Read>,
    ) -> Result<Json<entity::Model>> {
        Ok(Json(user.into_inner()))
    }

    #[post("/")]
    #[use_guards(AuthGuard, AppAbilityGuard)]
    #[api(
        summary = "Create a user in the caller's org",
        description = "Requires a bearer JWT (obtain one from `POST /auth/login`).",
        tags("Users")
    )]
    async fn create(
        &self,
        _authz: Authorize<Create, entity::Entity>,
        auth: Ctx<AuthUser>,
        body: Valid<Json<CreateUserInput>>,
    ) -> Result<Json<entity::Model>> {
        let row = self.svc.create(body.into_inner(), auth.org_id).await?;
        Ok(Json(row))
    }
}
