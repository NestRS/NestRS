use std::sync::Arc;

use nestrs_core::Container;
use poem::Route;

type MountFn = dyn Fn(&Container, Route) -> Route + Send + Sync;

/// Discovery metadata for a self-mounting HTTP endpoint whose internals are
/// owned by another surface crate — a GraphQL schema, an MCP streamable-HTTP
/// service. Unlike [`crate::HttpControllerMeta`] there is no declarative route
/// table: the closure nests one opaque sub-endpoint at its own path. `path`
/// and `label` are carried only so the transport can list the mount in its
/// boot-time route log.
///
/// [`crate::HttpTransport`] applies these at boot, after controllers, via
/// [`nestrs_core::DiscoveryService::meta`]. So any surface that produces a
/// `poem` endpoint mounts itself simply by being listed in a `#[module]` —
/// no hand-wiring in `main.rs`.
pub struct HttpEndpointMeta {
    path: &'static str,
    label: &'static str,
    mount: Arc<MountFn>,
}

impl HttpEndpointMeta {
    pub fn new<F>(path: &'static str, label: &'static str, mount: F) -> Self
    where
        F: Fn(&Container, Route) -> Route + Send + Sync + 'static,
    {
        Self {
            path,
            label,
            mount: Arc::new(mount),
        }
    }

    /// The path this endpoint nests at — for the boot route log.
    pub fn path(&self) -> &'static str {
        self.path
    }

    /// A short surface tag (`graphql`, `mcp`) — for the boot route log.
    pub fn label(&self) -> &'static str {
        self.label
    }

    /// Nest this endpoint onto `route`, using `container` to build whatever
    /// the surface needs (a schema from container-resolved resolvers, a
    /// per-session MCP handler factory). Called by `HttpTransport`.
    pub fn mount(&self, container: &Container, route: Route) -> Route {
        (self.mount)(container, route)
    }
}
