use std::collections::HashMap;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub request_headers: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: Option<i64>,
    pub request: HttpRequest,
    pub response_headers: HashMap<String, String>,
    pub key: Option<String>,
    pub error: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}
