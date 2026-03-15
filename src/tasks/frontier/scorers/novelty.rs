// Expected yield (reward novelty)

use crate::types::{
    configs::scorers::novelty_scorer_config::NoveltyScorerConfig, error::AppError,
    structs::metadata::signals_extracted::ExtractedSignal, traits::frontier_scorer::FrontierScorer,
};

pub struct NoveltyScorer;

impl NoveltyScorer {
    pub fn new(config: NoveltyScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for NoveltyScorer {
    async fn score(self, signals: Vec<ExtractedSignal>) -> Result<f32, AppError> {
        unimplemented!()
    }
}
