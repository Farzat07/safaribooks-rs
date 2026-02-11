use crate::cookies::CookieStore;
use anyhow::Result;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, COOKIE, REFERER, USER_AGENT,
};
use reqwest::Client;

/// Minimal HTTP client wrapper.
/// - Cookies are injected into the default `Cookie:` header.
/// - A few "browser-like" headers are pre-set (matching the spirit of the Python script).
pub struct HttpClient {
    client: Client,
    /// Kept for tests and internal checks; **do not log** this in production logs.
    cookie_header: String,
}

impl HttpClient {
    /// Build a HeaderMap with static browser-like values and an explicit Cookie header.
    fn build_default_headers(cookie_header: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // User-Agent: a modern desktop UA string (no device-specific flags).
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            ),
        );

        // Accept: prefer HTML, XML; also allow images and generic types.
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
            ),
        );

        // Accept-Encoding: Inform the server we can accept gzip/deflate.
        // (reqwest handles decompression automatically.)
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate"));

        // Referer: mirrors the original script's "login entry" intent (safe placeholder for now).
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://learning.oreilly.com/login/unified/?next=/home/"),
        );

        // Cookie: **all authentication lives here** (cookies-only flow).
        // IMPORTANT: HeaderValue::from_str validates and rejects invalid bytes.
        headers.insert(COOKIE, HeaderValue::from_str(cookie_header)?);

        Ok(headers)
    }

    /// Create an HttpClient from a CookieStore (preferred path).
    pub fn from_store(store: &CookieStore) -> Result<Self> {
        let cookie_header = store.to_header_value();
        Self::new(&cookie_header)
    }

    /// Create an HttpClient from a pre-rendered "Cookie: ..." value.
    pub fn new(cookie_header: &str) -> Result<Self> {
        let headers = Self::build_default_headers(cookie_header)?;
        let client = Client::builder().default_headers(headers).build()?;
        Ok(Self {
            client,
            cookie_header: cookie_header.to_string(),
        })
    }

    /// Access the underlying reqwest client (read-only).
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Expose the cookie header for tests/diagnostics (do **not** log this in production).
    pub fn cookie_header(&self) -> &str {
        &self.cookie_header
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cookies::CookieStore;
    use serde_json::json;

    #[test]
    fn builds_client_with_cookie_header_from_map() {
        let v = json!({ "sess": "abc", "OptanonConsent": "xyz" });
        let store = CookieStore::from_value(v).unwrap();
        let hc = HttpClient::from_store(&store).unwrap();

        // Deterministic order (sorted by name)
        assert_eq!(hc.cookie_header(), "OptanonConsent=xyz; sess=abc");
        // We don't assert on internal reqwest headers here; the presence of the header value suffices.
    }

    #[test]
    fn builds_client_with_cookie_header_from_list() {
        let v = json!([
            {"name": "a", "value": "1"},
            {"name": "b", "value": "2"}
        ]);
        let store = CookieStore::from_value(v).unwrap();
        let hc = HttpClient::from_store(&store).unwrap();

        assert_eq!(hc.cookie_header(), "a=1; b=2");
    }
}
