use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, MergedObject, Schema,
};
use nestrs_core::Container;
use poem::{handler, web::Html};

use crate::users::{UsersMutation, UsersQuery};

#[derive(MergedObject, Default)]
pub struct Query(UsersQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(UsersMutation);

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

// The IoC container is attached as schema data so resolvers can pull their
// dependencies via `ctx.data::<Container>()`.
pub fn build_schema(container: Container) -> AppSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(container)
        .finish()
}

#[handler]
pub async fn playground() -> Html<String> {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}
