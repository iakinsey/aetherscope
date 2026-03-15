// Spam (proximity to adversarial websites, content variance between pages such as templates or same titles, session ids, repeated path segments)

use crate::types::{
    configs::scorers::spam_scorer_config::SpamScorerConfig, error::AppError,
    structs::metadata::signals_extracted::ExtractedSignal, traits::frontier_scorer::FrontierScorer,
};

pub struct SpamScorer;

impl SpamScorer {
    pub fn new(config: SpamScorerConfig) {
        unimplemented!()
    }
}

impl FrontierScorer for SpamScorer {
    async fn score(self, signals: Vec<ExtractedSignal>) -> Result<f32, AppError> {
        unimplemented!()
    }
}
