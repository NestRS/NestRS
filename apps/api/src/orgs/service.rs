use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use nestrs_core::injectable;
use nestrs_graphql::dataloader;
use nestrs_orm::Repo;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;
use validator::Validate;

use crate::orgs::entity::{self, ActiveModel, CreateOrgInput, Entity as Orgs, Org};

#[injectable]
pub struct OrgsService {
    // Held for the `by_id` dataloader's keyed batch query, which runs outside a
    // request's ambient scope. The request-path methods below reach the database
    // through `Repo`, so they join the request transaction and scope reads to the
    // caller's ability with nothing threaded by hand.
    #[inject]
    db: Arc<DatabaseConnection>,
}

impl OrgsService {
    pub async fn list(&self) -> Result<Vec<entity::Model>> {
        Ok(Repo::<Orgs>::all().await?)
    }

    pub async fn create(&self, input: CreateOrgInput) -> Result<entity::Model> {
        input.validate()?;
        let row = ActiveModel {
            id: Set(Uuid::now_v7()),
            name: Set(input.name),
        };
        let conn = Repo::<Orgs>::conn()?;
        Ok(row.insert(&conn).await?)
    }
}

#[dataloader]
impl OrgsService {
    async fn by_id(&self, ids: &[Uuid]) -> HashMap<Uuid, Org> {
        Orgs::find()
            .filter(entity::Column::Id.is_in(ids.iter().cloned()))
            .all(self.db.as_ref())
            .await
            .unwrap_or_else(|err| {
                tracing::error!(target: "nestrs::loader", error = %err, "by_id loader query failed");
                Vec::new()
            })
            .iter()
            .map(|row| (row.id, Org::from(row)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> OrgsService {
        OrgsService {
            db: Arc::new(DatabaseConnection::default()),
        }
    }

    #[tokio::test]
    async fn create_rejects_empty_name() {
        let err = service()
            .create(CreateOrgInput { name: "".into() })
            .await
            .unwrap_err();
        assert!(err.to_string().contains("name"));
    }
}
