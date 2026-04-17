#![warn(clippy::disallowed_types)]

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use reqwest::header::CONTENT_TYPE;
use rustc_hash::FxHashMap as HashMap;
use serde::Serialize;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
        url: "https://asset-hub-polkadot.ibp.network",
    },
];
const MAX_LATENCY_RECORDS: usize = 50;
const RPC_BODY: &str =
    r#"{"id":"1","jsonrpc":"2.0","method":"chain_getFinalizedHead","params":[]}"#;

#[derive(Clone, Serialize)]
struct Measurement {
    started_at: u128,
    ended_at: u128,
    #[serde(serialize_with = "serialize_duration_as_millis")]
    latency: Duration,
    ip: IpAddr,
}

fn serialize_duration_as_millis<S: serde::Serializer>(
    d: &Duration,
    s: S,
) -> Result<S::Ok, S::Error> {
    s.serialize_u128(d.as_millis())
}

type Measurements = Arc<Mutex<HashMap<u32, Vec<Measurement>>>>;

fn push_measurement(
    measurements: &mut HashMap<u32, Vec<Measurement>>,
    endpoint_id: u32,
    record: Measurement,
) {
    let records = measurements.entry(endpoint_id).or_default();
    if records.len() == MAX_LATENCY_RECORDS {
        records.remove(0);
    }
    records.push(record);
}

async fn get_measurements(
    State(measurements): State<Measurements>,
) -> Json<HashMap<u32, Vec<Measurement>>> {
    Json(measurements.lock().unwrap().clone())
}

#[tokio::main]
async fn main() {
    let measurements: Measurements = Arc::new(Mutex::new(HashMap::default()));
    let client = Arc::new(reqwest::Client::new());

    let router = Router::new()
        .route("/measurements", get(get_measurements))
        .with_state(Arc::clone(&measurements));

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, router).await.unwrap();
    });

    loop {
        for endpoint in ENDPOINTS {
            let measurements = Arc::clone(&measurements);
            let client = Arc::clone(&client);
            tokio::spawn(async move {
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

                match result {
                    Ok(response) => {
                        if let Some(ip) = response.remote_addr().map(|a| a.ip()) {
                            println!(
                                "[{}] {} ({}) — {}ms",
                                endpoint.id,
                                endpoint.url,
                                ip,
                                latency.as_millis()
                            );
                            let mut measurements = measurements.lock().unwrap();
                            push_measurement(
                                &mut measurements,
                                endpoint.id,
                                Measurement {
                                    started_at,
                                    ended_at,
                                    latency,
                                    ip,
                                },
                            );
                        } else {
                            println!(
                                "[{}] {} — {}ms (no IP)",
                                endpoint.id,
                                endpoint.url,
                                latency.as_millis()
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("[{}] {} — error: {}", endpoint.id, endpoint.url, e);
                    }
                }
            });
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
