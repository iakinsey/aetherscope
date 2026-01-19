use crate::types::{
    configs::robots_filter_config::RobotsFilterConfig, error::AppError,
    traits::frontier_filter::FrontierFilter,
};

pub struct UniqueFilter {}

impl UniqueFilter {
    pub fn new(robots_filter_config: RobotsFilterConfig) -> Result<Self, AppError> {
        unimplemented!()
    }
}

impl FrontierFilter for UniqueFilter {
    async fn perform(
        self,
        uris: Vec<String>,
        _origin: &str,
    ) -> Result<Vec<(String, bool)>, AppError> {
        unimplemented!()
    }
}
