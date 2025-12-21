use crate::types::structs::metadata::{
    html_content::HtmlContent, http_request::HttpRequest, http_response::HttpResponse,
};

pub struct Record {
    pub uri: String,
    pub task_id: String,
    pub metadata: Vec<RecordMetadata>,
}

pub enum RecordMetadata {
    HtmlContent(HtmlContent),
    HttpRequest(HttpRequest),
    HttpResponse(HttpResponse),
}
