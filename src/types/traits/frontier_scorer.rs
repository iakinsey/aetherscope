use crate::types::{error::AppError, structs::metadata::signals_extracted::ExtractedSignal};

pub trait FrontierScorer {
    async fn score(self, signals: Vec<ExtractedSignal>) -> Result<f32, AppError>;
}
