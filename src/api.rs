use crate::data;
use crate::types::{Chain, Endpoint, Info, Provider};
use axum::Json;
use axum::extract::Path;

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

pub async fn get_chain_endpoints(Path(id): Path<u32>) -> Json<Vec<&'static Endpoint>> {
    Json(
        data::ENDPOINTS
            .iter()
            .filter(|e| e.chain_id == id)
            .collect(),
    )
}

pub async fn get_provider_endpoints(Path(id): Path<u32>) -> Json<Vec<&'static Endpoint>> {
    Json(
        data::ENDPOINTS
            .iter()
            .filter(|e| e.provider_id == id)
            .collect(),
    )
}

pub async fn get_chain_providers(Path(id): Path<u32>) -> Json<Vec<&'static Provider>> {
    let provider_ids: Vec<u32> = data::ENDPOINTS
        .iter()
        .filter(|e| e.chain_id == id)
        .map(|e| e.provider_id)
        .collect();
    Json(
        data::PROVIDERS
            .iter()
            .filter(|p| provider_ids.contains(&p.id))
            .collect(),
    )
}
