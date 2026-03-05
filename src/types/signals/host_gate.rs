use cdrs_tokio::types::IntoRustByName;
use std::sync::Arc;

use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};

use crate::{
    types::{
        error::AppError,
        structs::{
            record::{Record, RecordMetadata},
            signal_base::SignalBase,
        },
        traits::{object_store::ObjectStore, signal::Signal},
    },
    utils::cassandra::DbSession,
};

// Per-host politeness and scheduling gate.
// Enforces crawl delays and exclusive fetch leases so that
// only one worker fetches a host at a time.
#[derive(Debug, Clone, PartialEq)]
pub struct HostGate {
    // Hash of the host (scheme+host+port)
    pub host_key: Vec<u8>,
    // Earliest time at which this host may be fetched again
    pub next_allowed_ts: DateTime<Utc>,
    // Timestamp until which the current lease is valid
    pub lease_until_ts: DateTime<Utc>,
    // Identifier of the worker that currently owns the lease
    pub lease_owner: Option<String>,
}

impl HostGate {
    async fn get_latest(session: Arc<DbSession>, host_key: Vec<u8>) -> Result<Self, AppError> {
        const Q: &str = r#"
            SELECT
                host_key,
                next_allowed_ts,
                lease_until_ts,
                lease_owner
            FROM host_gate
            WHERE host_key = ?;
        "#;

        let prepared = session.prepare(Q).await?;
        let result = session
            .exec_with_values(&prepared, query_values!(host_key.clone()))
            .await?;

        let row = match result.response_body()?.into_rows() {
            Some(mut rows) if !rows.is_empty() => rows.remove(0),
            _ => {
                return Ok(Self {
                    host_key,
                    next_allowed_ts: Utc::now(),
                    lease_until_ts: Utc::now(),
                    lease_owner: None,
                });
            }
        };

        let next_allowed_ts: Option<DateTime<Utc>> = row.get_by_name("next_allowed_ts")?;
        let lease_until_ts: Option<DateTime<Utc>> = row.get_by_name("lease_until_ts")?;
        let lease_owner: Option<String> = row.get_by_name("lease_owner")?;

        Ok(Self {
            host_key,
            next_allowed_ts: next_allowed_ts.unwrap_or(Utc::now()),
            lease_until_ts: lease_until_ts.unwrap_or(Utc::now()),
            lease_owner: lease_owner,
        })
    }
}

impl Signal for HostGate {
    const CREATE_TABLE_QUERY: &'static str = r#"
        CREATE TABLE IF NOT EXISTS host_gate (
            host_key         blob PRIMARY KEY,
            next_allowed_ts  timestamp,
            lease_until_ts   timestamp,
            lease_owner      text
        )
    "#;

    const UPSERT_QUERY: &'static str = r#"
        INSERT INTO host_gate (
            host_key, next_allowed_ts, lease_until_ts, lease_owner
        ) VALUES (?, ?, ?, ?)
    "#;

    async fn from_record(
        _session: Arc<DbSession>,
        _object_store: Arc<dyn ObjectStore>,
        base: SignalBase,
        record: Record,
    ) -> Result<Vec<Self>, AppError> {
        let mut results = vec![];

        for m in &record.metadata {
            if let RecordMetadata::HttpResponse(resp) = m {
                let status = resp.status;
                let response_ts = resp.timestamp;
                let fallback_ts = Some(resp.request.timestamp);

                // no fetch attempt
                let now = match (response_ts, fallback_ts) {
                    (Some(t), _) => t,
                    (None, Some(t)) => t,
                    _ => return Ok(Vec::new()),
                };

                let delay = match status {
                    Some(200..=299) => 1,   // healthy
                    Some(301..=399) => 1,   // healthy, just redirect
                    Some(404) => 10,        // client error
                    Some(429) => 60,        // throttled
                    Some(500..=599) => 120, // server error
                    None => 180,            // timeout/network failure
                    _ => 30,
                };

                let next_allowed_ts = now + chrono::Duration::seconds(delay);

                // short lease window (worker ownership)
                let lease_until_ts = now + chrono::Duration::seconds(15);

                let row = Self {
                    host_key: base.host_key.clone(),
                    next_allowed_ts,
                    lease_until_ts,
                    lease_owner: Some(resp.request.worker_id.clone()),
                };

                results.push(row);
            }
        }

        Ok(results)
    }

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.host_key.clone(),
            self.next_allowed_ts.naive_utc(),
            self.lease_until_ts.naive_utc(),
            self.lease_owner.clone()
        )
    }
}
