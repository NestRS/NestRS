pub mod seed;

mod migrations;

pub use migrations::{migrate, Migrator};
