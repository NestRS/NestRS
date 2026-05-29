use api::AppModule;
use identity::{Claims, Role};
use nestrs_auth::{JwtOptions, JwtService};
use nestrs_testing::{EphemeralDatabase, TestApp};
use poem::http::{header, StatusCode};
use serde_json::json;
use uuid::Uuid;

const ORG_ID: &str = "018f0000-0000-7000-8000-000000000000";

const DEV_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIEYTRN4vmCuIfaUslO5G9pKyxkDJn3q3t9WDHo2FCfw3\n-----END PRIVATE KEY-----\n";

async fn boot() -> (EphemeralDatabase, TestApp) {
    let db = EphemeralDatabase::create::<db::Migrator>()
        .await
        .expect("create + migrate a throwaway database");
    let app = TestApp::builder()
        .module::<AppModule>()
        .with_test_telemetry()
        .provide_arc(db.connection())
        .build()
        .await
        .expect("AppModule boots against the throwaway database");
    (db, app)
}

async fn login(app: &TestApp) -> String {
    token_for(app, ORG_ID, "admin").await
}

async fn token_for(_app: &TestApp, org_id: &str, role: &str) -> String {
    let jwt = JwtService::new(JwtOptions::eddsa(
        DEV_PRIVATE_KEY,
        identity::DEV_PUBLIC_KEY_PEM,
    ))
    .expect("the dev keypair parses");
    let roles = match role {
        "admin" => vec![Role::Admin],
        _ => vec![Role::User],
    };
    jwt.sign(&Claims {
        org_id: Uuid::parse_str(org_id).expect("valid org uuid"),
        roles,
        exp: jwt.expiry(),
    })
    .expect("sign the test token")
}

async fn create_org(app: &TestApp, bearer: &str, name: &str) -> String {
    let resp = app
        .http()
        .post("/orgs")
        .header(header::AUTHORIZATION, bearer)
        .body_json(&json!({ "name": name }))
        .send()
        .await;
    resp.assert_status_is_ok();
    resp.json()
        .await
        .value()
        .object()
        .get("id")
        .string()
        .to_owned()
}

async fn user_names(app: &TestApp, bearer: &str) -> Vec<String> {
    let listed = app
        .http()
        .get("/users")
        .header(header::AUTHORIZATION, bearer)
        .send()
        .await;
    listed.assert_status_is_ok();
    listed
        .json()
        .await
        .value()
        .array()
        .iter()
        .map(|u| u.object().get("name").string().to_owned())
        .collect()
}

#[tokio::test]
async fn health_live_probe_is_ok() {
    let (_db, app) = boot().await;
    app.http()
        .get("/health/live")
        .send()
        .await
        .assert_status_is_ok();
}

#[tokio::test]
async fn openapi_document_describes_the_routes() {
    let (_db, app) = boot().await;
    let resp = app.http().get("/api-json").send().await;
    resp.assert_status_is_ok();
    let doc = resp.json().await;
    let paths = doc.value().object().get("paths").object();
    assert!(
        paths.get_opt("/orgs").is_some(),
        "OpenAPI paths include /orgs"
    );
    assert!(
        paths.get_opt("/users").is_some(),
        "OpenAPI paths include /users",
    );
}

#[tokio::test]
async fn protected_route_rejects_a_missing_or_bogus_bearer_token() {
    let (_db, app) = boot().await;

    app.http()
        .get("/orgs")
        .send()
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    app.http()
        .get("/orgs")
        .header(header::AUTHORIZATION, "Bearer not-a-real-jwt")
        .send()
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_org_persists_and_is_listed_with_a_bearer_token() {
    let (_db, app) = boot().await;
    let token = login(&app).await;
    let bearer = format!("Bearer {token}");
    let name = "Acme E2E";

    let created = app
        .http()
        .post("/orgs")
        .header(header::AUTHORIZATION, &bearer)
        .body_json(&json!({ "name": name }))
        .send()
        .await;
    created.assert_status_is_ok();
    let created_json = created.json().await;
    assert_eq!(created_json.value().object().get("name").string(), name);

    let listed = app
        .http()
        .get("/orgs")
        .header(header::AUTHORIZATION, &bearer)
        .send()
        .await;
    listed.assert_status_is_ok();
    let names: Vec<String> = listed
        .json()
        .await
        .value()
        .array()
        .iter()
        .map(|org| org.object().get("name").string().to_owned())
        .collect();
    assert!(
        names.contains(&name.to_string()),
        "the freshly created org appears in the list: {names:?}",
    );
}

#[tokio::test]
async fn users_are_scoped_to_their_org_and_bound_by_id() {
    let (_db, app) = boot().await;

    let bootstrap = format!("Bearer {}", token_for(&app, ORG_ID, "admin").await);
    let org_a = create_org(&app, &bootstrap, "Acme").await;
    let org_b = create_org(&app, &bootstrap, "Globex").await;
    let token_a = format!("Bearer {}", token_for(&app, &org_a, "admin").await);
    let token_b = format!("Bearer {}", token_for(&app, &org_b, "admin").await);

    let created = app
        .http()
        .post("/users")
        .header(header::AUTHORIZATION, &token_a)
        .body_json(&json!({ "name": "Ada", "email": "ada@acme.test" }))
        .send()
        .await;
    created.assert_status_is_ok();
    let user_a = created
        .json()
        .await
        .value()
        .object()
        .get("id")
        .string()
        .to_owned();

    assert!(
        user_names(&app, &token_b).await.is_empty(),
        "org B sees none of org A's users",
    );

    app.http()
        .get(format!("/users/{user_a}"))
        .header(header::AUTHORIZATION, &token_b)
        .send()
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.http()
        .get("/users/018f0000-0000-7000-8000-0000000000ff")
        .header(header::AUTHORIZATION, &token_b)
        .send()
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.http()
        .get("/users/not-a-uuid")
        .header(header::AUTHORIZATION, &token_b)
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);

    assert_eq!(user_names(&app, &token_a).await, vec!["Ada".to_string()]);
    let got = app
        .http()
        .get(format!("/users/{user_a}"))
        .header(header::AUTHORIZATION, &token_a)
        .send()
        .await;
    got.assert_status_is_ok();
    assert_eq!(
        got.json().await.value().object().get("name").string(),
        "Ada"
    );
}

#[tokio::test]
async fn a_plain_user_listing_masks_the_email() {
    let (_db, app) = boot().await;
    let bootstrap = format!("Bearer {}", token_for(&app, ORG_ID, "admin").await);
    let org = create_org(&app, &bootstrap, "Initech").await;

    app.http()
        .post("/users")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", token_for(&app, &org, "admin").await),
        )
        .body_json(&json!({ "name": "Bob", "email": "bob@initech.test" }))
        .send()
        .await
        .assert_status_is_ok();

    let user = format!("Bearer {}", token_for(&app, &org, "user").await);
    let listed = app
        .http()
        .get("/users")
        .header(header::AUTHORIZATION, &user)
        .send()
        .await;
    listed.assert_status_is_ok();
    let body = listed.json().await;
    let first = body
        .value()
        .array()
        .iter()
        .next()
        .expect("one user")
        .object();
    assert_eq!(first.get("name").string(), "Bob");
    assert!(
        first.get_opt("email").is_none(),
        "a plain user's listing masks the email field",
    );
}

#[tokio::test]
async fn a_failed_mutation_persists_nothing() {
    let (_db, app) = boot().await;
    let bootstrap = format!("Bearer {}", token_for(&app, ORG_ID, "admin").await);
    let org = create_org(&app, &bootstrap, "Hooli").await;
    let admin = format!("Bearer {}", token_for(&app, &org, "admin").await);

    app.http()
        .post("/users")
        .header(header::AUTHORIZATION, &admin)
        .body_json(&json!({ "name": "Ada", "email": "dup@hooli.test" }))
        .send()
        .await
        .assert_status_is_ok();

    app.http()
        .post("/users")
        .header(header::AUTHORIZATION, &admin)
        .body_json(&json!({ "name": "Grace", "email": "dup@hooli.test" }))
        .send()
        .await
        .assert_status(StatusCode::INTERNAL_SERVER_ERROR);

    assert_eq!(user_names(&app, &admin).await, vec!["Ada".to_string()]);
}

#[tokio::test]
async fn orgs_admin_sees_all_but_a_plain_user_is_scoped_to_its_own() {
    let (_db, app) = boot().await;
    let admin = format!("Bearer {}", token_for(&app, ORG_ID, "admin").await);
    let org_x = create_org(&app, &admin, "OrgX").await;
    let org_y = create_org(&app, &admin, "OrgY").await;

    let admin_list = app
        .http()
        .get("/orgs")
        .header(header::AUTHORIZATION, &admin)
        .send()
        .await;
    admin_list.assert_status_is_ok();
    let admin_names: Vec<String> = admin_list
        .json()
        .await
        .value()
        .array()
        .iter()
        .map(|o| o.object().get("name").string().to_owned())
        .collect();
    assert!(
        admin_names.contains(&"OrgX".to_string()) && admin_names.contains(&"OrgY".to_string()),
        "the admin sees every org: {admin_names:?}",
    );

    let user_x = format!("Bearer {}", token_for(&app, &org_x, "user").await);
    let user_list = app
        .http()
        .get("/orgs")
        .header(header::AUTHORIZATION, &user_x)
        .send()
        .await;
    user_list.assert_status_is_ok();
    let user_names: Vec<String> = user_list
        .json()
        .await
        .value()
        .array()
        .iter()
        .map(|o| o.object().get("name").string().to_owned())
        .collect();
    assert_eq!(user_names, vec!["OrgX".to_string()]);

    app.http()
        .get(format!("/orgs/{org_y}"))
        .header(header::AUTHORIZATION, &user_x)
        .send()
        .await
        .assert_status(StatusCode::FORBIDDEN);
    let got = app
        .http()
        .get(format!("/orgs/{org_x}"))
        .header(header::AUTHORIZATION, &user_x)
        .send()
        .await;
    got.assert_status_is_ok();
    assert_eq!(
        got.json().await.value().object().get("name").string(),
        "OrgX"
    );
}

#[tokio::test]
async fn graphql_requires_a_jwt_and_scopes_to_the_callers_org() {
    let (_db, app) = boot().await;
    let admin = format!("Bearer {}", token_for(&app, ORG_ID, "admin").await);
    let org_a = create_org(&app, &admin, "GqlAcme").await;
    let token_a = format!("Bearer {}", token_for(&app, &org_a, "admin").await);
    let token_b = format!(
        "Bearer {}",
        token_for(&app, &create_org(&app, &admin, "GqlGlobex").await, "admin").await
    );

    let created = app
        .http()
        .post("/users")
        .header(header::AUTHORIZATION, &token_a)
        .body_json(&json!({ "name": "Gql Ada", "email": "gqlada@acme.test" }))
        .send()
        .await;
    created.assert_status_is_ok();
    let user_a = created
        .json()
        .await
        .value()
        .object()
        .get("id")
        .string()
        .to_owned();

    let query = json!({ "query": "{ users { name } }" });

    let anon = app.http().post("/graphql").body_json(&query).send().await;
    anon.assert_status_is_ok();
    assert!(
        anon.json()
            .await
            .value()
            .object()
            .get_opt("errors")
            .is_some(),
        "an anonymous GraphQL query is rejected",
    );

    let b = app
        .http()
        .post("/graphql")
        .header(header::AUTHORIZATION, &token_b)
        .body_json(&query)
        .send()
        .await;
    b.assert_status_is_ok();
    let b_users = b.json().await;
    let b_names: Vec<String> = b_users
        .value()
        .object()
        .get("data")
        .object()
        .get("users")
        .array()
        .iter()
        .map(|u| u.object().get("name").string().to_owned())
        .collect();
    assert!(
        b_names.is_empty(),
        "org B sees no users in GraphQL: {b_names:?}"
    );

    let a = app
        .http()
        .post("/graphql")
        .header(header::AUTHORIZATION, &token_a)
        .body_json(&query)
        .send()
        .await;
    a.assert_status_is_ok();
    let a_users = a.json().await;
    let a_names: Vec<String> = a_users
        .value()
        .object()
        .get("data")
        .object()
        .get("users")
        .array()
        .iter()
        .map(|u| u.object().get("name").string().to_owned())
        .collect();
    assert_eq!(a_names, vec!["Gql Ada".to_string()]);

    let by_id = json!({ "query": format!("{{ user(id: \"{user_a}\") {{ name }} }}") });
    let a_one = app
        .http()
        .post("/graphql")
        .header(header::AUTHORIZATION, &token_a)
        .body_json(&by_id)
        .send()
        .await;
    a_one.assert_status_is_ok();
    assert_eq!(
        a_one
            .json()
            .await
            .value()
            .object()
            .get("data")
            .object()
            .get("user")
            .object()
            .get("name")
            .string(),
        "Gql Ada",
    );
    let b_one = app
        .http()
        .post("/graphql")
        .header(header::AUTHORIZATION, &token_b)
        .body_json(&by_id)
        .send()
        .await;
    b_one.assert_status_is_ok();
    assert!(
        b_one
            .json()
            .await
            .value()
            .object()
            .get_opt("errors")
            .is_some(),
        "org B is forbidden org A's user by id",
    );
}

#[tokio::test]
async fn graphql_namesakes_field_stays_within_the_callers_org() {
    let (_db, app) = boot().await;
    let admin = format!("Bearer {}", token_for(&app, ORG_ID, "admin").await);
    let org_a = create_org(&app, &admin, "NsA").await;
    let org_b = create_org(&app, &admin, "NsB").await;
    let token_a = format!("Bearer {}", token_for(&app, &org_a, "admin").await);
    let token_b = format!("Bearer {}", token_for(&app, &org_b, "admin").await);

    for (tok, email) in [
        (&token_a, "twina@x.test"),
        (&token_b, "twinb@x.test"),
        (&token_a, "twina2@x.test"),
    ] {
        app.http()
            .post("/users")
            .header(header::AUTHORIZATION, tok)
            .body_json(&json!({ "name": "Twin", "email": email }))
            .send()
            .await
            .assert_status_is_ok();
    }

    let resp = app
        .http()
        .post("/graphql")
        .header(header::AUTHORIZATION, &token_a)
        .body_json(&json!({ "query": "{ users { namesakes { email } } }" }))
        .send()
        .await;
    resp.assert_status_is_ok();
    let body = resp.json().await;
    let mut namesake_emails: Vec<String> = Vec::new();
    for user in body
        .value()
        .object()
        .get("data")
        .object()
        .get("users")
        .array()
        .iter()
    {
        for n in user.object().get("namesakes").array().iter() {
            namesake_emails.push(n.object().get("email").string().to_owned());
        }
    }
    assert!(
        !namesake_emails.is_empty(),
        "same-org namesakes still resolve",
    );
    assert!(
        !namesake_emails.contains(&"twinb@x.test".to_string()),
        "org B's user must not leak through the namesakes field: {namesake_emails:?}",
    );
}

#[tokio::test]
async fn crud_generated_update_and_delete_round_trip() {
    let (_db, app) = boot().await;
    let admin = format!("Bearer {}", token_for(&app, ORG_ID, "admin").await);

    let id = create_org(&app, &admin, "Before").await;

    let patched = app
        .http()
        .patch(format!("/orgs/{id}"))
        .header(header::AUTHORIZATION, &admin)
        .body_json(&json!({ "name": "After" }))
        .send()
        .await;
    patched.assert_status_is_ok();
    assert_eq!(
        patched.json().await.value().object().get("name").string(),
        "After"
    );

    let got = app
        .http()
        .get(format!("/orgs/{id}"))
        .header(header::AUTHORIZATION, &admin)
        .send()
        .await;
    got.assert_status_is_ok();
    assert_eq!(
        got.json().await.value().object().get("name").string(),
        "After"
    );

    app.http()
        .delete(format!("/orgs/{id}"))
        .header(header::AUTHORIZATION, &admin)
        .send()
        .await
        .assert_status(StatusCode::NO_CONTENT);

    app.http()
        .get(format!("/orgs/{id}"))
        .header(header::AUTHORIZATION, &admin)
        .send()
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn crud_cursor_pagination_walks_the_collection_in_order() {
    let (_db, app) = boot().await;
    let admin = format!("Bearer {}", token_for(&app, ORG_ID, "admin").await);

    let mut created = Vec::new();
    for i in 0..5 {
        created.push(create_org(&app, &admin, &format!("Page{i}")).await);
    }

    let mut seen: Vec<String> = Vec::new();
    let mut after: Option<String> = None;
    let mut first_page = true;
    loop {
        let path = match &after {
            Some(cursor) => format!("/orgs?first=2&after={cursor}"),
            None => "/orgs?first=2".to_string(),
        };
        let resp = app
            .http()
            .get(&path)
            .header(header::AUTHORIZATION, &admin)
            .send()
            .await;
        resp.assert_status_is_ok();
        if first_page {
            resp.assert_header_exist("x-next-cursor");
            first_page = false;
        }
        let body = resp.json().await;
        let page: Vec<String> = body
            .value()
            .array()
            .iter()
            .map(|o| o.object().get("id").string().to_owned())
            .collect();
        assert!(
            page.len() <= 2,
            "the page respects first=2: got {}",
            page.len()
        );
        if page.is_empty() {
            break;
        }
        after = page.last().cloned();
        seen.extend(page);
        if seen.len() >= created.len() {
            break;
        }
    }

    assert_eq!(seen.len(), 5, "all five orgs are paged through: {seen:?}");
    assert_eq!(
        seen, created,
        "keyset pages preserve ascending-id (chronological) order",
    );
}
