use std::collections::HashSet;
use std::time::Instant;
use crate::domain::*;
use crate::ports::*;
use super::{ParsedUrl, Config, parse_certificate};

pub struct GenerateReportUseCase<D, T, L, H, C>
where
    D: DnsResolver,
    T: TcpDialer,
    L: TlsHandshaker,
    H: HttpClient,
    C: Clock,
{
    dns: D,
    tcp: T,
    tls: L,
    http: H,
    clock: C,
    config: Config,
}

impl<D, T, L, H, C> GenerateReportUseCase<D, T, L, H, C>
where
    D: DnsResolver,
    T: TcpDialer,
    L: TlsHandshaker,
    H: HttpClient,
    C: Clock,
{
    pub fn new(dns: D, tcp: T, tls: L, http: H, clock: C, config: Config) -> Self {
        Self { dns, tcp, tls, http, clock, config }
    }

    pub async fn execute(&self, input_url: &str) -> Result<Report, UdocError> {
        let start = self.clock.now();
        let mut current_url = ParsedUrl::parse(input_url)?;
        let mut redirects: Vec<RedirectHop> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut hop_timings: Vec<HopTiming> = Vec::new();

        let mut total_dns_ms = 0.0;
        let mut total_tcp_ms = 0.0;
        let mut total_tls_ms: Option<f64> = None;
        let mut final_ttfb_ms = 0.0;

        let mut final_http: Option<HttpSummary> = None;
        let mut final_tls: Option<TlsSummary> = None;
        let mut final_cert: Option<CertSummary> = None;
        let mut final_resolved: Option<ResolvedTarget> = None;
        let mut was_downgrade = false;

        for hop_idx in 0..=self.config.max_redirects {
            if visited.contains(&current_url.full) {
                return Err(UdocError::http("redirect loop detected"));
            }
            visited.insert(current_url.full.clone());

            let mut hop = HopTiming::default();

            let (ips, dns_ms) = self.clock.timeout(self.config.timeout, self.dns.resolve(&current_url.host)).await??;
            hop.dns_ms = dns_ms;
            total_dns_ms += dns_ms;

            let ip = ips.first().copied().ok_or_else(|| UdocError::dns(format!("no IP addresses for {}", current_url.host)))?;
            let resolved = ResolvedTarget::new(ip, current_url.port, ips);
            final_resolved = Some(resolved.clone());

            let tcp_conn = self.clock.timeout(self.config.timeout, self.tcp.connect(ip, current_url.port)).await??;
            hop.tcp_ms = tcp_conn.tcp_ms;
            total_tcp_ms += tcp_conn.tcp_ms;

            let (response, tls_summary, cert_summary) = if current_url.is_https() {
                let tls_session = self.clock.timeout(self.config.timeout, self.tls.handshake(tcp_conn.stream, &current_url.host)).await??;
                hop.tls_ms = Some(tls_session.tls_ms);
                total_tls_ms = Some(total_tls_ms.unwrap_or(0.0) + tls_session.tls_ms);

                let cert = tls_session.peer_certs.first().map(|der| parse_certificate(der)).transpose()?;
                let summary = tls_session.summary.clone();

                let resp = if summary.is_h2() {
                    self.clock.timeout(self.config.timeout, self.http.request_h2(
                        tls_session.stream, "GET", &current_url.host, current_url.port, &current_url.path_and_query, self.config.body_limit
                    )).await??
                } else {
                    self.clock.timeout(self.config.timeout, self.http.request_h1(
                        tls_session.stream, "GET", &current_url.host, current_url.port, &current_url.path_and_query, true, self.config.body_limit
                    )).await??
                };

                (resp, Some(summary), cert)
            } else {
                let resp = self.clock.timeout(self.config.timeout, self.http.request_h1(
                    tcp_conn.stream, "GET", &current_url.host, current_url.port, &current_url.path_and_query, false, self.config.body_limit
                )).await??;
                (resp, None, None)
            };

            hop.ttfb_ms = response.ttfb_ms;
            final_ttfb_ms = response.ttfb_ms;
            hop_timings.push(hop);

            let status = response.summary.status;

            if is_redirect(status) {
                let location = response.headers.location.as_ref()
                    .ok_or_else(|| UdocError::http(format!("redirect {} without Location header", status)))?;

                let prev_https = current_url.is_https();
                redirects.push(RedirectHop::new(status, current_url.full.clone(), location.clone()));
                current_url = current_url.resolve_redirect(location)?;

                if prev_https && !current_url.is_https() {
                    was_downgrade = true;
                }

                if !matches!(current_url.scheme.as_str(), "http" | "https") {
                    return Err(UdocError::http(format!("redirect to unsupported scheme: {}", current_url.scheme)));
                }

                if hop_idx == self.config.max_redirects {
                    return Err(UdocError::http(format!("too many redirects (max {})", self.config.max_redirects)));
                }

                final_tls = tls_summary.or(final_tls);
                final_cert = cert_summary.or(final_cert);
                continue;
            }

            final_http = Some(response.summary);
            final_tls = tls_summary.or(final_tls);
            final_cert = cert_summary.or(final_cert);
            break;
        }

        let total_ms = elapsed_ms(start, self.clock.now());
        let final_resolved = final_resolved.ok_or_else(|| UdocError::other("no connection established"))?;
        let final_http = final_http.ok_or_else(|| UdocError::other("no HTTP response"))?;

        let timings = TimingBreakdown::new(total_dns_ms, total_tcp_ms, total_tls_ms, final_ttfb_ms, total_ms)
            .with_hops(hop_timings);

        Ok(Report {
            input_url: input_url.to_string(),
            final_url: current_url.full,
            host: current_url.host,
            resolved: final_resolved,
            redirects,
            timings,
            http: final_http,
            tls: final_tls,
            cert: final_cert,
            was_downgrade,
        })
    }
}

fn is_redirect(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

fn elapsed_ms(start: Instant, end: Instant) -> f64 {
    end.duration_since(start).as_secs_f64() * 1000.0
}
