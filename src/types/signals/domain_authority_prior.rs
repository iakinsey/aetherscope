use std::sync::Arc;

use async_trait::async_trait;
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

fn update_prior_ema(
    prev: f64,
    prev_ts: DateTime<Utc>,
    now: DateTime<Utc>,
    x: f64,
    tau_seconds: f64,
) -> f64 {
    let dt = (now - prev_ts).num_seconds().max(0) as f64;
    let decay = (-dt / tau_seconds).exp();
    prev * decay + x * (1.0 - decay)
}

// Static or slowly changing authority prior per site.
// Used to bootstrap importance before sufficient crawl data exists.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainAuthorityPrior {
    // Hash of eTLD+1
    pub domain_key: Vec<u8>,
    // Authority prior score
    pub authority: f64,
    // Most recent update timestamp
    pub updated_ts: DateTime<Utc>,
}

// TODO
impl DomainAuthorityPrior {
    pub async fn get_latest(
        session: Arc<DbSession>,
        domain_key: Vec<u8>,
    ) -> Result<Self, AppError> {
        unimplemented!()
    }
}

#[async_trait]
impl Signal for DomainAuthorityPrior {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "domain_authority_prior"
    }

    fn create_table_query() -> &'static str
    where
        Self: Sized,
    {
        r#"
        CREATE TABLE IF NOT EXISTS domain_authority_prior (
            domain_key  blob PRIMARY KEY,
            authority   double,
            updated_ts  timestamp
        )
    "#
    }

    fn upsert_query() -> &'static str
    where
        Self: Sized,
    {
        r#"
        INSERT INTO domain_authority_prior (
            domain_key, authority, updated_ts
        ) VALUES (?, ?, ?)
    "#
    }

    async fn from_record(
        session: Arc<DbSession>,
        _object_store: Arc<dyn ObjectStore>,
        base: SignalBase,
        record: Record,
    ) -> Result<Vec<Box<dyn Signal>>, AppError>
    where
        Self: Sized,
    {
        let domain_key = base.site_key;
        let mut updated_ts: Option<DateTime<Utc>> = None;
        let mut success = false;

        for m in &record.metadata {
            if let RecordMetadata::HttpResponse(resp) = m {
                updated_ts = resp.timestamp;
                if let Some(status) = resp.status {
                    if (200..400).contains(&status) {
                        success = true;
                    }
                }
                break;
            }
        }
        let updated_ts = match updated_ts {
            Some(t) => t,
            None => return Ok(Vec::new()),
        };

        let latest = Self::get_latest(session.clone(), domain_key.clone()).await?;
        let authority = update_prior_ema(
            latest.authority,
            latest.updated_ts,
            updated_ts,
            if success { 1.0 } else { 0.0 },
            90.0 * 24.0 * 3600.0,
        );

        Ok(vec![Box::new(Self {
            domain_key,
            authority,
            updated_ts,
        })])
    }

    fn bind_values(&self) -> QueryValues {
        query_values!(
            self.domain_key.clone(),
            self.authority,
            self.updated_ts.naive_utc()
        )
    }
}
