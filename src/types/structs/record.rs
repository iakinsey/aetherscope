use crate::types::structs::metadata::{http_response::HttpResponse, uris::Uris};

#[derive(Clone)]
pub struct Record {
    pub uri: String,
    pub task_id: String,
    pub metadata: Vec<RecordMetadata>,
    pub depth: u64,
}

#[derive(Clone)]
pub enum RecordMetadata {
    HttpResponse(HttpResponse),
    Uris(Uris),
}
