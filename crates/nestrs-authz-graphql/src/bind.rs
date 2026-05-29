//! [`bind`] — route-model binding for resolvers, the GraphQL analog of
//! `nestrs_authz_http::Bind<E, A>`.

use nestrs_authz::ActionMarker;
use nestrs_graphql::async_graphql::{Context, Error, Result};
use nestrs_orm::current_executor;
use sea_orm::{EntityTrait, PrimaryKeyTrait};
use uuid::Uuid;

use crate::context::{ability, forbidden};

/// Turn a by-id argument into the loaded, authorized entity, so a by-id resolver
/// is a single call instead of a manual parse + load + ability check — the
/// resolver analog of the controller's `Bind<E, A>` parameter. Parses the id as a
/// UUID v7 (a bad id errors), loads `E` through the **ambient request executor**
/// (so it joins the request's transaction), and instance-checks the caller's
/// [`Ability`](nestrs_authz::Ability) for action `A`:
///
/// - no such row → `Ok(None)` (a nullable `user(id)` field resolves to `null`);
/// - the row exists but the ability denies it → a `FORBIDDEN` error (existence is
///   not hidden, matching the HTTP `Bind`);
/// - otherwise → `Ok(Some(model))`.
///
/// Requires the ambient ability (so it doubles as the auth gate — no ability means
/// `FORBIDDEN`); the route needs the GraphQL auth bridge that installs it.
pub async fn bind<E, A>(ctx: &Context<'_>, id: &str) -> Result<Option<E::Model>>
where
    E: EntityTrait,
    E::PrimaryKey: PrimaryKeyTrait<ValueType = Uuid>,
    A: ActionMarker,
{
    let ability = ability(ctx)?;
    let id = Uuid::parse_str(id).map_err(|err| Error::new(err.to_string()))?;
    if id.get_version_num() != 7 {
        return Err(Error::new("id must be a UUID v7"));
    }
    let conn = current_executor()
        .ok_or_else(|| Error::new("no ambient database executor on the GraphQL request"))?;
    let model = E::find_by_id(id)
        .one(&conn)
        .await
        .map_err(|err| Error::new(err.to_string()))?;
    match model {
        Some(model) if ability.can::<E>(A::ACTION, &model) => Ok(Some(model)),
        Some(_) => Err(forbidden()),
        None => Ok(None),
    }
}
