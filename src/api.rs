use crate::data;
use crate::types::{Chain, Endpoint, Info, Provider};
use axum::Json;

pub async fn get_info() -> Json<Info> {
    Json(Info {
        version: env!("CARGO_PKG_VERSION"),
        location: std::env::var("LOCATION").unwrap_or_default(),
    })
}

pub async fn get_chains() -> Json<&'static [Chain]> {
    Json(data::CHAINS)
}

pub async fn get_providers() -> Json<&'static [Provider]> {
    Json(data::PROVIDERS)
}

pub async fn get_endpoints() -> Json<&'static [Endpoint]> {
    Json(data::ENDPOINTS)
}
