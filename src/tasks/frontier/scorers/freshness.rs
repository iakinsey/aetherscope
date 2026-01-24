// Freshness (separate new from recrawl)

use crate::types::{
    configs::scorers::freshness_scorer_config::FreshnessScorerConfig, error::AppError,
    traits::frontier_scorer::FrontierScorer,
};

pub struct FreshnessScorer;

impl FreshnessScorer {
    pub fn new(config: FreshnessScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for FreshnessScorer {
    async fn score(self, uris: Vec<String>, origin: &str) -> Result<Vec<(String, i32)>, AppError> {
        unimplemented!()
    }
}
