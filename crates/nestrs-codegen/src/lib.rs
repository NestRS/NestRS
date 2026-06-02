//! Shared helpers for nestrs decorator macros.
//!
//! A procedural macro must live in a `proc-macro = true` crate, which can
//! export nothing but macros — so the token-building logic every decorator
//! shares (`#[injectable]`-style construction, the `from_container`
//! constructor, the `Discoverable::dependencies` list) lives here, in a plain
//! library crate that each `nestrs-*-macros` crate depends on. Third-party
//! decorator crates can depend on it too.
//!
//! # Modules
//!
//! - [`mod@args`] — attribute-argument parsing. Today: `parse_named_str_arg`
//!   for the common single-string decorator (`#[controller(path = "…")]`,
//!   `#[cron_job(every = "…")]`). Use it instead of hand-writing a parser
//!   when a new decorator takes one `key = "value"` argument.
//! - [`mod@crud`] — the shared parser for `#[crud(...)]`, consumed by both
//!   `nestrs-http-macros` and `nestrs-graphql-macros`. Adds new options
//!   *once* (`readonly`, `paginate`, …) and every CRUD generator sees them.
//! - [`mod@inject`] — `#[injectable]` body construction, the
//!   `from_container` / `dependencies` / `injected` methods every
//!   provider-emitting decorator must produce. The biggest module: any
//!   decorator that produces a `Discoverable` provider goes through here
//!   so the access graph sees its injected types.
//! - [`mod@ty`] — type-path inspection (peel a `Path<Uuid>` from a poem
//!   extractor, label `dyn Trait` for a diagnostic, find the `Arc<T>`
//!   inner). Stateless, no token emission.
//!
//! # When to extend
//!
//! When writing a new decorator macro:
//! 1. Use the existing helpers (`build_injectable_body`,
//!    `dependencies_method`, `parse_named_str_arg`, `nth_generic_type`) so
//!    the new decorator's emitted code agrees with everyone else's on
//!    constructor name, dependency list, and error wording.
//! 2. If a helper does not exist, **add it here**, not in the macro crate.
//!    A shared helper is reusable by third-party decorators; a private
//!    helper in `*-macros` is not.
//! 3. Keep this crate token-only — never depend on `nestrs-core` or any
//!    `nestrs-*` surface crate. The path tokens emitted (`::nestrs_core::*`)
//!    are resolved at the macro's *call site*, so there is no compile-time
//!    cycle.

mod args;
mod crud;
mod inject;
mod ty;

pub use args::parse_named_str_arg;
pub use crud::{parse_crud_args, singular_of, CrudConfig, Paginate};
pub use inject::{
    build_injectable_body, dependencies_method, dependency_names_method, forwarded_arg_idents,
    forwarded_idents, from_container_method, injected_keys_expr, injected_keys_with_layers,
    injected_method, injected_method_with_layers, layer_inject_keys, optional_dependencies_method,
    InjectableBody,
};
pub use ty::{impl_self_ident, nth_generic_type};
