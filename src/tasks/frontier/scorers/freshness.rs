// Freshness (separate new from recrawl)

use crate::types::{
    configs::scorers::freshness_scorer_config::FreshnessScorerConfig, error::AppError,
    structs::metadata::signals_extracted::ExtractedSignal, traits::frontier_scorer::FrontierScorer,
};

pub struct FreshnessScorer;

impl FreshnessScorer {
    pub fn new(config: FreshnessScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for FreshnessScorer {
    async fn score(self, signals: Vec<ExtractedSignal>) -> Result<f32, AppError> {
        unimplemented!()
    }
}
