use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, Result};
use nestrs_authz::{Create, Read};
use nestrs_authz_graphql::{authorize, bind};
use nestrs_graphql::resolver;
use uuid::Uuid;

use crate::authn::AuthUser;
use crate::orgs::entity::Org;
use crate::orgs::service::OrgsServiceById;
use crate::users::entity::{self, CreateUserInput, User};
use crate::users::service::{UsersService, UsersServiceByName};

#[resolver]
pub struct UsersResolver {
    #[inject]
    users: Arc<UsersService>,
}

#[resolver]
impl UsersResolver {
    #[query]
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        authorize::<Read, entity::Entity>(ctx)?;
        // Scoped to the caller's org by the ambient ability — no filter by hand.
        let rows = self.users.list().await?;
        Ok(rows.iter().map(User::from).collect())
    }

    #[query]
    async fn user(&self, ctx: &Context<'_>, id: String) -> Result<Option<User>> {
        // `bind` parses the id, loads the entity, and runs the instance check —
        // the resolver analog of the controller's `Bind` parameter.
        Ok(bind::<entity::Entity, Read>(ctx, &id)
            .await?
            .as_ref()
            .map(User::from))
    }

    #[mutation]
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<User> {
        authorize::<Create, entity::Entity>(ctx)?;
        let actor = ctx.data::<AuthUser>()?;
        let row = self.users.create(input, actor.org_id).await?;
        Ok(User::from(&row))
    }

    #[field]
    async fn org(&self, parent: &User, by_id: &DataLoader<OrgsServiceById>) -> Result<Option<Org>> {
        let id = Uuid::parse_str(&parent.org_id)?;
        Ok(by_id.load_one(id).await?)
    }

    #[field]
    async fn namesakes(
        &self,
        parent: &User,
        by_name: &DataLoader<UsersServiceByName>,
    ) -> Result<Vec<User>> {
        let same_name = by_name
            .load_one(parent.name.clone())
            .await?
            .unwrap_or_default();
        // A dataloader runs its batch off the request task, so the ambient
        // ability does not reach it — its read is unscoped. We confine the result
        // to the parent's own org (the parent is already within the caller's
        // scope), so no cross-org row leaks through this field.
        Ok(same_name
            .into_iter()
            .filter(|u| u.id != parent.id && u.org_id == parent.org_id)
            .collect())
    }
}
