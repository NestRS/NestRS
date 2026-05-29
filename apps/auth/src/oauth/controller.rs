//! The authorization-server endpoints, on the OAuth2-standard paths:
//!
//! - `POST /token` — the **token endpoint** (RFC 6749 §3.2): an
//!   `application/x-www-form-urlencoded` request returns the standard
//!   `{access_token, token_type, expires_in}` envelope. Bearer tokens, signed with
//!   the private key.
//! - `GET /authorize` — the **authorization endpoint** (§3.1): redirects the
//!   user-agent to the upstream provider to begin the OAuth flow.
//! - `GET /callback` — the redirect URI the provider returns to; exchanges the
//!   code and issues this app's token.
//!
//! The handlers hold no logic — they adapt the request to [`TokenIssuer`] and back.

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

/// An OAuth2 token request (`application/x-www-form-urlencoded`). The demo uses the
/// `client_credentials` grant; `scope` carries the requested roles (space-separated,
/// the OAuth convention), and `org_id` the tenant the token authorizes within.
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub org_id: Uuid,
    #[serde(default)]
    pub scope: Option<String>,
}

/// The standard OAuth2 token response envelope (RFC 6749 §5.1).
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
        // Only one grant is wired in the demo; reject the rest per the spec.
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
    async fn authorize(&self) {
        // Unreachable: with no `code`, OAuthGuard challenges with a 302 to the
        // provider before this handler runs.
    }

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
    /// Sign a token and wrap it in the standard OAuth2 response envelope.
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

/// Map OAuth `scope` tokens to roles. Unknown scopes are ignored; an empty scope
/// defaults to a plain `user`.
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
