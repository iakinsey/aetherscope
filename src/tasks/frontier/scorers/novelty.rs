// Expected yield (reward novelty)

use crate::types::{
    configs::scorers::novelty_scorer_config::NoveltyScorerConfig, error::AppError,
    traits::frontier_scorer::FrontierScorer,
};

pub struct NoveltyScorer;

impl NoveltyScorer {
    pub fn new(config: NoveltyScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for NoveltyScorer {
    async fn score(self, uris: Vec<String>, origin: &str) -> Result<Vec<(String, i32)>, AppError> {
        unimplemented!()
    }
}
