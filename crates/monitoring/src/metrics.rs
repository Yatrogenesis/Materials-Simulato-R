//! Prometheus metrics

use metrics::{counter, histogram, gauge};

pub fn record_request(method: &str, path: &str, status: u16, duration_ms: f64) {
    counter!("http_requests_total", "method" => method.to_string(), "path" => path.to_string(), "status" => status.to_string()).increment(1);
    histogram!("http_request_duration_ms", "method" => method.to_string(), "path" => path.to_string()).record(duration_ms);
}

pub fn record_database_query(database: &str, operation: &str, duration_ms: f64) {
    histogram!("database_query_duration_ms", "database" => database.to_string(), "operation" => operation.to_string()).record(duration_ms);
}
