//! Token issuance — signing the workspace's JWTs. Kept out of the controller,
//! which only adapts the request to this service and the result back to the
//! OAuth2 token response.

use std::sync::Arc;

use anyhow::Result;
use nestrs_auth::JwtService;
use nestrs_core::injectable;
use uuid::Uuid;

use identity::{Claims, Role};

#[injectable]
pub struct TokenIssuer {
    #[inject]
    jwt: Arc<JwtService>,
}

impl TokenIssuer {
    /// Mint a bearer token: build the [`Claims`], stamp the configured expiry, sign
    /// with the **private** key. Returns the token and its lifetime in seconds (the
    /// OAuth2 `expires_in`). A real deployment gates this behind a credential check.
    pub fn issue(&self, org_id: Uuid, roles: Vec<Role>) -> Result<(String, u64)> {
        let token = self.jwt.sign(&Claims {
            org_id,
            roles,
            exp: self.jwt.expiry(),
        })?;
        Ok((token, self.jwt.ttl_secs()))
    }
}
