use std::sync::Arc;

use async_trait::async_trait;

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
            metadata::execution_context::ExecutionContext, record::Record, signal_base::SignalBase,
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

macro_rules! collect_signals {
    ($session:expr, $store:expr, $base:expr, $record:expr, [$($t:ty),* $(,)?]) => {{
        let mut out: Vec<Box<dyn Signal>> = Vec::new();
        $(
            out.extend(
                <$t as Signal>::from_record(
                    $session.clone(),
                    $store.clone(),
                    $base.clone(),
                    $record.clone(),
                ).await?
            );
        )*
        out
    }};
}

#[async_trait]
impl<'a> Task for SignalExtractor<'a> {
    async fn on_message(&self, ctx: ExecutionContext, message: Record) -> Result<Record, AppError> {
        let signal_base = SignalBase::new(&message)?;
        let signals = collect_signals!(
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

        unimplemented!()
    }
}
