// Coverage (penalize websites with large url counts)

use crate::types::{
    configs::scorers::coverage_scorer_config::CoverageScorerConfig, error::AppError,
    structs::metadata::signals_extracted::ExtractedSignal, traits::frontier_scorer::FrontierScorer,
};

pub struct CoverageScorer;

impl CoverageScorer {
    pub fn new(config: CoverageScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for CoverageScorer {
    async fn score(self, signals: Vec<ExtractedSignal>) -> Result<f32, AppError> {
        unimplemented!()
    }
}
