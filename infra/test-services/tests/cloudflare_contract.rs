#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "test helpers outside #[test] functions, where clippy's test allowances don't reach"
)]

use actix_web::http::StatusCode;
use actix_web::{App, test};
use serde_json::{Value, json};
use test_services::cloudflare_saas::{State, configure};

const TOKEN: &str = "contract-token";
const ZONE: &str = "/client/v4/zones/contract-zone/custom_hostnames";

fn contract() -> Value {
    serde_json::from_str(include_str!("../contract/cloudflare-custom-hostnames.json")).unwrap()
}

fn against(schema: &Value, body: &Value) -> Vec<String> {
    let validator = jsonschema::validator_for(schema).unwrap();
    validator
        .iter_errors(body)
        .map(|error| format!("{} at {}", error, error.instance_path()))
        .collect()
}

fn violations(operation: &str, status: &str, body: &Value) -> Vec<String> {
    against(&contract()["operations"][operation][status], body)
}

fn failure_violations(operation: &str, body: &Value) -> Vec<String> {
    against(
        &contract()["operations"][operation]["4XX"]["allOf"][1],
        body,
    )
}

#[actix_web::test]
async fn cloudflares_failure_schemas_still_demand_success() {
    let real_failure = json!({
        "result": null,
        "success": false,
        "errors": [{ "code": 1436, "message": "The custom hostname was not found." }],
        "messages": []
    });
    for operation in ["list", "create"] {
        assert!(
            !violations(operation, "4XX", &real_failure).is_empty(),
            "{operation}: Cloudflare's 4XX schema now accepts a real failure; validate against it whole"
        );
        assert_eq!(
            failure_violations(operation, &real_failure),
            Vec::<String>::new(),
            "{operation}"
        );
    }
    assert_eq!(
        violations("delete", "4XX", &real_failure),
        Vec::<String>::new()
    );
}

#[actix_web::test]
async fn every_response_the_fake_gives_matches_cloudflares_schema() {
    let app = test::init_service(
        App::new()
            .app_data(State::new(TOKEN.to_string()))
            .configure(configure),
    )
    .await;
    let bearer = ("Authorization", format!("Bearer {TOKEN}"));

    let created = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(ZONE)
            .insert_header(bearer.clone())
            .set_json(json!({ "hostname": "a.test", "ssl": { "method": "http", "type": "dv" } }))
            .to_request(),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let created: Value = test::read_body_json(created).await;
    assert_eq!(violations("create", "200", &created), Vec::<String>::new());

    let listed = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("{ZONE}?hostname.exact=a.test"))
            .insert_header(bearer.clone())
            .to_request(),
    )
    .await;
    assert_eq!(listed.status(), StatusCode::OK);
    let listed: Value = test::read_body_json(listed).await;
    assert_eq!(violations("list", "200", &listed), Vec::<String>::new());

    test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/fake/rejections/b.test")
            .set_json(json!({ "message": "Refused" }))
            .to_request(),
    )
    .await;
    let refused = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(ZONE)
            .insert_header(bearer.clone())
            .set_json(json!({ "hostname": "b.test", "ssl": { "method": "http", "type": "dv" } }))
            .to_request(),
    )
    .await;
    assert_eq!(refused.status(), StatusCode::BAD_REQUEST);
    let refused: Value = test::read_body_json(refused).await;
    assert_eq!(failure_violations("create", &refused), Vec::<String>::new());

    let id = created["result"]["id"].as_str().unwrap().to_string();
    let deleted = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri(&format!("{ZONE}/{id}"))
            .insert_header(bearer.clone())
            .to_request(),
    )
    .await;
    assert_eq!(deleted.status(), StatusCode::OK);
    let deleted: Value = test::read_body_json(deleted).await;
    assert_eq!(violations("delete", "200", &deleted), Vec::<String>::new());

    let missing = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri(&format!("{ZONE}/{id}"))
            .insert_header(bearer)
            .to_request(),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::BAD_REQUEST);
    let missing: Value = test::read_body_json(missing).await;
    assert_eq!(violations("delete", "4XX", &missing), Vec::<String>::new());
}
