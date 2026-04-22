use serde::de::StdError;

pub fn describe_reqwest_error(e: &reqwest::Error) -> String {
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
