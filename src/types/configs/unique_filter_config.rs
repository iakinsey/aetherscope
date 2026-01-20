pub struct SqliteHashSetConfig {
    enable: bool,
    path: String,
}

pub struct RedisHashSetConfig {
    enable: bool,
    uri: String,
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

pub struct FilterConfig {
    pub bloom_filter: BloomFilterConfig,
    pub hash_set: HashSetConfig,
}
pub struct UniqueFilterConfig {
    pub filter_urls: FilterConfig,
    pub filter_domains: FilterConfig,
}
