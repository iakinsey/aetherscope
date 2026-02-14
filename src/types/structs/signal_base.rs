use url::Url;

pub struct SignalBase {
    url: Url,
    url_key: Vec<u8>,
    host_key: Vec<u8>,
    site_key: Vec<u8>,
    prefix_key: Vec<u8>,
}
