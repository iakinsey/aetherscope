use chrono::{DateTime, Utc};

use crate::types::{error::AppError, traits::signal::Signal};

#[derive(Debug, Clone, PartialEq)]
pub struct UrlState {
    // Hash of the URL
    pub url_key: Vec<u8>,
    // Hash of the host (scheme+host+port)
    pub host_key: Vec<u8>,
    // Hash of eTLD+1
    pub site_key: Vec<u8>,
    // Most recent successful fetch attempt
    pub last_fetch_ts: Option<DateTime<Utc>>,
    // Last http response status
    pub last_status: Option<i16>,
    // Last observed HTTP etag header value
    pub etag: Option<String>,
    // Last observed Last-Modified header value
    pub last_modified: Option<DateTime<Utc>>,
    // SimHash fingerprint of fetched content
    pub fp_simhash: Option<i64>,
    // EMA of content change events
    pub change_ema: Option<f64>,
    // EMA of 404-like responses
    pub soft404_ema: Option<f64>,
    // EMA of low-information content
    pub thin_ema: Option<f64>,
    // EMA of latency
    pub latency_ms_ema: Option<f64>,
    // EMA of byte response size
    pub bytes_ema: Option<f64>,
}

impl Signal for UrlState {
    fn get_query() -> String {
        unimplemented!()
    }

    fn create_table() -> Result<(), AppError> {
        unimplemented!()
    }
}
