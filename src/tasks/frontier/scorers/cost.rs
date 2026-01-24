// Cost (estimate latency from previous requests, how many errors occurred on the host prior, limit how many per host, robots.txt limits)

use crate::types::{
    configs::scorers::cost_scorer_config::CostScorerConfig, error::AppError,
    traits::frontier_scorer::FrontierScorer,
};

pub struct CostScorer;

impl CostScorer {
    pub fn new(config: CostScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for CostScorer {
    async fn score(self, uris: Vec<String>, origin: &str) -> Result<Vec<(String, i32)>, AppError> {
        unimplemented!()
    }
}
