# deserve-monitor

RPC latency monitor for DeServe.

## What it does

Polls a set of JSON-RPC endpoints every 10 seconds using `chain_getFinalizedHead`, records the latency and resolved IP
of each response, and exposes the last 50 measurements per endpoint over HTTP.

## API

### `GET /measurements`

Returns the last 50 measurements for each endpoint, keyed by endpoint ID.

```json
{
  "0": [
    {
      "started_at": 1713340000000,
      "ended_at": 1713340000123,
      "latency": 123,
      "ip": "1.2.3.4"
    }
  ]
}
```

Fields:
- `started_at` — Unix timestamp in milliseconds when the request was sent
- `ended_at` — Unix timestamp in milliseconds when the response was received
- `latency` — round-trip time in milliseconds
- `ip` — DNS-resolved IP address used for the connection

## Running

```sh
cargo run
```

Server listens on `0.0.0.0:1881`.
