#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "test helpers outside #[test] functions, where clippy's test allowances don't reach"
)]
#![allow(
    clippy::future_not_send,
    reason = "actix-web's test services run on a single-threaded executor, so their futures never cross threads"
)]

use actix_web::dev::{Service, ServiceResponse};
use actix_web::http::StatusCode;
use actix_web::{App, HttpResponse, test, web};
use serde_json::{Value, json};
use std::sync::Mutex;
use test_services::postmark::{State, configure};

const TOKEN: &str = "postmark-test-token";

type Recorded = web::Data<Mutex<Vec<Value>>>;

async fn record(body: web::Json<Value>, recorded: Recorded) -> HttpResponse {
    recorded.lock().unwrap().push(body.into_inner());
    HttpResponse::Ok().json(json!({ "ID": "recorded" }))
}

fn mailpit() -> (actix_test::TestServer, Recorded) {
    let recorded: Recorded = web::Data::new(Mutex::new(Vec::new()));
    let shared = recorded.clone();
    let server = actix_test::start(move || {
        App::new()
            .app_data(shared.clone())
            .route("/api/v1/send", web::post().to(record))
    });
    (server, recorded)
}

async fn app(
    mailpit: String,
) -> impl Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error> {
    let state = State::new(TOKEN.to_string(), mailpit.trim_end_matches('/').to_string());
    test::init_service(App::new().app_data(state).configure(configure)).await
}

fn send(body: &Value) -> actix_http::Request {
    test::TestRequest::post()
        .uri("/email")
        .insert_header(("X-Postmark-Server-Token", TOKEN))
        .set_json(body)
        .to_request()
}

async fn outcome(response: ServiceResponse) -> (StatusCode, Option<String>, Value) {
    let status = response.status();
    let code = response
        .headers()
        .get("X-PM-ApiErrorCode")
        .map(|value| value.to_str().unwrap().to_string());
    let bytes = test::read_body(response).await;
    let mut body: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    if let Some(submitted) = body.get_mut("SubmittedAt") {
        assert!(
            submitted
                .as_str()
                .unwrap()
                .parse::<jiff::Timestamp>()
                .is_ok()
        );
        *submitted = json!("<timestamp>");
    }
    if let Some(id) = body.get_mut("MessageID") {
        assert!(uuid::Uuid::parse_str(id.as_str().unwrap()).is_ok());
        *id = json!("<uuid>");
    }
    for suppression in body
        .get_mut("Suppressions")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
    {
        assert!(
            suppression["CreatedAt"]
                .as_str()
                .unwrap()
                .parse::<jiff::Timestamp>()
                .is_ok()
        );
        suppression["CreatedAt"] = json!("<timestamp>");
    }
    (status, code, body)
}

fn accepted(to: &str) -> (StatusCode, Option<String>, Value) {
    (
        StatusCode::OK,
        None,
        json!({
            "To": to,
            "SubmittedAt": "<timestamp>",
            "MessageID": "<uuid>",
            "ErrorCode": 0,
            "Message": "OK",
        }),
    )
}

fn message(to: &str, stream: Option<&str>) -> Value {
    let mut message = json!({
        "From": "fortropp <noreply@fortropp.dev>",
        "To": to,
        "ReplyTo": "support@fortropp.dev",
        "Subject": "Your code",
        "TextBody": "Your code is 123456.",
        "HtmlBody": "<p>Your code is 123456.</p>",
    });
    if let Some(stream) = stream {
        message["MessageStream"] = json!(stream);
    }
    message
}

fn delivered(to: &str, stream: &str) -> Value {
    json!({
        "From": { "Email": "noreply@fortropp.dev", "Name": "fortropp" },
        "To": [{ "Email": to, "Name": "" }],
        "ReplyTo": [{ "Email": "support@fortropp.dev", "Name": "" }],
        "Subject": "Your code",
        "Text": "Your code is 123456.",
        "HTML": "<p>Your code is 123456.</p>",
        "Tags": [stream],
    })
}

async fn suppress(
    app: &impl Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
    stream: &str,
    email: &str,
    reason: &str,
) {
    let request = test::TestRequest::put()
        .uri(&format!("/fake/suppressions/{stream}/{email}"))
        .set_json(json!({ "reason": reason, "origin": "Recipient" }))
        .to_request();
    assert_eq!(
        test::call_service(app, request).await.status(),
        StatusCode::OK
    );
}

async fn fail(
    app: &impl Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
    failure: &str,
) {
    let request = test::TestRequest::put()
        .uri("/fake/failure")
        .set_json(json!({ "failure": failure }))
        .to_request();
    assert_eq!(
        test::call_service(app, request).await.status(),
        StatusCode::OK
    );
}

#[actix_web::test]
async fn a_message_is_delivered_to_mailpit_tagged_with_its_stream() {
    let (server, recorded) = mailpit();
    let app = app(server.url("")).await;
    let default_stream =
        outcome(test::call_service(&app, send(&message("user@example.test", None))).await).await;
    let broadcast = outcome(
        test::call_service(&app, send(&message("user@example.test", Some("broadcast")))).await,
    )
    .await;
    assert_eq!(
        (default_stream, broadcast, recorded.lock().unwrap().clone()),
        (
            accepted("user@example.test"),
            accepted("user@example.test"),
            vec![
                delivered("user@example.test", "outbound"),
                delivered("user@example.test", "broadcast"),
            ],
        )
    );
}

#[actix_web::test]
async fn the_blackhole_address_is_accepted_and_never_delivered() {
    let (server, recorded) = mailpit();
    let app = app(server.url("")).await;
    let response = outcome(
        test::call_service(&app, send(&message("test@blackhole.postmarkapp.com", None))).await,
    )
    .await;
    assert_eq!(
        (response, recorded.lock().unwrap().clone()),
        (accepted("test@blackhole.postmarkapp.com"), vec![])
    );
}

#[actix_web::test]
async fn a_missing_or_wrong_token_is_refused() {
    let (server, recorded) = mailpit();
    let app = app(server.url("")).await;
    let refusal = (
        StatusCode::UNAUTHORIZED,
        Some("10".to_string()),
        json!({
            "ErrorCode": 10,
            "Message": "Request does not contain a valid Server or Account token.",
        }),
    );
    let missing = test::TestRequest::post()
        .uri("/email")
        .set_json(message("user@example.test", None))
        .to_request();
    let wrong = test::TestRequest::post()
        .uri("/email")
        .insert_header(("X-Postmark-Server-Token", "wrong"))
        .set_json(message("user@example.test", None))
        .to_request();
    let missing = outcome(test::call_service(&app, missing).await).await;
    let wrong = outcome(test::call_service(&app, wrong).await).await;
    assert_eq!(
        (missing, wrong, recorded.lock().unwrap().clone()),
        (refusal.clone(), refusal, vec![])
    );
}

#[actix_web::test]
async fn a_suppressed_recipient_is_inactive_on_its_stream_only() {
    let (server, recorded) = mailpit();
    let app = app(server.url("")).await;
    suppress(&app, "outbound", "gone@example.test", "HardBounce").await;
    let outbound =
        outcome(test::call_service(&app, send(&message("gone@example.test", None))).await).await;
    let broadcast = outcome(
        test::call_service(&app, send(&message("gone@example.test", Some("broadcast")))).await,
    )
    .await;
    assert_eq!(
        (outbound, broadcast, recorded.lock().unwrap().clone()),
        (
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Some("406".to_string()),
                json!({
                    "ErrorCode": 406,
                    "Message": "You tried to send to recipient(s) that have been marked as inactive. Found inactive addresses: gone@example.test. Inactive recipients are ones that have generated a hard bounce, a spam complaint, or a manual suppression.",
                }),
            ),
            accepted("gone@example.test"),
            vec![delivered("gone@example.test", "broadcast")],
        )
    );
}

#[actix_web::test]
async fn the_suppression_dump_reports_the_reason_for_an_address() {
    let (server, _recorded) = mailpit();
    let app = app(server.url("")).await;
    suppress(&app, "outbound", "gone@example.test", "SpamComplaint").await;
    let dump = |email: &str| {
        test::TestRequest::get()
            .uri(&format!(
                "/message-streams/outbound/suppressions/dump?EmailAddress={email}"
            ))
            .insert_header(("X-Postmark-Server-Token", TOKEN))
            .to_request()
    };
    let suppressed = outcome(test::call_service(&app, dump("gone@example.test")).await).await;
    let active = outcome(test::call_service(&app, dump("here@example.test")).await).await;
    assert_eq!(
        (suppressed, active),
        (
            (
                StatusCode::OK,
                None,
                json!({ "Suppressions": [{
                    "EmailAddress": "gone@example.test",
                    "SuppressionReason": "SpamComplaint",
                    "Origin": "Recipient",
                    "CreatedAt": "<timestamp>",
                }] }),
            ),
            (StatusCode::OK, None, json!({ "Suppressions": [] })),
        )
    );
}

#[actix_web::test]
async fn an_outage_and_a_rate_limit_refuse_every_send_until_cleared() {
    let (server, recorded) = mailpit();
    let app = app(server.url("")).await;
    fail(&app, "outage").await;
    let outage = outcome(
        test::call_service(&app, send(&message("test@blackhole.postmarkapp.com", None))).await,
    )
    .await;
    fail(&app, "rate_limited").await;
    let rate_limited = outcome(
        test::call_service(&app, send(&message("test@blackhole.postmarkapp.com", None))).await,
    )
    .await;
    fail(&app, "none").await;
    let cleared = outcome(
        test::call_service(&app, send(&message("test@blackhole.postmarkapp.com", None))).await,
    )
    .await;
    assert_eq!(
        (
            outage,
            rate_limited,
            cleared,
            recorded.lock().unwrap().clone()
        ),
        (
            (StatusCode::SERVICE_UNAVAILABLE, None, Value::Null),
            (
                StatusCode::TOO_MANY_REQUESTS,
                Some("429".to_string()),
                json!({
                    "ErrorCode": 429,
                    "Message": "You are making requests at a rate that exceeds acceptable use of the API.",
                }),
            ),
            accepted("test@blackhole.postmarkapp.com"),
            vec![],
        )
    );
}
