use chrono::{DateTime, Utc};

use crate::types::structs::metadata::{
    http_response::HttpResponse, signals_extracted::SignalsExtracted, uris::Uris,
};

#[derive(Clone)]
pub struct Record {
    pub uri: String,
    pub task_id: String,
    pub metadata: Vec<RecordMetadata>,
    pub depth: i32,
    pub discovered: DateTime<Utc>,
}

#[derive(Clone)]
pub enum RecordMetadata {
    HttpResponse(HttpResponse),
    Uris(Uris),
    SignalsExtracted(SignalsExtracted),
}
