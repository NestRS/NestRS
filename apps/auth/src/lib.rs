//! `auth` — the dedicated authorization server. It owns *token issuance* (the
//! OAuth2 `POST /token` endpoint and the OAuth redirect flow) for the whole
//! workspace, deployed on its own subdomain (`auth.*`). The `api` app (`api.*`) is
//! a pure resource server that only **verifies** the tokens this app mints.
//!
//! The two apps are decoupled: they share the [`identity`] claims contract and the
//! workspace database, but `api` never calls `auth` at runtime — it validates a
//! self-contained JWT with the **public** key, while the **private** signing key
//! lives only here. That key isolation is the security point of the split.

mod app;
mod oauth;

pub use app::AppModule;
