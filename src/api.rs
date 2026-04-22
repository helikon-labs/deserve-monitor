use crate::data;
use crate::types::{Endpoint, Info};
use axum::Json;

pub async fn get_info() -> Json<Info> {
    Json(Info {
        version: env!("CARGO_PKG_VERSION"),
        location: std::env::var("LOCATION").unwrap_or_default(),
    })
}

pub async fn get_endpoints() -> Json<&'static [Endpoint]> {
    Json(data::ENDPOINTS)
}
