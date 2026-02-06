use std::{net::IpAddr, sync::Arc};

use bytes::Bytes;
use psl::{domain, domain_str};
use reqwest::Client;
use url::{ParseError, Url};

use crate::types::{
    error::AppError, structs::metadata::http_response::HttpResponse,
    traits::object_store::ObjectStore,
};

pub fn get_user_agent(user_agent: Option<String>) -> String {
    if let Some(user_agent) = user_agent {
        user_agent
    } else {
        format!("{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}

/*
    TODO
        - Resolve ./ and ../
        - Remove default hosts/ports (eg 80/443) if set
*/
pub fn normalize_url(origin: &Url, href: &str) -> Result<Url, ParseError> {
    let h = href.trim();

    // Empty / whitespace-only: treat as origin.
    if h.is_empty() {
        return Ok(origin.clone());
    }

    // Already absolute (has a scheme).
    if let Ok(u) = Url::parse(h) {
        return Ok(u);
    }

    // Scheme-relative: //host/path
    if let Some(rest) = h.strip_prefix("//") {
        return Url::parse(&format!("{}://{}", origin.scheme(), rest));
    }

    // Pure fragment: #id
    if h.starts_with('#') {
        let mut u = origin.clone();
        u.set_fragment(Some(&h[1..]));
        return Ok(u);
    }

    // Pure query: ?k=v
    if h.starts_with('?') {
        let mut u = origin.clone();
        u.set_query(Some(&h[1..]));
        u.set_fragment(None);
        return Ok(u);
    }

    // Heuristic: domain-ish without scheme (or domain/path) -> treat as absolute host,
    // don't resolve against origin path.
    if looks_like_domainish(h) {
        return Url::parse(&format!("{}://{}", origin.scheme(), h));
    }

    // Everything else: resolve as relative to origin per RFC 3986.
    origin.join(h)
}

fn looks_like_domainish(s: &str) -> bool {
    // reject obvious relative-only forms
    if s.starts_with('/') || s.starts_with('.') || s.starts_with('?') || s.starts_with('#') {
        return false;
    }
    // reject anything with spaces
    if s.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    // If it has a scheme already, Url::parse() would have succeeded earlier.
    let end = s.find(|c| c == '?' || c == '#').unwrap_or_else(|| s.len());
    let head = &s[..end];

    if head.starts_with('[') {
        return true;
    }

    let slash_pos = head.find('/').unwrap_or_else(|| head.len());
    let hostish = &head[..slash_pos];

    // dot in host portion => domain-ish
    if hostish.contains('.') {
        return true;
    }

    // digit-leading + colon in host portion => likely host:port (IPv4:port)
    if hostish.chars().next().is_some_and(|c| c.is_ascii_digit()) && hostish.contains(':') {
        return true;
    }

    false
}

pub async fn fetch_http_simple(client: Client, uri: &str) -> Result<Bytes, AppError> {
    let req = client.get(uri).build()?;
    let resp = client.execute(req).await?;
    let status = resp.status().as_u16();

    if status < 200 || status > 299 {
        return Err(AppError::FetchError(status, uri.to_string()));
    }

    Ok(resp.bytes().await?)
}

pub fn get_robots_url(input: &str) -> Result<String, AppError> {
    let url = Url::parse(input)?;

    let scheme = url.scheme();
    let host = match url.host_str() {
        Some(h) => h,
        None => return Err(AppError::from(url::ParseError::EmptyHost)),
    };
    let port = url.port();

    let base = match port {
        Some(p) => format!("{scheme}://{host}:{p}"),
        None => format!("{scheme}://{host}"),
    };

    Ok(format!("{base}/robots.txt"))
}

pub fn extract_host(u: &Url) -> Result<String, AppError> {
    let scheme = u.scheme().to_ascii_lowercase();

    let host = u
        .host_str()
        .ok_or(AppError::Generic(format!(
            "failed to get host of url {}",
            u.to_string()
        )))?
        .to_ascii_lowercase();

    let port = u.port_or_known_default().expect("known default port");

    Ok(format!("{}://{}:{}", scheme, host, port))
}

pub fn extract_site(url: &Url) -> Result<String, AppError> {
    let host = url.host_str().ok_or(AppError::Generic(format!(
        "failed to get host of url {}",
        url.to_string()
    )))?;

    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ip.to_string());
    }

    let etld_plus_one = domain_str(host).unwrap_or(host);

    Ok(etld_plus_one.to_string())
}

pub async fn is_soft404(
    object_store: Arc<dyn ObjectStore>,
    resp: HttpResponse,
) -> Result<bool, AppError> {
    // TODO start here next, develop FSM that can detect attributes indicating a soft 404
    // You will need to update the FSM system
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    fn origin_https() -> Url {
        Url::parse("https://example.com/a/b?orig=1#frag").unwrap()
    }

    fn origin_http_with_port() -> Url {
        Url::parse("http://example.com:8080/dir/index.html").unwrap()
    }

    #[test]
    fn empty_href_returns_origin() {
        let o = origin_https();
        let u = normalize_url(&o, "").unwrap();
        assert_eq!(u.as_str(), o.as_str());

        let u2 = normalize_url(&o, "   ").unwrap();
        assert_eq!(u2.as_str(), o.as_str());
    }

    #[test]
    fn absolute_url_passthrough() {
        let o = origin_https();
        let u = normalize_url(&o, "https://help.com/x?y=1#z").unwrap();
        assert_eq!(u.as_str(), "https://help.com/x?y=1#z");
    }

    #[test]
    fn scheme_relative_uses_origin_scheme() {
        let o = origin_https();
        let u = normalize_url(&o, "//help.com/x").unwrap();
        assert_eq!(u.as_str(), "https://help.com/x");

        let o2 = origin_http_with_port();
        let u2 = normalize_url(&o2, "//help.com/x").unwrap();
        assert_eq!(u2.as_str(), "http://help.com/x");
    }

    #[test]
    fn fragment_only_updates_fragment_and_preserves_other_parts() {
        let o = origin_https();
        let u = normalize_url(&o, "#helpme").unwrap();
        assert_eq!(u.as_str(), "https://example.com/a/b?orig=1#helpme");
    }

    #[test]
    fn query_only_sets_query_and_clears_fragment() {
        let o = origin_https();
        let u = normalize_url(&o, "?query=1234").unwrap();
        assert_eq!(u.as_str(), "https://example.com/a/b?query=1234");
    }

    #[test]
    fn root_relative_path_replaces_path() {
        let o = origin_https();
        let u = normalize_url(&o, "/test/1234").unwrap();
        assert_eq!(u.as_str(), "https://example.com/test/1234");
    }

    #[test]
    fn relative_path_joins_against_origin_directory() {
        let o = Url::parse("https://example.com/a/b").unwrap();
        let u = normalize_url(&o, "example/test").unwrap();
        assert_eq!(u.as_str(), "https://example.com/a/example/test");
    }

    #[test]
    fn dot_relative_path_is_not_domainish() {
        let o = Url::parse("https://example.com/a/b").unwrap();
        let u = normalize_url(&o, "./c").unwrap();
        assert_eq!(u.as_str(), "https://example.com/a/c");

        let u2 = normalize_url(&o, "../c").unwrap();
        assert_eq!(u2.as_str(), "https://example.com/c");
    }

    #[test]
    fn domain_without_scheme_is_treated_as_new_host_using_origin_scheme() {
        let o = origin_https();
        let u = normalize_url(&o, "help.com").unwrap();
        assert_eq!(u.as_str(), "https://help.com/");

        let u2 = normalize_url(&o, "help.com/path").unwrap();
        assert_eq!(u2.as_str(), "https://help.com/path");
    }

    #[test]
    fn domainish_does_not_overwrite_with_origin_domain() {
        let o = Url::parse("https://example.com/a/b").unwrap();
        let u = normalize_url(&o, "help.com").unwrap();
        assert_eq!(u.host_str(), Some("help.com"));
        assert_eq!(u.as_str(), "https://help.com/");
    }

    #[test]
    fn ipv4_with_port_is_domainish() {
        let o = origin_https();
        let u = normalize_url(&o, "127.0.0.1:8000/x").unwrap();
        assert_eq!(u.as_str(), "https://127.0.0.1:8000/x");
    }

    #[test]
    fn ipv6_literal_is_domainish() {
        let o = origin_https();
        let u = normalize_url(&o, "[2001:db8::1]/x").unwrap();
        assert_eq!(u.as_str(), "https://[2001:db8::1]/x");
    }

    #[test]
    fn punycode_host_normalizes_when_domainish() {
        let o = origin_https();
        let u = normalize_url(&o, "bücher.example/x").unwrap();
        assert_eq!(u.as_str(), "https://xn--bcher-kva.example/x");
    }

    #[test]
    fn normalize_preserves_rfc_file_vs_directory_semantics() {
        use url::Url;

        let origin_file = Url::parse("http://example.com/test").unwrap();
        let origin_dir = Url::parse("http://example.com/test/").unwrap();

        let u1 = normalize_url(&origin_file, "hello").unwrap();
        assert_eq!(u1.as_str(), "http://example.com/hello");

        let u2 = normalize_url(&origin_dir, "hello").unwrap();
        assert_eq!(u2.as_str(), "http://example.com/test/hello");
    }

    #[test]
    fn looks_like_domainish_basics() {
        assert!(looks_like_domainish("example.com"));
        assert!(looks_like_domainish("example.com/test"));
        assert!(looks_like_domainish("127.0.0.1:8000/x"));
        assert!(looks_like_domainish("[2001:db8::1]/x"));

        assert!(!looks_like_domainish("/test"));
        assert!(!looks_like_domainish("./test"));
        assert!(!looks_like_domainish("../test"));
        assert!(!looks_like_domainish("?q=1"));
        assert!(!looks_like_domainish("#frag"));
        assert!(!looks_like_domainish("has space.com"));
    }

    #[test]
    fn parse_error_propagates_for_unparseable_domainish() {
        let o = origin_https();
        normalize_url(&o, "http://bad host.com").unwrap_err();
    }
}
