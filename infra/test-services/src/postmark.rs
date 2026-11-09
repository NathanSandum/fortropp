use actix_web::{HttpRequest, HttpResponse, web};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Mutex;

const BLACKHOLE: &str = "test@blackhole.postmarkapp.com";
const DEFAULT_STREAM: &str = "outbound";
const INVALID_TOKEN: u32 = 10;
const INACTIVE_RECIPIENT: u32 = 406;
const PENDING_APPROVAL: u32 = 412;
const RATE_LIMITED: u32 = 429;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Failure {
    None,
    Outage,
    RateLimited,
    PendingApproval,
}

#[derive(Clone, Deserialize, Serialize)]
enum Reason {
    HardBounce,
    SpamComplaint,
    ManualSuppression,
}

#[derive(Clone, Deserialize, Serialize)]
enum Origin {
    Recipient,
    Customer,
    Admin,
}

#[derive(Clone)]
struct Suppression {
    reason: Reason,
    origin: Origin,
    created_at: String,
}

pub struct State {
    token: String,
    mailpit: String,
    failure: Mutex<Failure>,
    suppressions: Mutex<BTreeMap<(String, String), Suppression>>,
}

impl State {
    #[must_use]
    pub fn new(token: String, mailpit: String) -> web::Data<Self> {
        web::Data::new(Self {
            token,
            mailpit,
            failure: Mutex::new(Failure::None),
            suppressions: Mutex::new(BTreeMap::new()),
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SendBody {
    from: String,
    to: String,
    subject: Option<String>,
    html_body: Option<String>,
    text_body: Option<String>,
    reply_to: Option<String>,
    message_stream: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Mailbox {
    email: String,
    name: String,
}

impl Mailbox {
    fn parse(address: &str) -> Self {
        let address = address.trim();
        match address.rsplit_once('<') {
            Some((name, rest)) => Self {
                email: rest.trim_end_matches('>').trim().to_string(),
                name: name.trim().trim_matches('"').to_string(),
            },
            None => Self {
                email: address.to_string(),
                name: String::new(),
            },
        }
    }

    fn list(addresses: &str) -> Vec<Self> {
        addresses.split(',').map(Self::parse).collect()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct MailpitMessage {
    from: Mailbox,
    to: Vec<Mailbox>,
    reply_to: Vec<Mailbox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(rename = "HTML", skip_serializing_if = "Option::is_none")]
    html: Option<String>,
    tags: Vec<String>,
}

impl MailpitMessage {
    fn new(body: SendBody, stream: String) -> Self {
        Self {
            from: Mailbox::parse(&body.from),
            to: Mailbox::list(&body.to),
            reply_to: body
                .reply_to
                .as_deref()
                .into_iter()
                .flat_map(Mailbox::list)
                .collect(),
            subject: body.subject,
            text: body.text_body,
            html: body.html_body,
            tags: vec![stream],
        }
    }
}

fn now() -> String {
    jiff::Timestamp::now().to_string()
}

fn postmark_error(code: u32, message: &str) -> HttpResponse {
    HttpResponse::UnprocessableEntity()
        .insert_header(("X-PM-ApiErrorCode", code.to_string()))
        .json(json!({ "ErrorCode": code, "Message": message }))
}

fn poisoned() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}

fn refused(request: &HttpRequest, state: &State) -> Option<HttpResponse> {
    let authorized = request
        .headers()
        .get("x-postmark-server-token")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == state.token);
    if !authorized {
        return Some(
            HttpResponse::Unauthorized()
                .insert_header(("X-PM-ApiErrorCode", INVALID_TOKEN.to_string()))
                .json(json!({
                    "ErrorCode": INVALID_TOKEN,
                    "Message": "Request does not contain a valid Server token.",
                })),
        );
    }
    let Ok(failure) = state.failure.lock() else {
        return Some(poisoned());
    };
    match *failure {
        Failure::None | Failure::PendingApproval => None,
        Failure::Outage => Some(HttpResponse::ServiceUnavailable().finish()),
        Failure::RateLimited => Some(
            HttpResponse::TooManyRequests()
                .insert_header(("X-PM-ApiErrorCode", RATE_LIMITED.to_string()))
                .json(json!({
                    "ErrorCode": RATE_LIMITED,
                    "Message": "You are making requests at a rate that exceeds acceptable use of the API.",
                })),
        ),
    }
}

enum Decision {
    Deliver(MailpitMessage),
    Drop,
    Inactive(Vec<String>),
    PendingApproval,
    Poisoned,
}

fn domain(email: &str) -> &str {
    email.rsplit_once('@').map_or("", |(_, domain)| domain)
}

fn decide(body: SendBody, state: &State) -> Decision {
    let stream = body
        .message_stream
        .clone()
        .unwrap_or_else(|| DEFAULT_STREAM.to_string());
    let recipients = Mailbox::list(&body.to);
    let Ok(failure) = state.failure.lock() else {
        return Decision::Poisoned;
    };
    let pending = matches!(*failure, Failure::PendingApproval);
    drop(failure);
    let sender = Mailbox::parse(&body.from);
    if pending
        && recipients.iter().any(|recipient| {
            recipient.email != BLACKHOLE && domain(&recipient.email) != domain(&sender.email)
        })
    {
        return Decision::PendingApproval;
    }
    let Ok(suppressions) = state.suppressions.lock() else {
        return Decision::Poisoned;
    };
    let inactive: Vec<String> = recipients
        .iter()
        .map(|recipient| recipient.email.clone())
        .filter(|email| suppressions.contains_key(&(stream.clone(), email.clone())))
        .collect();
    drop(suppressions);
    if !inactive.is_empty() {
        return Decision::Inactive(inactive);
    }
    if recipients
        .iter()
        .all(|recipient| recipient.email == BLACKHOLE)
    {
        return Decision::Drop;
    }
    Decision::Deliver(MailpitMessage::new(body, stream))
}

fn inactive_recipients(addresses: &[String]) -> HttpResponse {
    postmark_error(
        INACTIVE_RECIPIENT,
        &format!(
            "You tried to send to recipient(s) that have been marked as inactive. Found inactive addresses: {}. Inactive recipients are ones that have generated a hard bounce, a spam complaint, or a manual suppression.",
            addresses.join(", ")
        ),
    )
}

async fn deliver(mailpit: &str, message: &MailpitMessage) -> Result<(), String> {
    let mut response = awc::Client::default()
        .post(format!("{mailpit}/api/v1/send"))
        .send_json(message)
        .await
        .map_err(|error| format!("Mailpit unreachable: {error}"))?;
    if response.status().is_success() {
        return Ok(());
    }
    let status = response.status();
    Err(match response.body().await {
        Ok(body) => format!(
            "Mailpit answered {status}: {}",
            String::from_utf8_lossy(&body)
        ),
        Err(error) => format!("Mailpit answered {status}, body unreadable: {error}"),
    })
}

async fn send(
    request: HttpRequest,
    body: web::Json<SendBody>,
    state: web::Data<State>,
) -> HttpResponse {
    if let Some(refusal) = refused(&request, &state) {
        return refusal;
    }
    let body = body.into_inner();
    let to = body.to.clone();
    match decide(body, &state) {
        Decision::Deliver(message) => {
            if let Err(reason) = deliver(&state.mailpit, &message).await {
                eprintln!("delivery to Mailpit failed: {reason}");
                return HttpResponse::InternalServerError().body(reason);
            }
        }
        Decision::Drop => {}
        Decision::Inactive(addresses) => return inactive_recipients(&addresses),
        Decision::PendingApproval => {
            return postmark_error(
                PENDING_APPROVAL,
                "While your account is pending approval, all recipient addresses must share the same domain as the 'From' address.",
            );
        }
        Decision::Poisoned => return poisoned(),
    }
    HttpResponse::Ok().json(json!({
        "To": to,
        "SubmittedAt": now(),
        "MessageID": uuid::Uuid::new_v4().to_string(),
        "ErrorCode": 0,
        "Message": "OK",
    }))
}

#[derive(Deserialize)]
struct DumpQuery {
    #[serde(rename = "EmailAddress")]
    email_address: Option<String>,
}

async fn dump(
    request: HttpRequest,
    stream: web::Path<String>,
    query: web::Query<DumpQuery>,
    state: web::Data<State>,
) -> HttpResponse {
    if let Some(refusal) = refused(&request, &state) {
        return refusal;
    }
    let Ok(suppressions) = state.suppressions.lock() else {
        return poisoned();
    };
    let stream = stream.into_inner();
    let found: Vec<_> = suppressions
        .iter()
        .filter(|((suppressed_stream, email), _)| {
            *suppressed_stream == stream
                && query
                    .email_address
                    .as_ref()
                    .is_none_or(|wanted| wanted == email)
        })
        .map(|((_, email), suppression)| {
            json!({
                "EmailAddress": email,
                "SuppressionReason": suppression.reason,
                "Origin": suppression.origin,
                "CreatedAt": suppression.created_at,
            })
        })
        .collect();
    HttpResponse::Ok().json(json!({ "Suppressions": found }))
}

#[derive(Deserialize)]
struct SuppressBody {
    reason: Reason,
    origin: Origin,
}

async fn suppress(
    path: web::Path<(String, String)>,
    body: web::Json<SuppressBody>,
    state: web::Data<State>,
) -> HttpResponse {
    let Ok(mut suppressions) = state.suppressions.lock() else {
        return poisoned();
    };
    let body = body.into_inner();
    suppressions.insert(
        path.into_inner(),
        Suppression {
            reason: body.reason,
            origin: body.origin,
            created_at: now(),
        },
    );
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
struct FailureBody {
    failure: Failure,
}

async fn fail(body: web::Json<FailureBody>, state: web::Data<State>) -> HttpResponse {
    let Ok(mut failure) = state.failure.lock() else {
        return poisoned();
    };
    *failure = body.into_inner().failure;
    HttpResponse::Ok().finish()
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/email", web::post().to(send))
        .route(
            "/message-streams/{stream}/suppressions/dump",
            web::get().to(dump),
        )
        .route(
            "/fake/suppressions/{stream}/{email}",
            web::put().to(suppress),
        )
        .route("/fake/failure", web::put().to(fail))
        .route("/health", web::get().to(health));
}
