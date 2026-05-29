//! [`Repo`] — the query entry point that makes security and transactions
//! transparent.
//!
//! A service queries through `Repo::<E>` instead of holding a connection, so two
//! cross-cutting concerns disappear from its code:
//!
//! - **Transactions** — every `Repo` call runs against the *ambient*
//!   [`Executor`](crate::Executor), which is the request's transaction when one
//!   is open. The service never threads a transaction handle.
//! - **Row-level security** — every *read* is filtered by the caller's
//!   [`Ability`](nestrs_authz::Ability) (`condition_for`), read from the ambient
//!   request ability. A feature cannot forget to scope its reads to what the
//!   caller may see; with no ambient ability the filter is the SQL identity
//!   (`TRUE`), so non-request and unauthenticated paths read unscoped.
//!
//! `Repo` requires an ambient executor (the [`DbContext`](crate::DbContext)
//! interceptor installs it per request); a call outside that scope errors rather
//! than silently reaching a connection it does not have. For a write or a custom
//! query, take the ambient executor with [`Repo::conn`] and drive SeaORM directly.

use std::marker::PhantomData;

use nestrs_authz::{current_ability, Action};
use sea_orm::sea_query::Condition;
use sea_orm::{DbErr, EntityTrait, PrimaryKeyTrait, QueryFilter, Select};

use crate::executor::{current_executor, Executor};

/// The caller's row-level filter for `action` on `E`, taken from the ambient
/// [`Ability`](nestrs_authz::Ability). With no ambient ability it is
/// [`Condition::all`] — the SQL identity (`TRUE`), i.e. unscoped.
pub fn scope_for<E: EntityTrait>(action: Action) -> Condition {
    current_ability()
        .map(|ability| ability.condition_for::<E>(action))
        .unwrap_or_else(Condition::all)
}

/// Repository over entity `E`, bound to the ambient request executor and ability.
/// Zero-sized — its methods are associated functions named at the call site
/// (`Repo::<users::Entity>::all()`).
pub struct Repo<E: EntityTrait>(PhantomData<fn() -> E>);

impl<E: EntityTrait> Repo<E> {
    /// The ambient request executor (the transaction when one is open, else the
    /// pool), for a write or a custom query: `active.insert(&Repo::<E>::conn()?)`.
    pub fn conn() -> Result<Executor, DbErr> {
        current_executor().ok_or_else(|| {
            DbErr::Custom(
                "no ambient database executor — a Repo query must run inside the request \
                 scope installed by nestrs-orm's DbContext interceptor"
                    .to_owned(),
            )
        })
    }

    /// Every row of `E` the caller may [`Read`](Action::Read).
    pub async fn all() -> Result<Vec<E::Model>, DbErr> {
        let conn = Self::conn()?;
        E::find()
            .filter(scope_for::<E>(Action::Read))
            .all(&conn)
            .await
    }

    /// A row by primary key, returned only if the caller may [`Read`](Action::Read)
    /// it — a row outside the caller's scope reads as `None`, never leaking its
    /// existence.
    pub async fn find_by_id(
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Result<Option<E::Model>, DbErr> {
        let conn = Self::conn()?;
        E::find_by_id(id)
            .filter(scope_for::<E>(Action::Read))
            .one(&conn)
            .await
    }

    /// A [`Select`] pre-filtered to what the caller may `action`, for a custom
    /// query. Chain further constraints and execute against [`Repo::conn`], e.g.
    /// `Repo::<E>::scoped(Action::Update).all(&Repo::<E>::conn()?)`.
    pub fn scoped(action: Action) -> Select<E> {
        E::find().filter(scope_for::<E>(action))
    }
}
