use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use nestrs_core::Container;

use crate::users::dto::{CreateUserInput, UserDto};
use crate::users::service::UsersService;

fn users_service(ctx: &Context<'_>) -> Result<Arc<UsersService>> {
    ctx.data::<Container>()?
        .get()
        .ok_or_else(|| async_graphql::Error::new("UsersService is not registered"))
}

fn to_gql_error(error: impl std::fmt::Display) -> async_graphql::Error {
    async_graphql::Error::new(error.to_string())
}

#[derive(Default)]
pub struct UsersQuery;

#[Object]
impl UsersQuery {
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<UserDto>> {
        Ok(users_service(ctx)?.list().await)
    }

    async fn user(&self, ctx: &Context<'_>, id: String) -> Result<Option<UserDto>> {
        users_service(ctx)?.find(&id).await.map_err(to_gql_error)
    }
}

#[derive(Default)]
pub struct UsersMutation;

#[Object]
impl UsersMutation {
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<UserDto> {
        users_service(ctx)?
            .create(input)
            .await
            .map_err(to_gql_error)
    }
}
