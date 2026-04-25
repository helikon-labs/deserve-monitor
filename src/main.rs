#![warn(clippy::disallowed_types)]

use crate::types::{EndpointStats, Measurement, Measurements};
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use reqwest::header::CONTENT_TYPE;
use rustc_hash::FxHashMap as HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod api;
mod constants;
mod data;
mod types;
mod util;

fn push_measurement(
    measurements: &mut HashMap<u32, VecDeque<Measurement>>,
    endpoint_id: u32,
    record: Measurement,
) {
    let records = measurements.entry(endpoint_id).or_default();
    if records.len() == constants::MAX_LATENCY_RECORDS {
        records.pop_front();
    }
    records.push_back(record);
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
    let router = Router::new()
        .route("/", get(api::get_info))
        .route("/chains", get(api::get_chains))
        .route("/endpoints", get(api::get_endpoints))
        .route("/chains/{id}/endpoints", get(api::get_chain_endpoints))
        .route("/chains/{id}/providers", get(api::get_chain_providers))
        .route("/measurements", get(get_measurements))
        .route("/providers", get(api::get_providers))
        .route(
            "/providers/{id}/endpoints",
            get(api::get_provider_endpoints),
        )
        .with_state(Arc::clone(&measurements));

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", constants::API_PORT))
            .await
            .unwrap();
        axum::serve(listener, router).await.unwrap();
    });

    let mut _round_index: u64 = 0;
    loop {
        let mut round: Vec<(u32, Measurement)> = Vec::new();
        let client = reqwest::Client::builder()
            .connect_timeout(constants::CONNECTION_TIMEOUT)
            .timeout(constants::REQUEST_TIMEOUT)
            .build()
            .unwrap();
        for endpoint in data::ENDPOINTS.iter().rev() {
            for _ in 0..5 {
                let started_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let result = client
                    .post(format!("https://{}", endpoint.url))
                    .header(CONTENT_TYPE, "application/json")
                    .body(endpoint.service_type.get_request_body())
                    .send()
                    .await;
                let ended_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let latency = Duration::from_millis((ended_at - started_at) as u64);

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
                        let error = util::describe_reqwest_error(&e);
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
                tokio::time::sleep(constants::SERIES_INTERVAL).await;
            }
        }
        {
            let mut measurements = measurements.lock().unwrap();
            for (endpoint_id, measurement) in round {
                push_measurement(&mut measurements, endpoint_id, measurement);
            }
        }
        tokio::time::sleep(constants::POLL_INTERVAL).await;
        _round_index += 1;
    }
}
