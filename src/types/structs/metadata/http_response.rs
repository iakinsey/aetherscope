use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub request_headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: Option<i64>,
    pub request: HttpRequest,
    pub response_headers: HashMap<String, String>,
    pub key: Option<String>,
    pub error: Option<String>,
}
