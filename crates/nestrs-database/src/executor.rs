//! The ambient, request-scoped database executor.
//!
//! A request's queries should run against *whichever* connection the request is
//! currently bound to — the pool normally, or a [`DatabaseTransaction`] when one
//! is open — without the service knowing which. [`Executor`] is that handle: a
//! pool or a transaction, both [`ConnectionTrait`], carried in a task-local that
//! the [`DbContext`](crate::DbContext) interceptor installs per request and
//! [`Repo`](crate::Repo) reads back.
//!
//! SeaORM's `ConnectionTrait` has generic methods, so it is not object-safe (no
//! `&dyn ConnectionTrait`). The enum sidesteps that: it implements the trait by
//! forwarding the non-generic methods to whichever variant it holds, so a single
//! `&Executor` drives any SeaORM query exactly like a concrete connection.

use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr, ExecResult,
    QueryResult, Statement,
};

/// The connection a request's queries run against: the shared pool, or the
/// per-request [`DatabaseTransaction`]. Cheap to clone (an `Arc` either way).
#[derive(Clone)]
pub enum Executor {
    /// The shared connection pool — the default outside a transaction.
    Pool(Arc<DatabaseConnection>),
    /// The current request's transaction (opened by the [`DbContext`](crate::DbContext)
    /// interceptor for a mutating request).
    Txn(Arc<DatabaseTransaction>),
}

#[async_trait]
impl ConnectionTrait for Executor {
    fn get_database_backend(&self) -> DbBackend {
        match self {
            Executor::Pool(c) => c.get_database_backend(),
            Executor::Txn(t) => t.get_database_backend(),
        }
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            Executor::Pool(c) => c.execute_raw(stmt).await,
            Executor::Txn(t) => t.execute_raw(stmt).await,
        }
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        match self {
            Executor::Pool(c) => c.execute_unprepared(sql).await,
            Executor::Txn(t) => t.execute_unprepared(sql).await,
        }
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            Executor::Pool(c) => c.query_one_raw(stmt).await,
            Executor::Txn(t) => t.query_one_raw(stmt).await,
        }
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            Executor::Pool(c) => c.query_all_raw(stmt).await,
            Executor::Txn(t) => t.query_all_raw(stmt).await,
        }
    }

    fn support_returning(&self) -> bool {
        match self {
            Executor::Pool(c) => c.support_returning(),
            Executor::Txn(t) => t.support_returning(),
        }
    }

    fn is_mock_connection(&self) -> bool {
        match self {
            Executor::Pool(c) => c.is_mock_connection(),
            Executor::Txn(t) => t.is_mock_connection(),
        }
    }
}

tokio::task_local! {
    static EXECUTOR: Executor;
}

/// The ambient [`Executor`] for the current request, or `None` outside one (no
/// [`DbContext`](crate::DbContext) interceptor has run — a non-request context).
pub fn current_executor() -> Option<Executor> {
    EXECUTOR.try_with(Clone::clone).ok()
}

/// Run `fut` with `executor` installed as the ambient request executor, so
/// [`current_executor`] (and [`Repo`](crate::Repo)) resolve to it throughout.
pub async fn with_executor<F: Future>(executor: Executor, fut: F) -> F::Output {
    EXECUTOR.scope(executor, fut).await
}
