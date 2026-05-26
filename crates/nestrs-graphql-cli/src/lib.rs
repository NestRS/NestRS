//! Schema tooling shared by every nestrs GraphQL app.
//!
//! A GraphQL app commits its schema as SDL so the API surface is reviewable in
//! diffs. Because the schema is composed from the resolvers *linked into a
//! binary* (they self-register at link time), it can only be rendered from
//! inside the app — so each app ships a small `schema` binary that calls
//! [`write_schema`]. With multiple apps (and federation, where each app is a
//! subgraph), that logic is identical everywhere; this crate holds it once.
//!
//! The crate is deliberately minimal — just emit. Federation-aware commands
//! (subgraph SDL, composition) land here when federation itself does.

use std::io;
use std::path::Path;

use nestrs_core::Container;
use nestrs_graphql::schema_sdl;

/// Render the GraphQL SDL from the resolvers linked into the calling binary and
/// write it to `path` (the app's committed `schema.graphql`).
///
/// `container` is the caller's: an app whose resolvers inject a
/// `DatabaseConnection` seeds a disconnected stand-in before calling here (the
/// schema is described, never executed). The container must be built inside the
/// app's own binary so the linked-in resolvers are visible.
pub fn write_schema(container: &Container, path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    std::fs::write(path, schema_sdl(container))?;
    println!("wrote {}", path.display());
    Ok(())
}
