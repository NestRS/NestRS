//! `api` — the **resource-server** example: a REST + GraphQL API that verifies
//! the EdDSA tokens `apps/auth` mints (it holds only the public key) and scopes
//! every read to the caller through the ambient ability. The composition root is
//! [`AppModule`]; the feature modules behind it are crate-private.

mod app;
mod authn;
mod authz;
mod orgs;
mod users;

pub use app::AppModule;
