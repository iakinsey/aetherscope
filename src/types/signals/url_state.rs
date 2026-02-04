use std::{str::FromStr, sync::Arc};

use cdrs_tokio::types::IntoRustByName;
use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};
use url::Url;
use xxhrs::XXH3_128;

use crate::{
    types::{
        error::AppError,
        structs::record::{Record, RecordMetadata},
        traits::signal::{DbSession, Signal},
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
    pub fp_minhash: Option<Vec<u64>>,
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

impl UrlState {
    pub async fn get_latest(
        session: Arc<DbSession>,
        url_key: Vec<u8>,
        host_key: Vec<u8>,
        site_key: Vec<u8>,
    ) -> Result<Option<Self>, AppError> {
        const Q: &str = r#"
            SELECT
                last_fetch_ts,
                last_status,
                etag,
                last_modified,
                fp_minhash,
                change_ema,
                soft404_ema,
                thin_ema,
                latency_ms_ema,
                bytes_ema
            FROM url_state
            WHERE url_key = ?
        "#;

        let prepared = session.prepare(Q).await?;
        let result = session
            .exec_with_values(&prepared, query_values!(url_key.clone()))
            .await?;

        let row = match result.response_body()?.into_rows() {
            Some(mut rows) if !rows.is_empty() => rows.remove(0),
            _ => return Ok(None),
        };

        let last_fetch_ts: Option<DateTime<Utc>> = row.get_by_name("last_fetch_ts")?;
        let last_status: Option<i16> = row.get_by_name("last_status")?;
        let etag: Option<String> = row.get_by_name("etag")?;
        let last_modified: Option<DateTime<Utc>> = row.get_by_name("last_modified")?;

        let fp_minhash: Option<Vec<u64>> = {
            let s: Option<String> = row.get_by_name("fp_minhash")?;

            s.map(|txt| {
                txt.split(',')
                    .filter(|x| !x.is_empty())
                    .map(|x| x.parse::<u64>().unwrap())
                    .collect()
            })
        };

        let change_ema: Option<f64> = row.get_by_name("change_ema")?;
        let soft404_ema: Option<f64> = row.get_by_name("soft404_ema")?;
        let thin_ema: Option<f64> = row.get_by_name("thin_ema")?;
        let latency_ms_ema: Option<f64> = row.get_by_name("latency_ms_ema")?;
        let bytes_ema: Option<f64> = row.get_by_name("bytes_ema")?;

        Ok(Some(Self {
            url_key,
            host_key,
            site_key,
            last_fetch_ts: last_fetch_ts.unwrap_or_else(Utc::now),
            last_status: last_status.unwrap_or(0),
            etag,
            last_modified,
            fp_minhash,
            change_ema: change_ema.unwrap_or(0.0),
            soft404_ema: soft404_ema.unwrap_or(0.0),
            thin_ema: thin_ema.unwrap_or(0.0),
            latency_ms_ema: latency_ms_ema.unwrap_or(0.0),
            bytes_ema: bytes_ema.unwrap_or(0.0),
        }))
    }
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
            fp_minhash      text,
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
            fp_minhash,
            change_ema,
            soft404_ema,
            thin_ema,
            latency_ms_ema,
            bytes_ema
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    fn from_record(session: Arc<DbSession>, record: Record) -> Result<Vec<Self>, AppError> {
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
            let last_status = resp.status;
            let etag = resp.response_headers.get("Etag");
            let last_modified = resp.response_headers.get("Last-Modified");
            let fp_minhash = resp.minhash;
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
            self.fp_minhash.clone(),
            self.change_ema,
            self.soft404_ema,
            self.thin_ema,
            self.latency_ms_ema,
            self.bytes_ema
        )
    }
}
