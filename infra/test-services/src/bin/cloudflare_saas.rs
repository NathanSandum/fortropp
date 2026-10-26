use actix_web::{App, HttpServer};
use test_services::cloudflare_saas::{State, configure};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let token = std::env::var("FAKE_CLOUDFLARE_TOKEN").map_err(std::io::Error::other)?;
    let state = State::new(token);
    HttpServer::new(move || App::new().app_data(state.clone()).configure(configure))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
