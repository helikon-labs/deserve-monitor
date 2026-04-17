#![warn(clippy::disallowed_types)]

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use reqwest::header::CONTENT_TYPE;
use rustc_hash::FxHashMap as HashMap;
use serde::Serialize;
use std::collections::VecDeque;
use std::error::Error as StdError;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct Endpoint {
    id: u32,
    url: &'static str,
}

const ENDPOINTS: &[Endpoint] = &[
    Endpoint {
        id: 0,
        url: "https://asset-hub.polkadot.rpc.deserve.network",
    },
    Endpoint {
        id: 1,
        url: "https://coretime.polkadot.rpc.deserve.network",
    },
    Endpoint {
        id: 2,
        url: "https://asset-hub-polkadot.ibp.network",
    },
    Endpoint {
        id: 3,
        url: "https://coretime-polkadot.ibp.network",
    },
    Endpoint {
        id: 4,
        url: "https://asset-hub-polkadot.dotters.network",
    },
    Endpoint {
        id: 5,
        url: "https://coretime-polkadot.dotters.network",
    },
];
const MAX_LATENCY_RECORDS: usize = 360;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_secs(10);
const RPC_BODY: &str =
    r#"{"id":"1","jsonrpc":"2.0","method":"chain_getFinalizedHead","params":[]}"#;
const API_PORT: u16 = 1881;

#[derive(Clone, Serialize)]
struct Measurement {
    started_at: u128,
    ended_at: u128,
    is_successful: bool,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_opt_duration_as_millis"
    )]
    latency: Option<Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ip: Option<IpAddr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn describe_reqwest_error(e: &reqwest::Error) -> String {
    let kind = if e.is_timeout() {
        "timeout"
    } else if e.is_connect() {
        "connect"
    } else if e.is_request() {
        "request"
    } else if e.is_body() {
        "body"
    } else if e.is_decode() {
        "decode"
    } else {
        "unknown"
    };

    let mut source: Option<&dyn StdError> = e.source();
    let mut root_cause = None;
    while let Some(s) = source {
        root_cause = Some(s.to_string());
        source = s.source();
    }

    match root_cause {
        Some(cause) => format!("{}: {}", kind, cause),
        None => kind.to_string(),
    }
}

fn serialize_opt_duration_as_millis<S: serde::Serializer>(
    d: &Option<Duration>,
    s: S,
) -> Result<S::Ok, S::Error> {
    s.serialize_u128(d.unwrap().as_millis())
}

type Measurements = Arc<Mutex<HashMap<u32, VecDeque<Measurement>>>>;

fn push_measurement(
    measurements: &mut HashMap<u32, VecDeque<Measurement>>,
    endpoint_id: u32,
    record: Measurement,
) {
    let records = measurements.entry(endpoint_id).or_default();
    if records.len() == MAX_LATENCY_RECORDS {
        records.pop_front();
    }
    records.push_back(record);
}

#[derive(Serialize)]
struct Info {
    version: &'static str,
    location: String,
}

async fn get_info() -> Json<Info> {
    Json(Info {
        version: env!("CARGO_PKG_VERSION"),
        location: std::env::var("LOCATION").unwrap_or_default(),
    })
}

async fn get_endpoints() -> Json<&'static [Endpoint]> {
    Json(ENDPOINTS)
}

#[derive(Serialize)]
struct EndpointStats {
    average_latency: u128,
    median_latency: u128,
    p95_latency: u128,
    success_percent: f64,
    measurements: VecDeque<Measurement>,
}

async fn get_measurements(
    State(measurements): State<Measurements>,
) -> Json<HashMap<u32, EndpointStats>> {
    let measurements = measurements.lock().unwrap();
    let stats = measurements
        .iter()
        .map(|(id, records)| {
            let mut latencies: Vec<u128> = records
                .iter()
                .filter_map(|m| m.latency.map(|d| d.as_millis()))
                .collect();
            latencies.sort_unstable();

            let average_latency = if latencies.is_empty() {
                0
            } else {
                latencies.iter().sum::<u128>() / latencies.len() as u128
            };

            let median_latency = if latencies.is_empty() {
                0
            } else {
                let idx = ((latencies.len() as f64 * 0.50).ceil() as usize).saturating_sub(1);
                latencies[idx]
            };

            let p95_latency = if latencies.is_empty() {
                0
            } else {
                let idx = ((latencies.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
                latencies[idx]
            };

            let success_percent = if records.is_empty() {
                0.0
            } else {
                let successful = records.iter().filter(|m| m.is_successful).count();
                (successful as f64 / records.len() as f64 * 100.0 * 10.0).round() / 10.0
            };

            (
                *id,
                EndpointStats {
                    average_latency,
                    median_latency,
                    p95_latency,
                    success_percent,
                    measurements: records.clone(),
                },
            )
        })
        .collect();

    Json(stats)
}

#[tokio::main]
async fn main() {
    let measurements: Measurements = Arc::new(Mutex::new(HashMap::default()));
    let client = reqwest::Client::builder()
        .connect_timeout(CONNECTION_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .unwrap();

    let router = Router::new()
        .route("/", get(get_info))
        .route("/endpoints", get(get_endpoints))
        .route("/measurements", get(get_measurements))
        .with_state(Arc::clone(&measurements));

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{API_PORT}"))
            .await
            .unwrap();
        axum::serve(listener, router).await.unwrap();
    });

    let mut first_run = true;

    loop {
        let mut round: Vec<(u32, Measurement)> = Vec::new();

        for endpoint in ENDPOINTS {
            let start = Instant::now();
            let started_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let result = client
                .post(endpoint.url)
                .header(CONTENT_TYPE, "application/json")
                .body(RPC_BODY)
                .send()
                .await;
            let latency = start.elapsed();
            let ended_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            let measurement = match result {
                Ok(response) => {
                    let ip = response.remote_addr().map(|a| a.ip());
                    let status = response.status();
                    if status.is_success() {
                        println!(
                            "[{}] {} ({}) — {}ms",
                            endpoint.id,
                            endpoint.url,
                            ip.map_or("-".to_string(), |ip| ip.to_string()),
                            latency.as_millis()
                        );
                        Measurement {
                            started_at,
                            ended_at,
                            is_successful: true,
                            latency: Some(latency),
                            ip,
                            error: None,
                        }
                    } else {
                        let body = response.text().await.unwrap_or_default();
                        let error = format!("HTTP {} — {}", status, body.trim());
                        eprintln!("[{}] {} — {}", endpoint.id, endpoint.url, error);
                        Measurement {
                            started_at,
                            ended_at,
                            is_successful: false,
                            latency: Some(latency),
                            ip,
                            error: Some(error),
                        }
                    }
                }
                Err(e) => {
                    let error = describe_reqwest_error(&e);
                    eprintln!("[{}] {} — {}", endpoint.id, endpoint.url, error);
                    Measurement {
                        started_at,
                        ended_at,
                        is_successful: false,
                        latency: None,
                        ip: None,
                        error: Some(error),
                    }
                }
            };

            round.push((endpoint.id, measurement));
        }

        if !first_run {
            let mut measurements = measurements.lock().unwrap();
            for (endpoint_id, measurement) in round {
                push_measurement(&mut measurements, endpoint_id, measurement);
            }
        }

        first_run = false;
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}
