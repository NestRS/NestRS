//! Authorization for the GraphQL surface, mirroring what the controllers get
//! from `#[use_guards(AuthGuard, AppAbilityGuard)]`.
//!
//! `/graphql` is self-mounted by `GraphqlModule` and carries no per-route guards.
//! Rather than a global guard (which would break the public routes) or a
//! hand-rolled path-matching interceptor, the framework's [`GraphqlAbilityBridge`]
//! plugs into `nestrs-graphql`'s `OperationGuard` seam: the GraphQL endpoint runs
//! it around every operation — authenticate, build the caller's ability, make it
//! ambient so resolver `Repo` reads scope to the caller's org exactly like REST.
//! Each resolver still gates with `nestrs_authz_graphql::authorize::<A, S>(ctx)`.
//!
//! No token (or a bad one) means no ambient ability, so every resolver's
//! `authorize` returns `FORBIDDEN`: GraphQL is closed to anonymous callers. The
//! playground (`GET /graphql`) still loads — it executes no resolver.

use nestrs_authz_graphql::GraphqlAbilityBridge;
use nestrs_core::module;
use nestrs_graphql::{ContextSeed, OperationGuard};

use crate::authn::{AuthGuard, AuthUser, AuthnModule};
use crate::authz::{AppAbilityGuard, AuthzModule};

/// The app's GraphQL operation guard: the framework bridge bound to the app's
/// authentication guard and ability guard — the GraphQL counterpart of the
/// controllers' `#[use_guards(AuthGuard, AppAbilityGuard)]`.
pub type GraphqlAuthGuard = GraphqlAbilityBridge<AuthGuard, AppAbilityGuard>;

/// Wires the GraphQL operation guard, importing the authn/authz modules so the
/// bridge can resolve the same guards the controllers use. Add it to the app's
/// `imports`.
#[module(
    imports = [AuthnModule, AuthzModule],
    providers = [GraphqlAuthGuard as dyn OperationGuard],
)]
pub struct AuthzGraphqlModule;

// Forward the authenticated actor into the GraphQL context, so a mutation can
// create rows in the caller's own org (`ctx.data::<AuthUser>()`), mirroring the
// controllers' `Ctx<AuthUser>`. The `Arc<Ability>` itself is forwarded by
// `nestrs-authz-graphql`'s own seed. App-specific: it names the app's principal.
nestrs_graphql::inventory::submit! {
    ContextSeed {
        seed: |req, _container, gql| match req.extensions().get::<AuthUser>() {
            Some(user) => gql.data(user.clone()),
            None => gql,
        },
    }
}
