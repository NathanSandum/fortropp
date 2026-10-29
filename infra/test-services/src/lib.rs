#![allow(
    clippy::future_not_send,
    reason = "actix-web runs each handler on its worker's single-threaded executor, so handler futures never cross threads"
)]

pub mod cloudflare_saas;
pub mod echo;
pub mod openapi_contract;
pub mod postmark;
