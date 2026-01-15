use crate::types::{error::AppError, traits::frontier_filter::FrontierFilter};

pub struct RobotsFilter;

impl FrontierFilter for RobotsFilter {
    async fn filter(uris: Vec<String>, origin: &str) -> Result<bool, AppError> {
        unimplemented!()
    }
}
