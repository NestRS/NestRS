//! WebSocket surface for `nestrs-authz` — the transparent data context for
//! gateway message handlers.
//!
//! It binds the transparent data layer to the WebSocket surface, the analog of
//! `nestrs-authz-http`'s `Authorize` shaper and `nestrs-authz-graphql`'s
//! `GraphqlAbilityBridge`. A gateway's connection loop runs in a task *after* the
//! upgrade request completes, so the ORM executor and the authz ability the HTTP
//! request installed have unwound by the time a message handler runs. This crate
//! implements `nestrs-ws`'s [`SocketContext`] seam to re-install both around each
//! dispatch, so a socket handler's `Repo` reads run against the request executor
//! and scope to the caller's `Ability` exactly like a controller.
//!
//! Bind it on a gateway's app — import `WsModule`, list the bridge `as dyn
//! SocketContext`, and put the connection guards that build the ability
//! (`AuthGuard` + `AbilityGuard`) on the gateway struct:
//!
//! ```ignore
//! #[gateway(path = "/ws")]
//! #[use_guards(AuthGuard, AppAbilityGuard)]
//! struct UsersGateway { #[inject] users: Arc<UsersService> }
//!
//! #[module(
//!     imports = [WsModule, AuthnModule, AuthzModule, UsersModule],
//!     providers = [UsersGateway, WsDataContext as dyn SocketContext],
//! )]
//! struct UsersWsModule;
//! ```
//!
//! Unlike `GraphqlAbilityBridge`, the bridge does **not** re-run the guard chain:
//! the gateway's connection-level guards already authenticated the handshake and
//! attached the `Ability` to the upgrade request, so the bridge only *captures*
//! it (and the executor) and re-installs it per message. It is therefore not
//! generic over the app's guards.
//!
//! **Scope of this cut.** The executor is bound as the connection **pool**, so
//! every message runs on the pool — there is no per-message transaction (a
//! WebSocket message has no safe/mutating HTTP method to classify). A mutating
//! handler's writes auto-commit individually. Per-message transactions, if
//! wanted, would layer on the same seam.

mod bridge;

pub use bridge::WsDataContext;
