use std::net::IpAddr;
use std::time::Instant;
use hickory_resolver::TokioAsyncResolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use crate::domain::UdocError;
use crate::ports::DnsResolver;

pub struct HickoryDnsResolver {
    resolver: TokioAsyncResolver,
}

impl HickoryDnsResolver {
    pub fn new() -> Result<Self, UdocError> {
        Ok(Self { resolver: TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()) })
    }
}


impl DnsResolver for HickoryDnsResolver {
    async fn resolve(&self, host: &str) -> Result<(Vec<IpAddr>, f64), UdocError> {
        let start = Instant::now();
        let response = self.resolver.lookup_ip(host).await
            .map_err(|e| UdocError::dns(format!("DNS lookup failed for '{}': {}", host, e)))?;
        let dns_ms = start.elapsed().as_secs_f64() * 1000.0;
        let ips: Vec<IpAddr> = response.iter().collect();
        if ips.is_empty() {
            return Err(UdocError::dns(format!("no DNS records for '{}'", host)));
        }
        Ok((ips, dns_ms))
    }
}
