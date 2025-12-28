use std::sync::Arc;
use std::time::Instant;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use rustls::ClientConfig;
use rustls::pki_types::ServerName;
use crate::domain::{TlsSummary, UdocError};
use crate::ports::{TlsHandshaker, TlsSession, IoStream, BoxedIoStream};

struct RustlsTlsStream<S>(TlsStream<S>);

impl<S: AsyncRead + AsyncWrite + Unpin + Send> IoStream for RustlsTlsStream<S> {}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for RustlsTlsStream<S> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for RustlsTlsStream<S> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

struct IoStreamAdapter(BoxedIoStream);

impl AsyncRead for IoStreamAdapter {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for IoStreamAdapter {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

pub struct RustlsTlsHandshaker {
    connector: TlsConnector,
}

impl RustlsTlsHandshaker {
    pub fn new() -> Result<Self, UdocError> {
        let root_store = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let mut config = ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth();
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Ok(Self { connector: TlsConnector::from(Arc::new(config)) })
    }
}

impl TlsHandshaker for RustlsTlsHandshaker {
    async fn handshake(&self, stream: BoxedIoStream, host: &str) -> Result<TlsSession, UdocError> {
        let start = Instant::now();
        let server_name = ServerName::try_from(host.to_string())
            .map_err(|_| UdocError::tls(format!("invalid server name: {}", host)))?;

        let adapter = IoStreamAdapter(stream);
        let tls_stream = self.connector.connect(server_name, adapter).await
            .map_err(|e| UdocError::tls(format!("TLS handshake failed: {}", e)))?;

        let tls_ms = start.elapsed().as_secs_f64() * 1000.0;
        let (_, conn) = tls_stream.get_ref();

        let version = match conn.protocol_version() {
            Some(rustls::ProtocolVersion::TLSv1_2) => "TLS1.2".to_string(),
            Some(rustls::ProtocolVersion::TLSv1_3) => "TLS1.3".to_string(),
            Some(v) => format!("{:?}", v),
            None => "unknown".to_string(),
        };

        let alpn = conn.alpn_protocol().map(|p| String::from_utf8_lossy(p).to_string());
        let cipher = conn.negotiated_cipher_suite().map(|cs| format!("{:?}", cs.suite())).unwrap_or_else(|| "unknown".to_string());
        let peer_certs: Vec<Vec<u8>> = conn.peer_certificates().map(|certs| certs.iter().map(|c| c.as_ref().to_vec()).collect()).unwrap_or_default();
        let chain_len = peer_certs.len();

        Ok(TlsSession {
            stream: BoxedIoStream(Box::new(RustlsTlsStream(tls_stream))),
            tls_ms,
            summary: TlsSummary::new(version, alpn, cipher, chain_len, true),
            peer_certs,
        })
    }
}
