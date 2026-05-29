//! `mcp` — the **Model Context Protocol** example: a feature exposing an
//! `#[mcp]` tool surface (live Open-Meteo weather) self-mounted at `/mcp`. The
//! composition root is [`AppModule`]; its feature module is crate-private.

mod app;
mod weather;

pub use app::AppModule;
