//! GraphQL surface for [`nestrs-authz`](nestrs_authz) — the resolver-side analog of
//! `nestrs-authz-http`, structured the same way (one concept per file):
//!
//! - [`bridge`] — [`GraphqlAbilityBridge`], the per-operation guard that
//!   authenticates the request and installs the caller's ambient ability (analog
//!   of HTTP's `AbilityGuard` + `Authorize` shaper); implements
//!   `nestrs-graphql`'s `OperationGuard` seam;
//! - [`context`] — the per-request [`Ability`](nestrs_authz::Ability) bridge into
//!   the GraphQL context (the `ContextSeed` + the [`ability`] accessor);
//! - [`authorize`](authorize()) — the class-level gate (analog of HTTP `Authorize`);
//! - [`bind`](bind()) — route-model binding by id (analog of HTTP `Bind`).
//!
//! ```ignore
//! use nestrs_authz::Read;
//! use nestrs_authz_graphql::{authorize, bind};
//!
//! #[resolver]
//! impl UsersResolver {
//!     #[query]
//!     async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
//!         authorize::<Read, users::Entity>(ctx)?; // gate; `Repo` then scopes the read
//!         // ...
//!     }
//!     #[query]
//!     async fn user(&self, ctx: &Context<'_>, id: String) -> Result<Option<User>> {
//!         Ok(bind::<users::Entity, Read>(ctx, &id).await?.as_ref().map(User::from))
//!     }
//! }
//! ```
//!
//! The caller's `Ability` reaches `/graphql` via the app's GraphQL auth bridge (the
//! guard chain run on that route); without it, `authorize`/`bind` return `FORBIDDEN`.

mod authorize;
mod bind;
mod bridge;
mod context;

pub use authorize::authorize;
pub use bind::bind;
pub use bridge::GraphqlAbilityBridge;
pub use context::ability;
