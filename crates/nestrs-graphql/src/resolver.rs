/// Which root each resolver contributes to. The `#[graphql(queries | mutations
/// | subscriptions = [...])]` macro stamps this on a [`GraphQLResolverMeta`] so
/// introspection tools (or a `/_resolvers` debug endpoint) can list resolvers
/// by kind without reparsing the GraphQL schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolverKind {
    Query,
    Mutation,
    Subscription,
}

/// Discovery metadata the `#[graphql]` macro attaches to each resolver it
/// composes. Informational, not load-bearing — the schema is built statically.
/// Read it via [`nestrs_core::DiscoveryService::meta`]; the resolver's own
/// `TypeId` comes back on the `Discovered::provider_type_id` paired with it,
/// so it is not duplicated here.
pub struct GraphQLResolverMeta {
    pub kind: ResolverKind,
}

impl GraphQLResolverMeta {
    pub fn new(kind: ResolverKind) -> Self {
        Self { kind }
    }
}
