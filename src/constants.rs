use std::time::Duration;

pub const MAX_LATENCY_RECORDS: usize = 360;
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
pub const POLL_INTERVAL: Duration = Duration::from_secs(10);
pub const RPC_BODY: &str =
    r#"{"id":"1","jsonrpc":"2.0","method":"chain_getFinalizedHead","params":[]}"#;
pub const API_PORT: u16 = 1881;
