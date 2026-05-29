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
    pub fn issue(&self, org_id: Uuid, roles: Vec<Role>) -> Result<(String, u64)> {
        let token = self.jwt.sign(&Claims {
            org_id,
            roles,
            exp: self.jwt.expiry(),
        })?;
        Ok((token, self.jwt.ttl_secs()))
    }
}
