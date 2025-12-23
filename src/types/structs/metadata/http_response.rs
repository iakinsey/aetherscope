use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub req_headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: i64,
    pub request: HttpRequest,
    pub resp_headers: HashMap<String, String>,
    pub key: Option<String>,
    pub error: Option<String>,
}
