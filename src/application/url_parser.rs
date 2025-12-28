use url::Url;
use crate::domain::UdocError;

#[derive(Debug, Clone)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path_and_query: String,
    pub full: String,
}

impl ParsedUrl {
    pub fn parse(input: &str) -> Result<Self, UdocError> {
        let url = Url::parse(input).map_err(|e| UdocError::input(format!("invalid URL: {}", e)))?;

        let scheme = url.scheme().to_string();
        if scheme != "http" && scheme != "https" {
            return Err(UdocError::input(format!("unsupported scheme '{}', expected http or https", scheme)));
        }

        let host = url.host_str().ok_or_else(|| UdocError::input("missing host"))?.to_string();
        let port = url.port_or_known_default().unwrap_or(if scheme == "https" { 443 } else { 80 });

        let path = url.path();
        let path_and_query = match url.query() {
            Some(q) => format!("{}?{}", path, q),
            None => path.to_string(),
        };
        let path_and_query = if path_and_query.is_empty() { "/".to_string() } else { path_and_query };

        Ok(Self { scheme, host, port, path_and_query, full: url.to_string() })
    }

    pub fn is_https(&self) -> bool {
        self.scheme == "https"
    }

    pub fn resolve_redirect(&self, location: &str) -> Result<ParsedUrl, UdocError> {
        let base = Url::parse(&self.full).map_err(|e| UdocError::http(format!("invalid base URL: {}", e)))?;
        let resolved = base.join(location).map_err(|e| UdocError::http(format!("invalid redirect location: {}", e)))?;
        ParsedUrl::parse(resolved.as_str())
    }
}
