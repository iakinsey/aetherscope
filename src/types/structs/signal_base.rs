use std::str::FromStr;

use url::Url;
use xxhrs::XXH3_128;

use crate::{
    types::{error::AppError, structs::record::Record},
    utils::web::{extract_host, extract_site, normalize_prefix},
};

pub struct SignalBase {
    pub url: Url,
    pub url_key: Vec<u8>,
    pub host_key: Vec<u8>,
    pub site_key: Vec<u8>,
    pub prefix_key: Vec<u8>,
}

impl SignalBase {
    pub fn new(record: &Record) -> Result<Self, AppError> {
        let url = Url::from_str(&record.uri)?;
        let host = extract_host(&url)?;
        let site = extract_site(&url)?;
        let path = url.path().to_ascii_lowercase();
        let normalized_prefix = normalize_prefix(&path);

        let site_key = XXH3_128::hash(site.as_bytes()).to_be_bytes().to_vec();
        let host_key = XXH3_128::hash(host.as_bytes()).to_be_bytes().to_vec();
        let url_key = XXH3_128::hash(record.uri.as_bytes()).to_be_bytes().to_vec();
        let prefix_key = XXH3_128::hash(normalized_prefix.as_bytes())
            .to_be_bytes()
            .to_vec();

        Ok(Self {
            url,
            url_key,
            host_key,
            site_key,
            prefix_key,
        })
    }
}
