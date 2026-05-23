use std::any::TypeId;

use crate::container::ContainerBuilder;

/// Anything a `#[module]` can pull in via `providers = [...]`.
///
/// The macros that decorate a struct (`#[injectable]`, `#[interceptor]`, and
/// future `#[cron_job]`/`#[event_handler]`/`#[mcp_tool]`/…) — together with
/// the `#[routes]` macro on a controller's `impl` block — emit a single
/// `impl Discoverable for Self` per type. The implementation either:
///
/// - registers the value as a provider (`provide` / `provide_dyn`), or
/// - attaches a piece of discovery metadata to the type
///   ([`ContainerBuilder::attach_meta`]), or both.
///
/// `#[module]` then loops over its `providers = [...]` list and calls
/// `<T as Discoverable>::register(builder)` uniformly — it knows nothing
/// about HTTP, MCP, GraphQL, or any future surface.
pub trait Discoverable {
    /// The provider types that must already be registered before
    /// [`register`](Discoverable::register) can build this one. `#[module]`
    /// reads this to register providers in dependency order, so the
    /// `providers = [...]` list can be written in any order.
    ///
    /// The default — no dependencies — fits anything that resolves its
    /// dependencies lazily rather than at registration time: a controller
    /// builds at mount time, a resolver at schema-build time, so neither
    /// needs its dependencies present when `register` runs. Providers built
    /// eagerly (`#[injectable]`, `#[interceptor]`) override this to list the
    /// `TypeId` of each `#[inject]` dependency.
    fn dependencies() -> Vec<TypeId> {
        Vec::new()
    }

    fn register(builder: ContainerBuilder) -> ContainerBuilder;
}
