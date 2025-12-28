use std::net::{IpAddr, SocketAddr};
use std::time::Instant;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use crate::domain::UdocError;
use crate::ports::{TcpDialer, TcpConnection, IoStream, BoxedIoStream};

struct TokioTcpStream(TcpStream);

impl IoStream for TokioTcpStream {}

impl AsyncRead for TokioTcpStream {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for TokioTcpStream {
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

pub struct TokioTcpDialer;

impl TokioTcpDialer {
    pub fn new() -> Self { Self }
}

impl TcpDialer for TokioTcpDialer {
    async fn connect(&self, ip: IpAddr, port: u16) -> Result<TcpConnection, UdocError> {
        let start = Instant::now();
        let addr = SocketAddr::new(ip, port);
        let stream = TcpStream::connect(addr).await.map_err(|e| {
            let msg = match e.kind() {
                std::io::ErrorKind::ConnectionRefused => format!("connection refused: {}:{}", ip, port),
                std::io::ErrorKind::TimedOut => format!("connection timed out: {}:{}", ip, port),
                _ => format!("TCP connect failed to {}:{}: {}", ip, port, e),
            };
            UdocError::tcp(msg)
        })?;
        Ok(TcpConnection {
            stream: BoxedIoStream(Box::new(TokioTcpStream(stream))),
            tcp_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }
}
