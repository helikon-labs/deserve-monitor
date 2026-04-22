use rustc_hash::FxHashMap as HashMap;
use serde::Serialize;
use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Serialize)]
pub struct Endpoint {
    pub id: u32,
    pub url: &'static str,
}

fn serialize_opt_duration_as_millis<S: serde::Serializer>(
    d: &Option<Duration>,
    s: S,
) -> Result<S::Ok, S::Error> {
    s.serialize_u128(d.unwrap().as_millis())
}

#[derive(Clone, Serialize)]
pub struct Measurement {
    pub started_at: u128,
    pub ended_at: u128,
    pub is_successful: bool,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_opt_duration_as_millis"
    )]
    pub latency: Option<Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<IpAddr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub type Measurements = Arc<Mutex<HashMap<u32, VecDeque<Measurement>>>>;

#[derive(Serialize)]
pub struct Info {
    pub version: &'static str,
    pub location: String,
}

#[derive(Serialize)]
pub struct EndpointStats {
    pub average_latency: u128,
    pub median_latency: u128,
    pub p95_latency: u128,
    pub success_percent: f64,
    pub measurements: VecDeque<Measurement>,
}
