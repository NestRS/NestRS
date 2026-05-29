//! HTTP surface bindings for [`nestrs_authz`].
//!
//! `nestrs-authz` is the transport-agnostic authorization engine; this crate is
//! its poem binding, mirroring how `nestrs-http`'s `Valid`/`Piped` bind the pure
//! `nestrs-pipes`. Splitting it out keeps `poem` out of the engine and `sea-orm`
//! (the masking deserializes into `EntityTrait::Model`) out of the generic HTTP
//! crate — each side keeps a single responsibility.
//!
//! The pieces, in request order:
//! - [`AbilityGuard<F>`] — the per-route guard that builds the request `Ability`
//!   from the actor an authentication guard attached.
//! - [`Authorize<A, S>`] — the access gate (a poem extractor): `403` unless the
//!   ability grants action `A` on subject `S`.
//! - [`Bind<E, A>`] — route-model binding: a path id becomes the loaded, authorized
//!   entity (`400`/`404`/`403` short-circuits), so the handler parameter is a domain
//!   object.
//! - [`Scope<E, A>`] — the caller's row-level filter as a `Condition` argument, for
//!   a handler that builds its own query.
//! - [`Authorize`]'s `RouteResponseShaper` impl (in `shape`) — `#[routes]` installs
//!   the ability as ambient state for the handler (so the data layer scopes reads)
//!   and masks the response to the fields and rows the ability permits, with no
//!   `mask` call in the handler.

mod bind;
mod extractor;
mod guard;
mod scope;
mod shape;

pub use bind::Bind;
pub use extractor::Authorize;
pub use guard::AbilityGuard;
pub use scope::Scope;
