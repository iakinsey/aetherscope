use std::sync::Arc;

use async_trait::async_trait;
use cdrs_tokio::query::QueryValues;

use crate::{
    types::{
        configs::tasks::signal_extractor_config::SignalExtractorConfig,
        error::AppError,
        signals::{
            domain_authority_prior::DomainAuthorityPrior, domain_coverage::DomainCoverage,
            host_gate::HostGate, host_stats_stripe::HostStatsStripe, inlink_agg::InlinkAgg,
            prefix_stats::PrefixStats, url_depth::UrlDepth, url_state::UrlState,
        },
        structs::{
            metadata::{
                execution_context::ExecutionContext,
                signals_extracted::{ExtractedSignal, SignalsExtracted},
            },
            record::{Record, RecordMetadata},
            signal_base::SignalBase,
        },
        traits::{object_store::ObjectStore, signal::Signal, task::Task},
    },
    utils::{cassandra::DbSession, dependencies::dependencies},
};

pub struct SignalExtractor<'a> {
    config: &'a SignalExtractorConfig<'a>,
    object_store: Arc<dyn ObjectStore>,
    db_session: Arc<DbSession>,
}

impl<'a> SignalExtractor<'a> {
    pub async fn new(config: &'a SignalExtractorConfig<'a>) -> Result<Self, AppError> {
        let object_store = dependencies()
            .lock()
            .await
            .get_object_store(&config.object_store)?;

        let db_session = dependencies()
            .lock()
            .await
            .get_db_session(&config.db_session)?;

        Ok(Self {
            config,
            object_store,
            db_session,
        })
    }
}

macro_rules! collect_upserts {
    ($session:expr, $store:expr, $base:expr, $record:expr, [$($t:ty),* $(,)?]) => {{
        let mut out: Vec<(String, String, Vec<QueryValues>)> = Vec::new();
        $(
            let signals = <$t as Signal>::from_record(
                $session.clone(),
                $store.clone(),
                $base.clone(),
                $record.clone(),
            ).await?;

            let values: Vec<QueryValues> = signals
                .into_iter()
                .map(|s| s.bind_values())
                .collect();

            if !values.is_empty() {
                out.push((
                    <$t as Signal>::name().to_string(),
                    <$t as Signal>::upsert_query().to_string(),
                    values
                ));
            }
        )*
        out
    }};
}

#[async_trait]
impl<'a> Task for SignalExtractor<'a> {
    async fn on_message(&self, ctx: ExecutionContext, message: Record) -> Result<Record, AppError> {
        let mut signals_extracted = vec![];
        let signal_base = SignalBase::new(&message)?;
        let mut metadata = message.metadata.clone();
        let signals = collect_upserts!(
            self.db_session,
            self.object_store,
            signal_base,
            message,
            [
                DomainAuthorityPrior,
                DomainCoverage,
                HostGate,
                HostStatsStripe,
                InlinkAgg,
                PrefixStats,
                UrlDepth,
                UrlState
            ]
        );

        for (name, query, rows) in signals {
            let prepared = self.db_session.prepare(query).await?;

            for row in rows {
                let err = self
                    .db_session
                    .exec_with_values(&prepared, row)
                    .await
                    .err()
                    .map(|e| e.to_string());

                signals_extracted.push(ExtractedSignal {
                    name: name.clone(),
                    error: err,
                });
            }
        }

        metadata.push(RecordMetadata::SignalsExtracted(SignalsExtracted {
            signals: signals_extracted,
        }));

        Ok(Record {
            uri: message.uri,
            task_id: message.task_id,
            metadata: metadata,
            depth: message.depth,
            discovered: message.discovered,
        })
    }
}
