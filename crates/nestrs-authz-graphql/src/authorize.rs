//! [`authorize`] — the class-level access gate, the GraphQL analog of
//! `nestrs_authz_http::Authorize<A, S>`.

use std::any::TypeId;

use nestrs_authz::{ActionMarker, Subject};
use nestrs_graphql::async_graphql::{Context, Result};

use crate::context::{ability, forbidden};

/// Class-level gate: require action `A` on subject `S`. Returns a GraphQL
/// `forbidden` error (code `FORBIDDEN`) when the caller's ability does not grant
/// it (or when no ability is present — so it doubles as the auth gate).
pub fn authorize<A: ActionMarker, S: Subject>(ctx: &Context<'_>) -> Result<()> {
    if ability(ctx)?.can_class(A::ACTION, TypeId::of::<S>()) {
        Ok(())
    } else {
        Err(forbidden())
    }
}
