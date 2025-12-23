use crate::types::structs::metadata::http_response::HttpResponse;

pub struct Record {
    pub uri: String,
    pub task_id: String,
    pub metadata: Vec<RecordMetadata>,
}

pub enum RecordMetadata {
    HttpResponse(HttpResponse),
}
