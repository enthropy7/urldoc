use crate::domain::{HttpSummary, UdocError};
use super::io::BoxedIoStream;

#[derive(Debug, Clone, Default)]
pub struct ResponseHeaders {
    pub location: Option<String>,
    pub server: Option<String>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub transfer_encoding: Option<String>,
}

pub struct HttpResponse {
    pub summary: HttpSummary,
    pub headers: ResponseHeaders,
    pub ttfb_ms: f64,
    pub body_preview: Vec<u8>,
}

pub trait HttpClient: Send + Sync {
    fn request_h1(&self, stream: BoxedIoStream, method: &str, host: &str, port: u16, path: &str, is_https: bool, body_limit: usize)
        -> impl std::future::Future<Output = Result<HttpResponse, UdocError>> + Send;

    fn request_h2(&self, stream: BoxedIoStream, method: &str, host: &str, port: u16, path: &str, body_limit: usize)
        -> impl std::future::Future<Output = Result<HttpResponse, UdocError>> + Send;
}
