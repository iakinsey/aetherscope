use crate::{
    tasks::frontier::filters::hash_sets::{
        redis_hash_set::RedisHashSet, sqlite_hash_set::SqliteHashSet,
    },
    types::{error::AppError, traits::check_hash_set::CheckHashSet},
};

#[derive(Clone)]
pub struct SqliteHashSetConfig {
    pub path: String,
}

#[derive(Clone)]
pub struct RedisHashSetConfig {
    pub uri: String,
}

pub enum HashSetConfig {
    Sqlite(SqliteHashSetConfig),
    Redis(RedisHashSetConfig),
    Empty,
}

pub struct BloomFilterConfig {
    pub enable: bool,
    pub false_positive_rate: f64,
    pub expected_size: usize,
}

pub struct UniqueFilterConfig {
    pub bloom_filter: BloomFilterConfig,
    pub hash_set: HashSetConfig,
}
