// Cost (estimate latency from previous requests, how many errors occurred on the host prior, limit how many per host, robots.txt limits)

use crate::types::{
    configs::scorers::cost_scorer_config::CostScorerConfig,
    error::AppError,
    signals::host_stats_stripe::HostStatsStripe,
    structs::metadata::{merged_host_cost::MergedHostCost, signals_extracted::ExtractedSignal},
    traits::frontier_scorer::FrontierScorer,
};

pub struct CostScorer;

impl CostScorer {
    pub fn new(config: CostScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for CostScorer {
    async fn score(self, signals: Vec<ExtractedSignal>) -> Result<f32, AppError> {
        let url_state = signals
            .iter()
            .find_map(|s| s.value.url_state().ok())
            .ok_or_else(|| AppError::MissingSignal("UrlState".into()))?;

        let host_stats: Vec<&HostStatsStripe> = signals
            .iter()
            .filter_map(|s| s.value.host_stats_stripe().ok())
            .collect();

        if host_stats.is_empty() {
            return Err(AppError::MissingSignal("HostStatsStripe".into()));
        }

        let host = MergedHostCost::from_host_stats_stripes(&host_stats);

        unimplemented!()
    }
}
