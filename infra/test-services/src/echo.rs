use actix_web::http::header;
use actix_web::{HttpRequest, HttpResponse, web};
use serde_json::json;

const REDIRECT_TARGET: &str = "https://elsewhere.example/landing";

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.default_service(web::to(answer));
}

async fn answer(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    let path = req.uri().to_string();
    if path.ends_with("/redirect") {
        return HttpResponse::Found()
            .insert_header((header::LOCATION, REDIRECT_TARGET))
            .finish();
    }
    let headers: Vec<[String; 2]> = req
        .headers()
        .iter()
        .map(|(name, value)| [name.as_str().to_string(), latin1(value.as_bytes())])
        .collect();
    HttpResponse::Ok()
        .insert_header((header::SET_COOKIE, "upstream=1; Path=/"))
        .insert_header(("x-upstream", "echo"))
        .json(json!({
            "method": req.method().as_str(),
            "path": path,
            "headers": headers,
            "body": latin1(&body),
        }))
}

fn latin1(bytes: &[u8]) -> String {
    bytes.iter().copied().map(char::from).collect()
}
