use crate::domain::{TlsSummary, UdocError};
use super::io::BoxedIoStream;

pub struct TlsSession {
    pub stream: BoxedIoStream,
    pub tls_ms: f64,
    pub summary: TlsSummary,
    pub peer_certs: Vec<Vec<u8>>,
}

pub trait TlsHandshaker: Send + Sync {
    fn handshake(&self, stream: BoxedIoStream, host: &str) -> impl std::future::Future<Output = Result<TlsSession, UdocError>> + Send;
}
