//! [`Bind<E, A>`] — route-model binding: turn a path id into the loaded,
//! authorized entity, so a handler's parameter is a domain object, not a scalar.
//!
//! It folds the three steps a by-id handler used to write by hand into one typed
//! parameter — parse the id, load the row, check the caller may act on it — and
//! short-circuits the request before the handler body runs:
//!
//! - the path id is not a UUID v7 → `400`;
//! - no row with that id → `404`;
//! - the row exists but the caller's [`Ability`] denies action `A` on it → `403`
//!   (the existence is intentionally not hidden, matching the gate's semantics);
//! - otherwise the handler receives the loaded [`EntityTrait::Model`].
//!
//! The load runs through the ambient request executor (`nestrs-orm`), so it joins
//! the request's transaction like any other query. The ability is read from the
//! request extensions, where the [`AbilityGuard`](crate::AbilityGuard) placed it;
//! its absence is a `500` (the guard did not run — a wiring bug).

use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use nestrs_authz::{Ability, ActionMarker};
use nestrs_orm::current_executor;
use poem::http::StatusCode;
use poem::web::Path;
use poem::{Error, FromRequest, Request, RequestBody, Result};
use sea_orm::{EntityTrait, PrimaryKeyTrait};
use uuid::Uuid;

/// The loaded, authorized entity bound from a path id. Declare it as a handler
/// parameter — `user: Bind<users::Entity, Read>` — and read the model via
/// [`Deref`] or own it with [`into_inner`](Bind::into_inner).
pub struct Bind<E: EntityTrait, A>(E::Model, PhantomData<fn() -> A>);

impl<E: EntityTrait, A> Bind<E, A> {
    pub fn into_inner(self) -> E::Model {
        self.0
    }
}

impl<E: EntityTrait, A> Deref for Bind<E, A> {
    type Target = E::Model;
    fn deref(&self) -> &E::Model {
        &self.0
    }
}

impl<'a, E, A> FromRequest<'a> for Bind<E, A>
where
    E: EntityTrait,
    E::PrimaryKey: PrimaryKeyTrait<ValueType = Uuid>,
    A: ActionMarker,
{
    async fn from_request(req: &'a Request, body: &mut RequestBody) -> Result<Self> {
        let Path(id) = Path::<Uuid>::from_request(req, body).await?;
        if id.get_version_num() != 7 {
            return Err(Error::from_string(
                "path id must be a UUID v7",
                StatusCode::BAD_REQUEST,
            ));
        }

        let ability = req.extensions().get::<Arc<Ability>>().ok_or_else(|| {
            Error::from_string(
                "missing request `Ability` — is the ability guard applied to this route?",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let conn = current_executor().ok_or_else(|| {
            Error::from_string(
                "no ambient database executor — is the DbContext interceptor installed?",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        // Load unscoped so a denied-but-existing row yields 403 (not 404); the
        // instance check then enforces the row condition.
        let model = E::find_by_id(id)
            .one(&conn)
            .await
            .map_err(|err| Error::from_string(err.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?
            .ok_or_else(|| Error::from_status(StatusCode::NOT_FOUND))?;

        if ability.can::<E>(A::ACTION, &model) {
            Ok(Bind(model, PhantomData))
        } else {
            Err(Error::from_status(StatusCode::FORBIDDEN))
        }
    }
}
