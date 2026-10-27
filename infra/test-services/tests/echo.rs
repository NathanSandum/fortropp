use actix_web::http::StatusCode;
use actix_web::http::header::HeaderMap;
use actix_web::{App, test};
use serde_json::{Value, json};
use test_services::echo::configure;

fn header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}

#[actix_web::test]
async fn echoes_the_method_path_headers_and_body() {
    let app = test::init_service(App::new().configure(configure)).await;
    let request = test::TestRequest::post()
        .uri("/v1/things?q=1")
        .insert_header(("x-custom", "kept"))
        .insert_header(("content-type", "text/plain"))
        .set_payload("hello")
        .to_request();
    let response = test::call_service(&app, request).await;
    let status = response.status();
    let headers = response.headers().clone();
    let mut echoed: Value = test::read_body_json(response).await;
    echoed["headers"]
        .as_array_mut()
        .unwrap()
        .sort_by_key(ToString::to_string);
    assert_eq!(
        (
            status,
            header(&headers, "content-type"),
            header(&headers, "set-cookie"),
            header(&headers, "x-upstream"),
            echoed,
        ),
        (
            StatusCode::OK,
            Some("application/json".to_string()),
            Some("upstream=1; Path=/".to_string()),
            Some("echo".to_string()),
            json!({
                "method": "POST",
                "path": "/v1/things?q=1",
                "headers": [
                    ["content-length", "5"],
                    ["content-type", "text/plain"],
                    ["x-custom", "kept"],
                ],
                "body": "hello",
            }),
        )
    );
}

#[actix_web::test]
async fn redirects_paths_ending_in_redirect() {
    let app = test::init_service(App::new().configure(configure)).await;
    let request = test::TestRequest::get().uri("/v1/redirect").to_request();
    let response = test::call_service(&app, request).await;
    assert_eq!(
        (response.status(), header(response.headers(), "location")),
        (
            StatusCode::FOUND,
            Some("https://elsewhere.example/landing".to_string()),
        )
    );
}
