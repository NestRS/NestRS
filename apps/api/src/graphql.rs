use nestrs_graphql::graphql;

use crate::users::{UsersMutation, UsersQuery};

#[graphql(
    path = "/graphql",
    queries = [UsersQuery],
    mutations = [UsersMutation],
)]
pub struct AppSchema;
