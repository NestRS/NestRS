//! [`JwtService`] — sign and verify JSON Web Tokens. The analog of `@nestjs/jwt`'s
//! `JwtService`, and (like that one) a thin wrapper over the standard JWT engine.
//!
//! Configured once via [`AuthModule::for_root`](crate::AuthModule::for_root) and
//! injected as `Arc<JwtService>` anywhere — a login handler signs the token an
//! authenticated caller carries, a [`Strategy`](crate::Strategy) verifies it.

use std::time::Duration;

use jsonwebtoken::{
    decode, encode, errors::ErrorKind, get_current_timestamp, Algorithm, DecodingKey, EncodingKey,
    Header, Validation,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::AuthError;

/// The key material backing a [`JwtService`].
#[derive(Clone)]
pub enum JwtKey {
    /// An HMAC shared secret: the **same** key signs and verifies (`HS*`). Simple,
    /// but every party that can verify can also mint — so it suits a single app,
    /// not a split where a resource server must only validate.
    Hmac(String),
    /// Asymmetric PEM keys (`EdDSA`). `public_pem` always verifies; `private_pem`
    /// signs and is **`None` for a verify-only service** — a resource server that
    /// validates tokens an authorization server minted but can never forge one.
    Pem {
        private_pem: Option<String>,
        public_pem: String,
    },
}

/// How [`JwtService`] signs and verifies. Built at the import site and handed to
/// [`AuthModule::for_root`](crate::AuthModule::for_root).
///
/// Two shapes: a symmetric [`JwtKey::Hmac`] secret (one app signs and verifies),
/// or an asymmetric [`JwtKey::Pem`] EdDSA pair — the authorization server holds the
/// private key ([`eddsa`](Self::eddsa)), every resource server holds only the
/// public key ([`eddsa_verify`](Self::eddsa_verify)), so a resource-server
/// compromise cannot mint tokens.
#[derive(Clone)]
pub struct JwtOptions {
    /// The key material (HMAC secret or EdDSA PEM pair).
    pub key: JwtKey,
    /// The signing/verification algorithm.
    pub algorithm: Algorithm,
    /// How long a freshly minted token stays valid; surfaced by [`JwtService::expiry`].
    pub expires_in: Duration,
}

impl JwtOptions {
    /// HMAC `HS256`, one-hour expiry — the symmetric form (one secret signs and
    /// verifies). Suits a single self-contained app.
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            key: JwtKey::Hmac(secret.into()),
            algorithm: Algorithm::HS256,
            expires_in: Duration::from_secs(3600),
        }
    }

    /// `EdDSA` with a PEM key pair — signs **and** verifies. For an authorization
    /// server (the only place the private key lives).
    pub fn eddsa(private_pem: impl Into<String>, public_pem: impl Into<String>) -> Self {
        Self {
            key: JwtKey::Pem {
                private_pem: Some(private_pem.into()),
                public_pem: public_pem.into(),
            },
            algorithm: Algorithm::EdDSA,
            expires_in: Duration::from_secs(3600),
        }
    }

    /// `EdDSA` **verify-only** with just the public PEM — for a resource server that
    /// validates tokens minted elsewhere and never signs. [`JwtService::sign`]
    /// returns an error on such a service.
    pub fn eddsa_verify(public_pem: impl Into<String>) -> Self {
        Self {
            key: JwtKey::Pem {
                private_pem: None,
                public_pem: public_pem.into(),
            },
            algorithm: Algorithm::EdDSA,
            expires_in: Duration::from_secs(3600),
        }
    }
}

/// Signs and verifies tokens for the app. Keys are precomputed once. A verify-only
/// service (a resource server) has no encoding key, so [`sign`](Self::sign) errors.
pub struct JwtService {
    encoding: Option<EncodingKey>,
    decoding: DecodingKey,
    header: Header,
    validation: Validation,
    expires_in: Duration,
}

impl JwtService {
    /// Precompute the keys from [`JwtOptions`]. Fallible: an EdDSA PEM may not parse
    /// (an HMAC secret cannot fail).
    pub fn new(options: JwtOptions) -> Result<Self, AuthError> {
        let (encoding, decoding) = match &options.key {
            JwtKey::Hmac(secret) => {
                let bytes = secret.as_bytes();
                (
                    Some(EncodingKey::from_secret(bytes)),
                    DecodingKey::from_secret(bytes),
                )
            }
            JwtKey::Pem {
                private_pem,
                public_pem,
            } => {
                let decoding = DecodingKey::from_ed_pem(public_pem.as_bytes())
                    .map_err(|e| AuthError::Failed(format!("invalid JWT public key: {e}")))?;
                let encoding = match private_pem {
                    Some(pem) => {
                        Some(EncodingKey::from_ed_pem(pem.as_bytes()).map_err(|e| {
                            AuthError::Failed(format!("invalid JWT private key: {e}"))
                        })?)
                    }
                    None => None,
                };
                (encoding, decoding)
            }
        };
        let mut validation = Validation::new(options.algorithm);
        // No audience contract by default; an app that wants one sets it on its
        // claims and we can expose `aud` configuration when it is needed.
        validation.validate_aud = false;
        Ok(Self {
            encoding,
            decoding,
            header: Header::new(options.algorithm),
            validation,
            expires_in: options.expires_in,
        })
    }

    /// Sign `claims` into a compact JWT. `claims` must carry an `exp` (use
    /// [`expiry`](Self::expiry)); the rest of the shape is the app's to define.
    /// Errors on a **verify-only** service (no signing key — see
    /// [`JwtOptions::eddsa_verify`]).
    pub fn sign<C: Serialize>(&self, claims: &C) -> Result<String, AuthError> {
        let encoding = self.encoding.as_ref().ok_or_else(|| {
            AuthError::Failed("this JwtService is verify-only — no signing key configured".into())
        })?;
        encode(&self.header, claims, encoding).map_err(|e| AuthError::Failed(e.to_string()))
    }

    /// Verify a token and deserialize its claims. Validates the signature and
    /// `exp`; maps an expired token to [`AuthError::Expired`] and anything else
    /// to [`AuthError::InvalidToken`].
    pub fn verify<C: DeserializeOwned>(&self, token: &str) -> Result<C, AuthError> {
        decode::<C>(token, &self.decoding, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                ErrorKind::ExpiredSignature => AuthError::Expired,
                _ => AuthError::InvalidToken,
            })
    }

    /// The Unix timestamp `now + expires_in` — set it as the `exp` claim when
    /// signing so the token expires per the configured lifetime.
    pub fn expiry(&self) -> u64 {
        get_current_timestamp() + self.expires_in.as_secs()
    }

    /// The configured token lifetime in seconds — the value for an OAuth2 token
    /// response's `expires_in` field.
    pub fn ttl_secs(&self) -> u64 {
        self.expires_in.as_secs()
    }
}
