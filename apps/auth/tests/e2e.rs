//! End-to-end for the authorization server: the OAuth2 `POST /token` endpoint
//! issues a Bearer token, and — the point of the split — a **verify-only** service
//! holding only the public key (what `api` runs) validates it. Driven in-process.

use auth::AppModule;
use identity::{Claims, Role, DEV_PUBLIC_KEY_PEM};
use nestrs_auth::{JwtOptions, JwtService};
use nestrs_testing::TestApp;
use poem::http::StatusCode;

const ORG_ID: &str = "018f0000-0000-7000-8000-000000000000";

async fn boot() -> TestApp {
    TestApp::builder()
        .module::<AppModule>()
        .with_test_telemetry()
        .build()
        .await
        .expect("the auth app boots")
}

/// The resource-server side: a `JwtService` built with ONLY the public key, exactly
/// as `api` configures it. It must verify what `auth` signed with the private key.
fn resource_server_verifier() -> JwtService {
    JwtService::new(JwtOptions::eddsa_verify(DEV_PUBLIC_KEY_PEM))
        .expect("the dev public key parses")
}

#[tokio::test]
async fn token_endpoint_issues_a_token_the_public_key_verifies() {
    let app = boot().await;

    let resp = app
        .http()
        .post("/token")
        .content_type("application/x-www-form-urlencoded")
        .body(format!(
            "grant_type=client_credentials&org_id={ORG_ID}&scope=admin+user"
        ))
        .send()
        .await;
    resp.assert_status_is_ok();

    let json = resp.json().await;
    let obj = json.value().object();
    assert_eq!(obj.get("token_type").string(), "Bearer");
    assert!(obj.get("expires_in").i64() > 0);
    let token = obj.get("access_token").string().to_owned();

    // Verify with the public key alone — the asymmetric trust `api` relies on.
    let claims: Claims = resource_server_verifier()
        .verify(&token)
        .expect("the public key verifies the privately-signed token");
    assert_eq!(claims.org_id.to_string(), ORG_ID);
    assert!(claims.roles.contains(&Role::Admin));
}

#[tokio::test]
async fn token_endpoint_rejects_an_unsupported_grant() {
    let app = boot().await;
    app.http()
        .post("/token")
        .content_type("application/x-www-form-urlencoded")
        .body(format!("grant_type=password&org_id={ORG_ID}"))
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_oauth_authorize_endpoint_redirects_to_the_provider() {
    let app = boot().await;
    let resp = app.http().get("/authorize").send().await;
    resp.assert_status(StatusCode::FOUND);
    resp.assert_header_exist("location");
    resp.assert_header_exist("set-cookie");
}
