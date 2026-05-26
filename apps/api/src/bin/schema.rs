//! Regenerate the committed GraphQL SDL (`schema.graphql`).
//!
//! Run by hand (`cargo run -p api --bin schema`, or `just graphql-schema`); it
//! is deliberately *not* a subcommand of the server binary — `main` only
//! serves. The schema is composed from the resolvers linked into this binary,
//! so the generator lives in the app crate next to them.

use std::process::ExitCode;

use api::AppModule;
use nestrs_core::{Container, Module};
use sea_orm::DatabaseConnection;

fn main() -> ExitCode {
    // Building the container synchronously cannot run the async DB factory, so
    // seed a disconnected connection — the schema is described, never executed —
    // letting the DB-injected providers register.
    let container =
        AppModule::register(Container::builder().provide(DatabaseConnection::default())).build();

    match nestrs_graphql_cli::write_schema(
        &container,
        concat!(env!("CARGO_MANIFEST_DIR"), "/schema.graphql"),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
