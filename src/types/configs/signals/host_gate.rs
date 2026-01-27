use cdrs_tokio::{query::QueryValues, query_values};
use chrono::{DateTime, Utc};

use crate::types::traits::signal::Signal;

#[derive(Debug, Clone, PartialEq)]
pub struct HostGate {
    // Hash of the host (scheme+host+port)
    pub host_key: Vec<u8>,
    // Earliest time at which this host may be fetched again
    pub next_allowed_ts: DateTime<Utc>,
    // Timestamp until which the current lease is valid
    pub lease_until_ts: DateTime<Utc>,
    // Identifier of the worker that currently owns the lease
    pub lease_owner: String,
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

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.host_key.clone(),
            self.next_allowed_ts.naive_utc(),
            self.lease_until_ts.naive_utc(),
            self.lease_owner.clone()
        )
    }
}
