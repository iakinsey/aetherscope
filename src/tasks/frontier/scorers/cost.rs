// Cost (estimate latency from previous requests, how many errors occurred on the host prior, limit how many per host, robots.txt limits)

use crate::{
    types::{
        configs::scorers::cost_scorer_config::CostScorerConfig,
        error::AppError,
        signals::host_stats_stripe::HostStatsStripe,
        structs::metadata::{merged_host_cost::MergedHostCost, signals_extracted::ExtractedSignal},
        traits::frontier_scorer::FrontierScorer,
    },
    utils::math::norm_log1p,
};

pub struct CostScorer {
    config: CostScorerConfig,
}

impl CostScorer {
    pub fn new(config: CostScorerConfig) -> Self {
        Self { config }
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
        let latency_ms = url_state.latency_ms_ema.max(host.latency_ms_ema);
        let bytes = url_state.bytes_ema.max(host.bytes_ema);
        let latency_cost = norm_log1p(latency_ms, self.config.max_latency_ms);
        let bytes_cost = norm_log1p(bytes, self.config.max_bytes);

        let cost = self.config.latency_weight * latency_cost
            + self.config.bytes_weight * bytes_cost
            + self.config.timeout_weight * host.timeout_ema
            + self.config.http5xx_weight * host.http5xx_ema
            + self.config.http429_weight * host.http429_ema
            + self.config.redirect_weight * host.redirect_ema;

        Ok(cost.clamp(0.0, 1.0) as f32)
    }
}
