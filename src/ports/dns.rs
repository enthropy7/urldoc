use std::net::IpAddr;
use crate::domain::UdocError;

pub trait DnsResolver: Send + Sync {
    fn resolve(&self, host: &str) -> impl std::future::Future<Output = Result<(Vec<IpAddr>, f64), UdocError>> + Send;
}
