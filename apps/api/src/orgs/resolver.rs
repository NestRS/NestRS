use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, Result};
use nestrs_authz::{Create, Read};
use nestrs_authz_graphql::{authorize, bind};
use nestrs_graphql::resolver;
use uuid::Uuid;

use crate::orgs::entity::{self, CreateOrgInput, Org};
use crate::orgs::service::OrgsService;
use crate::users::entity::User;
use crate::users::service::UsersServiceByOrg;

#[resolver]
pub struct OrgsResolver {
    #[inject]
    orgs: Arc<OrgsService>,
}

#[resolver]
impl OrgsResolver {
    #[query]
    async fn orgs(&self, ctx: &Context<'_>) -> Result<Vec<Org>> {
        authorize::<Read, entity::Entity>(ctx)?;
        // Scoped to the caller's org by the ambient ability (an admin sees all).
        let rows = self.orgs.list().await?;
        Ok(rows.iter().map(Org::from).collect())
    }

    #[query]
    async fn org(&self, ctx: &Context<'_>, id: String) -> Result<Option<Org>> {
        // `bind`: parse the id, load the org, instance-check — same shape as the
        // controller's `Bind`.
        Ok(bind::<entity::Entity, Read>(ctx, &id)
            .await?
            .as_ref()
            .map(Org::from))
    }

    #[mutation]
    async fn create_org(&self, ctx: &Context<'_>, input: CreateOrgInput) -> Result<Org> {
        authorize::<Create, entity::Entity>(ctx)?;
        let row = self.orgs.create(input).await?;
        Ok(Org::from(&row))
    }

    #[field]
    async fn users(
        &self,
        parent: &Org,
        by_org: &DataLoader<UsersServiceByOrg>,
    ) -> Result<Vec<User>> {
        let id = Uuid::parse_str(&parent.id)?;
        Ok(by_org.load_one(id).await?.unwrap_or_default())
    }
}
