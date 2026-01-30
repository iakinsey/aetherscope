use std::str::FromStr;

use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};
use url::Url;
use xxhrs::XXH3_128;

use crate::{
    types::{
        error::AppError,
        structs::record::{Record, RecordMetadata},
        traits::signal::Signal,
    },
    utils::web::{extract_host, extract_site},
};

// Per-URL state updated after each fetch attempt.
// Stores fetch metadata and URL-scoped signals used for ranking,
// freshness decisions, and spam suppression.
#[derive(Debug, Clone, PartialEq)]
pub struct UrlState {
    // Hash of the URL
    pub url_key: Vec<u8>,
    // Hash of the host (scheme+host+port)
    pub host_key: Vec<u8>,
    // Hash of eTLD+1
    pub site_key: Vec<u8>,
    // Most recent successful fetch attempt
    pub last_fetch_ts: DateTime<Utc>,
    // Last http response status
    pub last_status: i16,
    // Last observed HTTP etag header value
    pub etag: Option<String>,
    // Last observed Last-Modified header value
    pub last_modified: Option<DateTime<Utc>>,
    // SimHash fingerprint of fetched content
    pub fp_simhash: Option<i64>,
    // EMA of content change events
    pub change_ema: f64,
    // EMA of 404-like responses
    pub soft404_ema: f64,
    // EMA of low-information content
    pub thin_ema: f64,
    // EMA of latency
    pub latency_ms_ema: f64,
    // EMA of byte response size
    pub bytes_ema: f64,
}

impl Signal for UrlState {
    const CREATE_TABLE_QUERY: &'static str = r#"
        CREATE TABLE IF NOT EXISTS url_state (
            url_key         blob PRIMARY KEY,
            host_key        blob,
            site_key        blob,
            last_fetch_ts   timestamp,
            last_status     smallint,
            etag            text,
            last_modified   timestamp,
            fp_simhash      bigint,
            change_ema      double,
            soft404_ema     double,
            thin_ema        double,
            latency_ms_ema  double,
            bytes_ema       double
        )
    "#;

    const UPSERT_QUERY: &'static str = r#"
        INSERT INTO url_state (
            url_key,
            host_key,
            site_key,
            last_fetch_ts,
            last_status,
            etag,
            last_modified,
            fp_simhash,
            change_ema,
            soft404_ema,
            thin_ema,
            latency_ms_ema,
            bytes_ema
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    fn from_record(record: Record) -> Result<Vec<Self>, AppError> {
        let url = Url::from_str(&record.uri)?;
        let site = extract_site(&url)?;
        let host = extract_host(&url)?;

        let url_key = XXH3_128::hash(record.uri.as_bytes()).to_be_bytes().to_vec();
        let host_key = XXH3_128::hash(host.as_bytes()).to_be_bytes().to_vec();
        let site_key = XXH3_128::hash(site.as_bytes()).to_be_bytes().to_vec();
        //let last_fetch_ts = record

        for m in record.metadata {
            let RecordMetadata::HttpResponse(resp) = m else {
                continue;
            };

            let last_fetch_ts = resp.timestamp;
        }

        unimplemented!();
    }

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.url_key.clone(),
            self.host_key.clone(),
            self.site_key.clone(),
            self.last_fetch_ts.naive_utc(),
            self.last_status,
            self.etag.clone(),
            self.last_modified.map(|t| t.naive_utc()),
            self.fp_simhash,
            self.change_ema,
            self.soft404_ema,
            self.thin_ema,
            self.latency_ms_ema,
            self.bytes_ema
        )
    }
}
