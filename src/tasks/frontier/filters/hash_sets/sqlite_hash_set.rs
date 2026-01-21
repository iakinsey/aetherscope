use crate::types::{configs::unique_filter_config::SqliteHashSetConfig, traits::hash_set::HashSet};

pub struct SqliteHashSet;

impl SqliteHashSet {
    pub fn new(config: SqliteHashSetConfig) {
        unimplemented!();
    }
}

impl HashSet for SqliteHashSet {
    fn contains_entities(entities: Vec<String>) -> Vec<(String, bool)> {
        unimplemented!();
    }
}
