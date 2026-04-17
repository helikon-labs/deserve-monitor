use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use reqwest::header::CONTENT_TYPE;

struct Endpoint {
    id: u32,
    url: &'static str,
}

const ENDPOINTS: &[Endpoint] = &[
    Endpoint { id: 0, url: "https://asset-hub.polkadot.rpc.deserve.network" },
    Endpoint { id: 1, url: "https://asset-hub-polkadot.ibp.network" },
];
const MAX_LATENCY_RECORDS: usize = 10;
const RPC_BODY: &str = r#"{"id":"1","jsonrpc":"2.0","method":"chain_getFinalizedHead","params":[]}"#;

struct Measurement {
    started_at: u128,
    ended_at: u128,
    latency: Duration,
    ip: IpAddr,
}

fn push_measurement(measurements: &mut HashMap<u32, Vec<Measurement>>, endpoint_id: u32, record: Measurement) {
    let records = measurements.entry(endpoint_id).or_default();
    if records.len() == MAX_LATENCY_RECORDS {
        records.remove(0);
    }
    records.push(record);
}

#[tokio::main]
async fn main() {
    let mut measurements: HashMap<u32, Vec<Measurement>> = HashMap::new();
    let client = reqwest::Client::new();

    loop {
        for endpoint in ENDPOINTS {
            let start = Instant::now();
            let started_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            let result = client
                .post(endpoint.url)
                .header(CONTENT_TYPE, "application/json")
                .body(RPC_BODY)
                .send()
                .await;
            let latency = start.elapsed();
            let ended_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

            match result {
                Ok(response) => {
                    if let Some(ip) = response.remote_addr().map(|a| a.ip()) {
                        println!("[{}] {} ({}) — {}ms", endpoint.id, endpoint.url, ip, latency.as_millis());
                        push_measurement(&mut measurements, endpoint.id, Measurement { started_at, ended_at, latency, ip });
                    } else {
                        println!("[{}] {} — {}ms (no IP)", endpoint.id, endpoint.url, latency.as_millis());
                    }
                }
                Err(e) => {
                    eprintln!("[{}] {} — error: {}", endpoint.id, endpoint.url, e);
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
