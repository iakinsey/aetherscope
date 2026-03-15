// Importance (depth from initial seed, how many unique inlinks)

use crate::types::{
    configs::scorers::importance_scorer_config::ImportanceScorerConfig, error::AppError,
    structs::metadata::signals_extracted::ExtractedSignal, traits::frontier_scorer::FrontierScorer,
};

pub struct ImportanceScorer;

impl ImportanceScorer {
    pub fn new(config: ImportanceScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for ImportanceScorer {
    async fn score(self, signals: Vec<ExtractedSignal>) -> Result<f32, AppError> {
        unimplemented!()
    }
}
