// Coverage (penalize websites with large url counts)

use crate::types::{
    configs::scorers::coverage_scorer_config::CoverageScorerConfig, error::AppError,
    traits::frontier_scorer::FrontierScorer,
};

pub struct CoverageScorer;

impl CoverageScorer {
    pub fn new(config: CoverageScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for CoverageScorer {
    async fn score(self, uris: Vec<String>, origin: &str) -> Result<Vec<(String, i32)>, AppError> {
        unimplemented!()
    }
}
