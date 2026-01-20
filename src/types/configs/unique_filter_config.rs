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

pub struct BloomFilterConfig {}

pub struct FilterConfig {
    bloom_filter_config: BloomFilterConfig,
    hash_set_config: HashSetConfig,
}
pub struct UniqueFilterConfig {
    filter_urls: FilterConfig,
    filter_domains: FilterConfig,
}
