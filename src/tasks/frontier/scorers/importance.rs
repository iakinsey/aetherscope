// Importance (depth from initial seed, how many unique inlinks)

use crate::types::{
    configs::scorers::importance_scorer_config::ImportanceScorerConfig, error::AppError,
    traits::frontier_scorer::FrontierScorer,
};

pub struct ImportanceScorer;

impl ImportanceScorer {
    pub fn new(config: ImportanceScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for ImportanceScorer {
    async fn score(self, uris: Vec<String>, origin: &str) -> Result<Vec<(String, i32)>, AppError> {
        unimplemented!()
    }
}
