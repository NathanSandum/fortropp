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

fn record_requirements<'schema>(schema: &'schema mut Value, operation: &str) -> &'schema mut Value {
    let result = &mut schema["allOf"][1]["properties"]["result"];
    if operation == "dns_list" {
        &mut result["items"]["required"]
    } else {
        &mut result["required"]
    }
}

fn dns_record_violations(operation: &str, body: &Value) -> Vec<String> {
    let mut schema = contract()["operations"][operation]["200"].clone();
    let requirements = record_requirements(&mut schema, operation);
    let without_data: Vec<Value> = requirements
        .as_array()
        .unwrap()
        .iter()
        .filter(|field| field.as_str() != Some("data"))
        .cloned()
        .collect();
    *requirements = Value::Array(without_data);
    against(&schema, body)
}

#[actix_web::test]
async fn cloudflares_dns_record_schema_still_requires_data_on_every_record() {
    for operation in ["dns_create", "dns_list"] {
        let mut schema = contract()["operations"][operation]["200"].clone();
        assert!(
            record_requirements(&mut schema, operation)
                .as_array()
                .unwrap()
                .contains(&json!("data")),
            "{operation}: Cloudflare's DNS record schema no longer requires data on every record; validate against it whole"
        );
    }
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
async fn every_dns_record_response_the_fake_gives_matches_cloudflares_schema() {
    let app = test::init_service(
        App::new()
            .app_data(State::new(TOKEN.to_string()))
            .configure(configure),
    )
    .await;
    let bearer = ("Authorization", format!("Bearer {TOKEN}"));
    let records = "/client/v4/zones/contract-zone/dns_records";

    let created = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(records)
            .insert_header(bearer.clone())
            .set_json(json!({
                "name": "abcd2345.edge.fortropp.test",
                "type": "A",
                "content": "192.0.2.1",
                "ttl": 60,
                "proxied": false,
            }))
            .to_request(),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let created: Value = test::read_body_json(created).await;
    assert_eq!(
        dns_record_violations("dns_create", &created),
        Vec::<String>::new()
    );

    let listed = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!(
                "{records}?type=A&name.endswith=.edge.fortropp.test&per_page=1000"
            ))
            .insert_header(bearer.clone())
            .to_request(),
    )
    .await;
    assert_eq!(listed.status(), StatusCode::OK);
    let listed: Value = test::read_body_json(listed).await;
    assert_eq!(
        dns_record_violations("dns_list", &listed),
        Vec::<String>::new()
    );
    assert_eq!(listed["result"].as_array().unwrap().len(), 1);

    let id = created["result"]["id"].as_str().unwrap().to_string();
    let deleted = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri(&format!("{records}/{id}"))
            .insert_header(bearer.clone())
            .to_request(),
    )
    .await;
    assert_eq!(deleted.status(), StatusCode::OK);
    let deleted: Value = test::read_body_json(deleted).await;
    assert_eq!(
        violations("dns_delete", "200", &deleted),
        Vec::<String>::new()
    );

    let missing = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri(&format!("{records}/{id}"))
            .insert_header(bearer)
            .to_request(),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::BAD_REQUEST);
    let missing: Value = test::read_body_json(missing).await;
    assert_eq!(
        failure_violations("dns_delete", &missing),
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

    let pending = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("{ZONE}?hostname_status=pending&per_page=1000"))
            .insert_header(bearer.clone())
            .to_request(),
    )
    .await;
    assert_eq!(pending.status(), StatusCode::OK);
    let pending: Value = test::read_body_json(pending).await;
    assert_eq!(violations("list", "200", &pending), Vec::<String>::new());
    assert_eq!(pending["result"].as_array().unwrap().len(), 1);

    let id = created["result"]["id"].as_str().unwrap().to_string();
    let edited = test::call_service(
        &app,
        test::TestRequest::patch()
            .uri(&format!("{ZONE}/{id}"))
            .insert_header(bearer.clone())
            .set_json(json!({ "ssl": { "method": "http", "type": "dv" } }))
            .to_request(),
    )
    .await;
    assert_eq!(edited.status(), StatusCode::OK);
    let edited: Value = test::read_body_json(edited).await;
    assert_eq!(violations("edit", "200", &edited), Vec::<String>::new());

    let revalidations = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/fake/revalidations/a.test")
            .to_request(),
    )
    .await;
    let revalidations: Value = test::read_body_json(revalidations).await;
    assert_eq!(revalidations, json!({ "count": 1 }));

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

    let edited_missing = test::call_service(
        &app,
        test::TestRequest::patch()
            .uri(&format!("{ZONE}/{id}"))
            .insert_header(("Authorization", format!("Bearer {TOKEN}")))
            .set_json(json!({ "ssl": { "method": "http", "type": "dv" } }))
            .to_request(),
    )
    .await;
    assert_eq!(edited_missing.status(), StatusCode::BAD_REQUEST);
    let edited_missing: Value = test::read_body_json(edited_missing).await;
    assert_eq!(
        failure_violations("edit", &edited_missing),
        Vec::<String>::new()
    );
}
