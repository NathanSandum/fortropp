use actix_web::{HttpRequest, HttpResponse, web};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

const NOT_FOUND: u32 = 1436;
const REFUSED: u32 = 1411;
const UNAUTHORIZED: u32 = 10000;
const UNAVAILABLE: u32 = 10001;
const AWAITING_DNS: &str = "custom hostname does not CNAME to this zone.";

#[derive(Clone)]
struct Challenge {
    path: String,
    body: String,
}

impl Challenge {
    fn ownership() -> Self {
        Self {
            path: format!(
                "/.well-known/cf-custom-hostname-challenge/{}",
                uuid::Uuid::new_v4()
            ),
            body: uuid::Uuid::new_v4().to_string(),
        }
    }

    fn certificate() -> Self {
        Self {
            path: format!("/.well-known/pki-validation/{}.txt", uuid::Uuid::new_v4()),
            body: uuid::Uuid::new_v4().to_string(),
        }
    }

    fn json(&self) -> Value {
        json!({ "path": self.path, "body": self.body })
    }

    fn http(&self, hostname: &str) -> Value {
        json!({
            "http_url": format!("http://{hostname}{}", self.path),
            "http_body": self.body,
        })
    }
}

#[derive(Clone)]
struct Record {
    id: String,
    hostname: String,
    status: String,
    ssl_status: String,
    verification_errors: Vec<String>,
    revalidations: u32,
    lookups: u32,
    ownership: Challenge,
    certificate: Challenge,
}

impl Record {
    fn awaiting_dns(hostname: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            hostname,
            status: "pending".to_string(),
            ssl_status: "pending_validation".to_string(),
            verification_errors: vec![AWAITING_DNS.to_string()],
            revalidations: 0,
            lookups: 0,
            ownership: Challenge::ownership(),
            certificate: Challenge::certificate(),
        }
    }

    fn challenges(&self) -> Value {
        json!({ "ownership": self.ownership.json(), "certificate": self.certificate.json() })
    }

    fn json(&self) -> Value {
        let mut ssl = json!({
            "id": self.id,
            "status": self.ssl_status,
            "method": "http",
            "type": "dv",
        });
        if let Some(fields) = ssl.as_object_mut()
            && self.ssl_status == "pending_validation"
        {
            fields.insert(
                "validation_records".to_string(),
                json!([self.certificate.http(&self.hostname)]),
            );
        }
        let mut record = json!({
            "id": self.id,
            "hostname": self.hostname,
            "status": self.status,
            "verification_errors": self.verification_errors,
            "ssl": ssl,
        });
        if let Some(fields) = record.as_object_mut()
            && self.status == "pending"
        {
            fields.insert(
                "ownership_verification_http".to_string(),
                self.ownership.http(&self.hostname),
            );
        }
        record
    }
}

#[derive(Clone, Deserialize)]
struct DnsRecord {
    #[serde(skip_deserializing)]
    id: String,
    name: String,
    #[serde(rename = "type")]
    kind: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

impl DnsRecord {
    fn json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "type": self.kind,
            "content": self.content,
            "ttl": self.ttl,
            "proxied": self.proxied,
            "proxiable": true,
            "settings": {},
            "meta": {},
            "comment": "",
            "tags": [],
            "created_on": "2026-01-01T00:00:00Z",
            "modified_on": "2026-01-01T00:00:00Z",
        })
    }
}

pub struct State {
    token: String,
    hostnames: Mutex<BTreeMap<String, Record>>,
    dns_records: Mutex<BTreeMap<String, DnsRecord>>,
    rejections: Mutex<BTreeMap<String, String>>,
    outage: AtomicBool,
}

impl State {
    #[must_use]
    pub fn new(token: String) -> web::Data<Self> {
        web::Data::new(Self {
            token,
            hostnames: Mutex::new(BTreeMap::new()),
            dns_records: Mutex::new(BTreeMap::new()),
            rejections: Mutex::new(BTreeMap::new()),
            outage: AtomicBool::new(false),
        })
    }
}

fn unavailable() -> HttpResponse {
    HttpResponse::ServiceUnavailable().json(json!({
        "success": false,
        "errors": [{ "code": UNAVAILABLE, "message": "Service temporarily unavailable" }],
        "messages": [],
        "result": null,
    }))
}

fn refused(request: &HttpRequest, state: &State) -> Option<HttpResponse> {
    if !authorized(request, state) {
        return Some(failure(UNAUTHORIZED, "Authentication error"));
    }
    state.outage.load(Ordering::SeqCst).then(unavailable)
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
    hostname: Option<String>,
    hostname_status: Option<String>,
}

fn page(found: &[Value]) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "success": true,
        "errors": [],
        "messages": [],
        "result": found,
        "result_info": {
            "page": 1,
            "per_page": 1000,
            "count": found.len(),
            "total_count": found.len(),
            "total_pages": usize::from(!found.is_empty()),
        },
    }))
}

async fn list(
    request: HttpRequest,
    query: web::Query<ListQuery>,
    state: web::Data<State>,
) -> HttpResponse {
    if let Some(refusal) = refused(&request, &state) {
        return refusal;
    }
    let Some(mut hostnames) = locked(&state) else {
        return poisoned();
    };
    let query = query.into_inner();
    let found: Vec<Value> = match query.hostname {
        Some(hostname) => hostnames
            .get_mut(&hostname)
            .map(|record| {
                record.lookups = record.lookups.saturating_add(1);
                record.json()
            })
            .into_iter()
            .collect(),
        None => hostnames
            .values()
            .filter(|record| {
                query
                    .hostname_status
                    .as_ref()
                    .is_none_or(|status| &record.status == status)
            })
            .map(Record::json)
            .collect(),
    };
    page(&found)
}

#[derive(Deserialize)]
struct DnsListQuery {
    #[serde(rename = "type")]
    kind: Option<String>,
    #[serde(rename = "name.endswith")]
    name_endswith: Option<String>,
    #[serde(rename = "name.exact")]
    name_exact: Option<String>,
}

fn dns_locked(state: &State) -> Option<std::sync::MutexGuard<'_, BTreeMap<String, DnsRecord>>> {
    state.dns_records.lock().ok()
}

async fn dns_list(
    request: HttpRequest,
    query: web::Query<DnsListQuery>,
    state: web::Data<State>,
) -> HttpResponse {
    if let Some(refusal) = refused(&request, &state) {
        return refusal;
    }
    let Some(records) = dns_locked(&state) else {
        return poisoned();
    };
    let found: Vec<Value> = records
        .values()
        .filter(|record| query.kind.as_ref().is_none_or(|kind| &record.kind == kind))
        .filter(|record| {
            query
                .name_endswith
                .as_ref()
                .is_none_or(|suffix| record.name.ends_with(suffix.as_str()))
        })
        .filter(|record| {
            query
                .name_exact
                .as_ref()
                .is_none_or(|name| &record.name == name)
        })
        .map(DnsRecord::json)
        .collect();
    page(&found)
}

async fn dns_create(
    request: HttpRequest,
    body: web::Json<DnsRecord>,
    state: web::Data<State>,
) -> HttpResponse {
    if let Some(refusal) = refused(&request, &state) {
        return refusal;
    }
    let Some(mut records) = dns_locked(&state) else {
        return poisoned();
    };
    let mut record = body.into_inner();
    record.id = uuid::Uuid::new_v4().simple().to_string();
    let result = record.json();
    records.insert(record.id.clone(), record);
    envelope(&result)
}

async fn dns_delete(
    request: HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<State>,
) -> HttpResponse {
    if let Some(refusal) = refused(&request, &state) {
        return refusal;
    }
    let Some(mut records) = dns_locked(&state) else {
        return poisoned();
    };
    let (_, id) = path.into_inner();
    if records.remove(&id).is_none() {
        return failure(81044, "Record does not exist.");
    }
    envelope(&json!({ "id": id }))
}

async fn dns_records(state: web::Data<State>) -> HttpResponse {
    let Some(records) = dns_locked(&state) else {
        return poisoned();
    };
    let all: Vec<Value> = records
        .values()
        .map(|record| {
            json!({
                "name": record.name,
                "type": record.kind,
                "content": record.content,
                "ttl": record.ttl,
                "proxied": record.proxied,
            })
        })
        .collect();
    HttpResponse::Ok().json(all)
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
    if let Some(refusal) = refused(&request, &state) {
        return refusal;
    }
    let hostname = body.into_inner().hostname;
    let rejection = state
        .rejections
        .lock()
        .ok()
        .and_then(|rejections| rejections.get(&hostname).cloned());
    if let Some(message) = rejection {
        return failure(REFUSED, &message);
    }
    let Some(mut hostnames) = locked(&state) else {
        return poisoned();
    };
    let created = Record::awaiting_dns(hostname);
    let result = created.json();
    hostnames.insert(created.hostname.clone(), created);
    envelope(&result)
}

async fn delete(
    request: HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<State>,
) -> HttpResponse {
    if let Some(refusal) = refused(&request, &state) {
        return refusal;
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

async fn edit(
    request: HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<State>,
) -> HttpResponse {
    if let Some(refusal) = refused(&request, &state) {
        return refusal;
    }
    let Some(mut hostnames) = locked(&state) else {
        return poisoned();
    };
    let (_, id) = path.into_inner();
    let Some(hostname) = hostnames.values_mut().find(|hostname| hostname.id == id) else {
        return failure(NOT_FOUND, "The custom hostname was not found.");
    };
    hostname.revalidations = hostname.revalidations.saturating_add(1);
    envelope(&hostname.json())
}

async fn revalidations(path: web::Path<String>, state: web::Data<State>) -> HttpResponse {
    let Some(hostnames) = locked(&state) else {
        return poisoned();
    };
    let count = hostnames
        .get(&path.into_inner())
        .map_or(0, |hostname| hostname.revalidations);
    HttpResponse::Ok().json(json!({ "count": count }))
}

async fn lookups(path: web::Path<String>, state: web::Data<State>) -> HttpResponse {
    let Some(hostnames) = locked(&state) else {
        return poisoned();
    };
    let count = hostnames
        .get(&path.into_inner())
        .map_or(0, |hostname| hostname.lookups);
    HttpResponse::Ok().json(json!({ "count": count }))
}

async fn challenge(path: web::Path<String>, state: web::Data<State>) -> HttpResponse {
    let Some(hostnames) = locked(&state) else {
        return poisoned();
    };
    hostnames.get(&path.into_inner()).map_or_else(
        || HttpResponse::NotFound().finish(),
        |hostname| HttpResponse::Ok().json(hostname.challenges()),
    )
}

async fn replace_challenge(path: web::Path<String>, state: web::Data<State>) -> HttpResponse {
    let Some(mut hostnames) = locked(&state) else {
        return poisoned();
    };
    let Some(hostname) = hostnames.get_mut(&path.into_inner()) else {
        return HttpResponse::NotFound().finish();
    };
    hostname.ownership = Challenge::ownership();
    hostname.certificate = Challenge::certificate();
    HttpResponse::Ok().json(hostname.challenges())
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

#[derive(Deserialize)]
struct RejectBody {
    message: String,
}

async fn reject(
    path: web::Path<String>,
    body: web::Json<RejectBody>,
    state: web::Data<State>,
) -> HttpResponse {
    let Ok(mut rejections) = state.rejections.lock() else {
        return poisoned();
    };
    rejections.insert(path.into_inner(), body.into_inner().message);
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
struct OutageBody {
    down: bool,
}

async fn outage(body: web::Json<OutageBody>, state: web::Data<State>) -> HttpResponse {
    state.outage.store(body.down, Ordering::SeqCst);
    HttpResponse::Ok().finish()
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route(
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
    .route(
        "/client/v4/zones/{zone}/custom_hostnames/{id}",
        web::patch().to(edit),
    )
    .route("/fake/hostnames/{hostname}", web::put().to(set))
    .route(
        "/fake/revalidations/{hostname}",
        web::get().to(revalidations),
    )
    .route(
        "/client/v4/zones/{zone}/dns_records",
        web::get().to(dns_list),
    )
    .route(
        "/client/v4/zones/{zone}/dns_records",
        web::post().to(dns_create),
    )
    .route(
        "/client/v4/zones/{zone}/dns_records/{id}",
        web::delete().to(dns_delete),
    )
    .route("/fake/dns_records", web::get().to(dns_records))
    .route("/fake/lookups/{hostname}", web::get().to(lookups))
    .route("/fake/challenges/{hostname}", web::get().to(challenge))
    .route(
        "/fake/challenges/{hostname}",
        web::put().to(replace_challenge),
    )
    .route("/fake/rejections/{hostname}", web::put().to(reject))
    .route("/fake/outage", web::put().to(outage))
    .route("/health", web::get().to(health));
}
