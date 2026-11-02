use actix_web::{App, HttpServer};
use rcgen::{CertifiedKey, generate_simple_self_signed};
use rustls::ServerConfig;
use rustls::pki_types::PrivateKeyDer;
use std::io;
use test_services::echo::configure;

const HOSTNAME: &str = "echo.fortropp.test";
const CERTIFICATE_PATH: &str = "/certificate/echo.pem";

fn tls() -> io::Result<ServerConfig> {
    let CertifiedKey { cert, signing_key } =
        generate_simple_self_signed(vec![HOSTNAME.to_string()]).map_err(io::Error::other)?;
    std::fs::write(CERTIFICATE_PATH, cert.pem())?;
    let key = PrivateKeyDer::Pkcs8(signing_key.serialize_der().into());
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert.der().clone()], key)
        .map_err(io::Error::other)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| App::new().configure(configure))
        .bind_rustls_0_23(("0.0.0.0", 8080), tls()?)?
        .run()
        .await
}
