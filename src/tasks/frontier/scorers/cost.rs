// Cost (estimate latency from previous requests, how many errors occurred on the host prior, limit how many per host, robots.txt limits)

use crate::types::{
    configs::scorers::cost_scorer_config::CostScorerConfig, error::AppError,
    structs::metadata::signals_extracted::ExtractedSignal, traits::frontier_scorer::FrontierScorer,
};

pub struct CostScorer;

impl CostScorer {
    pub fn new(config: CostScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for CostScorer {
    async fn score(self, signals: Vec<ExtractedSignal>) -> Result<f32, AppError> {
        unimplemented!()
    }
}
