use actix_web::{App, HttpServer};
use test_services::postmark::{State, configure};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let token = std::env::var("FAKE_POSTMARK_TOKEN").map_err(std::io::Error::other)?;
    let mailpit = std::env::var("FAKE_POSTMARK_MAILPIT_URL").map_err(std::io::Error::other)?;
    let state = State::new(token, mailpit);
    HttpServer::new(move || App::new().app_data(state.clone()).configure(configure))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
