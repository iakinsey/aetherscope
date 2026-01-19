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

pub struct FilterConfig {
    enable: bool,
    enable_bloom_filter: bool,
    enable_hash_set: HashSetConfig,
}
pub struct UniqueFilterConfig {
    filter_urls: FilterConfig,
    filter_domains: FilterConfig,
}
