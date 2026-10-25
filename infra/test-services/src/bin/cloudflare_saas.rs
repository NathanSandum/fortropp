#![allow(
    clippy::future_not_send,
    reason = "actix-web runs each handler on its worker's single-threaded executor, so handler futures never cross threads"
)]

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Mutex;

const NOT_FOUND: u32 = 1436;
const UNAUTHORIZED: u32 = 10000;
const AWAITING_DNS: &str = "custom hostname does not CNAME to this zone.";

#[derive(Clone)]
struct Record {
    id: String,
    hostname: String,
    status: String,
    ssl_status: String,
    verification_errors: Vec<String>,
}

impl Record {
    fn awaiting_dns(hostname: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            hostname,
            status: "pending".to_string(),
            ssl_status: "pending_validation".to_string(),
            verification_errors: vec![AWAITING_DNS.to_string()],
        }
    }

    fn json(&self) -> Value {
        json!({
            "id": self.id,
            "hostname": self.hostname,
            "status": self.status,
            "verification_errors": self.verification_errors,
            "ssl": {
                "id": self.id,
                "status": self.ssl_status,
                "method": "http",
                "type": "dv",
            },
        })
    }
}

struct State {
    token: String,
    hostnames: Mutex<BTreeMap<String, Record>>,
}

fn envelope(result: &Value) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "success": true,
        "errors": [],
        "messages": [],
        "result": result,
    }))
}

fn failure(code: u32, message: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({
        "success": false,
        "errors": [{ "code": code, "message": message }],
        "messages": [],
        "result": null,
    }))
}

fn authorized(request: &HttpRequest, state: &State) -> bool {
    request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == format!("Bearer {}", state.token))
}

fn locked(state: &State) -> Option<std::sync::MutexGuard<'_, BTreeMap<String, Record>>> {
    state.hostnames.lock().ok()
}

fn poisoned() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(rename = "hostname.exact")]
    hostname: String,
}

async fn list(
    request: HttpRequest,
    query: web::Query<ListQuery>,
    state: web::Data<State>,
) -> HttpResponse {
    if !authorized(&request, &state) {
        return failure(UNAUTHORIZED, "Authentication error");
    }
    let Some(hostnames) = locked(&state) else {
        return poisoned();
    };
    let found: Vec<Value> = hostnames
        .get(&query.hostname)
        .map(Record::json)
        .into_iter()
        .collect();
    HttpResponse::Ok().json(json!({
        "success": true,
        "errors": [],
        "messages": [],
        "result": found,
        "result_info": {
            "page": 1,
            "per_page": 20,
            "count": found.len(),
            "total_count": found.len(),
            "total_pages": usize::from(!found.is_empty()),
        },
    }))
}

#[derive(Deserialize)]
struct CreateBody {
    hostname: String,
}

async fn create(
    request: HttpRequest,
    body: web::Json<CreateBody>,
    state: web::Data<State>,
) -> HttpResponse {
    if !authorized(&request, &state) {
        return failure(UNAUTHORIZED, "Authentication error");
    }
    let Some(mut hostnames) = locked(&state) else {
        return poisoned();
    };
    let created = Record::awaiting_dns(body.into_inner().hostname);
    let result = created.json();
    hostnames.insert(created.hostname.clone(), created);
    envelope(&result)
}

async fn delete(
    request: HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<State>,
) -> HttpResponse {
    if !authorized(&request, &state) {
        return failure(UNAUTHORIZED, "Authentication error");
    }
    let Some(mut hostnames) = locked(&state) else {
        return poisoned();
    };
    let (_, id) = path.into_inner();
    let Some(hostname) = hostnames
        .values()
        .find(|hostname| hostname.id == id)
        .map(|hostname| hostname.hostname.clone())
    else {
        return failure(NOT_FOUND, "The custom hostname was not found.");
    };
    hostnames.remove(&hostname);
    envelope(&json!({ "id": id, "ssl": null }))
}

#[derive(Deserialize)]
struct SetBody {
    status: String,
    ssl_status: String,
    verification_errors: Vec<String>,
}

async fn set(
    path: web::Path<String>,
    body: web::Json<SetBody>,
    state: web::Data<State>,
) -> HttpResponse {
    let Some(mut hostnames) = locked(&state) else {
        return poisoned();
    };
    let Some(hostname) = hostnames.get_mut(&path.into_inner()) else {
        return HttpResponse::NotFound().finish();
    };
    let body = body.into_inner();
    hostname.status = body.status;
    hostname.ssl_status = body.ssl_status;
    hostname.verification_errors = body.verification_errors;
    HttpResponse::Ok().json(hostname.json())
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let token = std::env::var("FAKE_CLOUDFLARE_TOKEN").map_err(std::io::Error::other)?;
    let state = web::Data::new(State {
        token,
        hostnames: Mutex::new(BTreeMap::new()),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route(
                "/client/v4/zones/{zone}/custom_hostnames",
                web::get().to(list),
            )
            .route(
                "/client/v4/zones/{zone}/custom_hostnames",
                web::post().to(create),
            )
            .route(
                "/client/v4/zones/{zone}/custom_hostnames/{id}",
                web::delete().to(delete),
            )
            .route("/fake/hostnames/{hostname}", web::put().to(set))
            .route("/health", web::get().to(health))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
