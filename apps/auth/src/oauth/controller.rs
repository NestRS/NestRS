use std::sync::Arc;

use nestrs_http::{controller, routes, Ctx};
use nestrs_throttler::{Throttle, ThrottlerGuard};
use poem::http::StatusCode;
use poem::web::{Form, Json};
use poem::{Error, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use identity::Role;

use crate::oauth::service::TokenIssuer;
use crate::oauth::strategy::{Caller, OAuthGuard};

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub org_id: Uuid,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[controller(path = "/")]
pub struct OAuthController {
    #[inject]
    issuer: Arc<TokenIssuer>,
}

#[routes]
impl OAuthController {
    #[post("/token")]
    #[use_guards(ThrottlerGuard)]
    #[meta(Throttle::per_minute(10))]
    #[api(summary = "OAuth2 token endpoint (demo issuer)", tags("OAuth2"))]
    async fn token(&self, body: Form<TokenRequest>) -> Result<Json<TokenResponse>> {
        let TokenRequest {
            grant_type,
            org_id,
            scope,
        } = body.0;
        if grant_type != "client_credentials" {
            return Err(Error::from_string(
                "unsupported_grant_type",
                StatusCode::BAD_REQUEST,
            ));
        }
        let access_token = self.issue(org_id, roles_from_scope(scope.as_deref()))?;
        Ok(Json(access_token))
    }

    #[get("/authorize")]
    #[use_guards(OAuthGuard)]
    #[api(
        summary = "OAuth2 authorization endpoint — redirects to the provider",
        tags("OAuth2")
    )]
    async fn authorize(&self) {}

    #[get("/callback")]
    #[use_guards(OAuthGuard)]
    #[api(
        summary = "OAuth2 redirect URI — issues this app's token",
        tags("OAuth2")
    )]
    async fn callback(&self, caller: Ctx<Caller>) -> Result<Json<TokenResponse>> {
        let access_token = self.issue(caller.org_id, caller.roles.clone())?;
        Ok(Json(access_token))
    }
}

impl OAuthController {
    fn issue(&self, org_id: Uuid, roles: Vec<Role>) -> Result<TokenResponse> {
        let (access_token, expires_in) = self.issuer.issue(org_id, roles).map_err(|err| {
            Error::from_string(err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".into(),
            expires_in,
        })
    }
}

fn roles_from_scope(scope: Option<&str>) -> Vec<Role> {
    let roles: Vec<Role> = scope
        .unwrap_or("")
        .split_whitespace()
        .filter_map(|s| match s {
            "admin" => Some(Role::Admin),
            "user" => Some(Role::User),
            _ => None,
        })
        .collect();
    if roles.is_empty() {
        vec![Role::User]
    } else {
        roles
    }
}
