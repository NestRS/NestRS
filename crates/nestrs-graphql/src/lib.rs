//! GraphQL support for nestrs.
//!
//! Two-layer model that mirrors `nestrs-http`:
//!
//! - **Per-resolver** — `#[resolver]` builds the resolver from the container
//!   (with `#[inject]` fields). Each resolver stays a regular
//!   `#[async_graphql::Object]` impl block; its kind comes from the
//!   `#[graphql]` list it is named in.
//! - **App-level composition** — `#[graphql(queries=[...],
//!   mutations=[...], subscriptions=[...])]` generates the `MergedObject`
//!   wrappers and a `build(container)` constructor, and attaches a
//!   [`GraphQLResolverMeta`] per listed resolver for discovery / introspection.
//!   Composition is static (async-graphql requires its root types at
//!   compile time), which is why composition lives in a macro at the
//!   app level rather than running off `DiscoveryService` at runtime.

mod resolver;

pub use resolver::{GraphQLResolverMeta, ResolverKind};

pub use async_graphql;
pub use async_graphql_poem;

/// GraphQL decorators (`#[graphql]`, `#[resolver]`), defined in
/// `nestrs-graphql-macros` and surfaced here so apps write
/// `nestrs_graphql::graphql` etc.
pub use nestrs_graphql_macros::{graphql, resolver};
