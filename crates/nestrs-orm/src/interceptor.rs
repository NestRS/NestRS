//! [`DbContext`] — the request boundary that installs the ambient executor.
//!
//! Auto-installed by [`DatabaseModule`](crate::DatabaseModule), it wraps every
//! request *outside* the route's guards, so guards and handlers alike resolve the
//! same ambient [`Executor`](crate::Executor) via [`Repo`](crate::Repo). A **safe**
//! method (GET/HEAD/OPTIONS/TRACE) runs on the pool; a **mutating** method runs in
//! a transaction opened here, committed when the handler answers with a success
//! (2xx) or redirect (3xx) and rolled back otherwise — so a developer never writes
//! a transaction by hand, and a failed mutation never half-persists.

use std::sync::Arc;

use async_trait::async_trait;
use nestrs_http::interceptor;
use nestrs_middleware::{Interceptor, Next};
use poem::http::{Method, StatusCode};
use poem::{Error, Request, Response, Result};
use sea_orm::{DatabaseConnection, TransactionTrait};

use crate::executor::{with_executor, Executor};

#[interceptor]
pub(crate) struct DbContext {
    #[inject]
    db: Arc<DatabaseConnection>,
}

#[async_trait]
impl Interceptor for DbContext {
    async fn intercept(&self, req: Request, next: Next<'_>) -> Result<Response> {
        if is_safe(req.method()) {
            return with_executor(Executor::Pool(self.db.clone()), next.run(req)).await;
        }

        let txn = match self.db.begin().await {
            Ok(txn) => Arc::new(txn),
            Err(err) => {
                tracing::error!(target: "nestrs::orm", error = %err, "failed to open transaction");
                return Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR));
            }
        };

        let result = with_executor(Executor::Txn(txn.clone()), next.run(req)).await;

        // The handler future has completed and its executor clone dropped, so we
        // are normally the sole owner and can take the transaction back to commit
        // or roll it back. A lingering reference (a service that spawned a task
        // holding it) is a bug — drop ours and let `Drop` roll back.
        let txn = match Arc::try_unwrap(txn) {
            Ok(txn) => txn,
            Err(_) => {
                tracing::error!(
                    target: "nestrs::orm",
                    "transaction still referenced after the handler returned — rolling back"
                );
                return result;
            }
        };

        let commit = matches!(
            &result,
            Ok(resp) if resp.status().is_success() || resp.status().is_redirection()
        );
        if commit {
            if let Err(err) = txn.commit().await {
                tracing::error!(target: "nestrs::orm", error = %err, "transaction commit failed");
                return Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR));
            }
        } else if let Err(err) = txn.rollback().await {
            tracing::error!(target: "nestrs::orm", error = %err, "transaction rollback failed");
        }
        result
    }
}

/// HTTP methods that must not mutate state, so they need no transaction.
fn is_safe(method: &Method) -> bool {
    matches!(
        *method,
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    )
}
