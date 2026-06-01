//! Configuration failures.

use thiserror::Error;

/// A configuration failure: an environment variable set to a value that does not
/// parse into its target type, or a violation of a config type's declarative
/// `validator` rules (run by [`Config::load`](crate::Config::load) after the
/// explicit `from_env` mapping).
#[derive(Debug, Error)]
pub enum ConfigError {
    /// An environment variable was set but could not be parsed into the field's
    /// type (e.g. `NESTRS_DATABASE__MAX_CONNECTIONS=abc` for a `u32`). Names the
    /// variable so the misconfiguration is obvious at boot.
    #[error("invalid value for {var}: {message}")]
    Parse { var: String, message: String },
    /// A loaded value broke a `#[validate(...)]` rule.
    #[error("configuration validation failed: {0}")]
    Validation(#[from] validator::ValidationErrors),
}

impl ConfigError {
    /// Build a [`Parse`](Self::Parse) error naming the offending variable.
    pub fn parse(var: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Parse {
            var: var.into(),
            message: message.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, ConfigError>;
