use std::net::IpAddr;
use crate::domain::UdocError;
use super::io::BoxedIoStream;

pub struct TcpConnection {
    pub stream: BoxedIoStream,
    pub tcp_ms: f64,
}

pub trait TcpDialer: Send + Sync {
    fn connect(&self, ip: IpAddr, port: u16) -> impl std::future::Future<Output = Result<TcpConnection, UdocError>> + Send;
}
