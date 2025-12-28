use crate::domain::Report;
use crate::ports::Renderer;

pub struct PrettyRenderer;

impl PrettyRenderer {
    pub fn new() -> Self { Self }
}

impl Renderer for PrettyRenderer {
    fn render(&self, report: &Report) -> String {
        let mut out = String::new();

        let tls_ver = report.tls.as_ref().map(|t| t.version.as_str()).unwrap_or("-");
        let proto_ver = &report.http.version;
        out.push_str(&format!(
            "{} {}  {}  ip={}  total={:.1}ms  ttfb={:.1}ms  tls={}  bottleneck={}\n",
            report.http.status,
            report.http.reason.as_deref().unwrap_or(""),
            proto_ver,
            report.resolved.ip,
            report.timings.total_ms,
            report.timings.ttfb_ms,
            tls_ver,
            report.bottleneck()
        ));

        if report.was_downgrade {
            out.push_str("⚠ WARNING: HTTPS→HTTP downgrade detected!\n");
        }

        if let Some(ref cert) = report.cert {
            if cert.days_left < 14 {
                out.push_str(&format!("⚠ CERT EXPIRING in {} days!\n", cert.days_left));
            }
        }

        out.push('\n');
        out.push_str("URL\n");
        out.push_str(&format!("  input:  {}\n", report.input_url));
        out.push_str(&format!("  final:  {}\n", report.final_url));
        out.push_str(&format!("  host:   {}\n", report.host));
        out.push_str(&format!("  ip:     {}   ({})\n", report.resolved.as_socket_str(), report.resolved.family));
        if report.resolved.all_ips.len() > 1 {
            out.push_str(&format!("  ips:    {}\n", report.resolved.ips_short()));
        }

        if !report.redirects.is_empty() {
            out.push('\n');
            out.push_str(&format!("REDIRECTS ({})\n", report.redirects.len()));
            for (i, hop) in report.redirects.iter().enumerate() {
                out.push_str(&format!("  [{}] {} → {}\n", hop.status, shorten_url(&hop.from, 40), shorten_url(&hop.to, 40)));
                if i < report.timings.hops.len() {
                    let ht = &report.timings.hops[i];
                    out.push_str(&format!("      dns={:.1}ms tcp={:.1}ms", ht.dns_ms, ht.tcp_ms));
                    if let Some(tls) = ht.tls_ms { out.push_str(&format!(" tls={:.1}ms", tls)); }
                    out.push_str(&format!(" ttfb={:.1}ms\n", ht.ttfb_ms));
                }
            }
        }

        out.push('\n');
        out.push_str("HTTP\n");
        out.push_str(&format!("  status: {}\n", report.http.status_line()));
        out.push_str(&format!("  proto:  {}\n", report.http.proto));
        out.push_str(&format!("  ver:    {}\n", report.http.version));

        out.push('\n');
        out.push_str("TIMINGS\n");
        out.push_str(&format!("  dns:    {:>8.1} ms\n", report.timings.dns_ms));
        out.push_str(&format!("  tcp:    {:>8.1} ms\n", report.timings.tcp_ms));
        if let Some(tls_ms) = report.timings.tls_ms {
            out.push_str(&format!("  tls:    {:>8.1} ms\n", tls_ms));
        }
        out.push_str(&format!("  ttfb:   {:>8.1} ms\n", report.timings.ttfb_ms));
        out.push_str(&format!("  total:  {:>8.1} ms\n", report.timings.total_ms));

        if let Some(ref tls) = report.tls {
            out.push('\n');
            out.push_str("TLS\n");
            out.push_str(&format!("  version: {}\n", tls.version));
            if let Some(ref alpn) = tls.alpn { out.push_str(&format!("  alpn:    {}\n", alpn)); }
            out.push_str(&format!("  cipher:  {}\n", tls.cipher));
            out.push_str(&format!("  chain:   {} certs\n", tls.chain_len));
            out.push_str(&format!("  verify:  {}\n", if tls.verified { "ok" } else { "FAILED" }));
        }

        if let Some(ref cert) = report.cert {
            out.push('\n');
            out.push_str("CERT\n");
            if let Some(ref cn) = cert.subject_cn { out.push_str(&format!("  subject: CN={}\n", cn)); }
            out.push_str(&format!("  issuer:  {}\n", cert.issuer));
            if !cert.san_short.is_empty() { out.push_str(&format!("  san:     {}\n", cert.san_short)); }
            out.push_str(&format!("  valid:   {}\n", cert.validity_range()));
            out.push_str(&format!("  sha256:  {}\n", cert.short_fingerprint()));
        }

        out
    }
}

fn shorten_url(url: &str, max: usize) -> String {
    if url.len() <= max { url.to_string() } else { format!("{}...", &url[..max.saturating_sub(3)]) }
}

pub struct JsonRenderer;

impl JsonRenderer {
    pub fn new() -> Self { Self }
}

impl Renderer for JsonRenderer {
    fn render(&self, report: &Report) -> String {
        let mut out = String::from("{\n");
        out.push_str(&format!("  \"input_url\": {:?},\n", report.input_url));
        out.push_str(&format!("  \"final_url\": {:?},\n", report.final_url));
        out.push_str(&format!("  \"host\": {:?},\n", report.host));
        out.push_str(&format!("  \"ip\": {:?},\n", report.resolved.ip.to_string()));
        out.push_str(&format!("  \"port\": {},\n", report.resolved.port));
        out.push_str(&format!("  \"status\": {},\n", report.http.status));
        out.push_str(&format!("  \"http_version\": {:?},\n", report.http.version));
        out.push_str(&format!("  \"proto\": {:?},\n", report.http.proto));
        out.push_str(&format!("  \"redirects\": {},\n", report.redirects.len()));
        out.push_str(&format!("  \"was_downgrade\": {},\n", report.was_downgrade));

        out.push_str("  \"timings\": {\n");
        out.push_str(&format!("    \"dns_ms\": {:.2},\n", report.timings.dns_ms));
        out.push_str(&format!("    \"tcp_ms\": {:.2},\n", report.timings.tcp_ms));
        if let Some(tls) = report.timings.tls_ms {
            out.push_str(&format!("    \"tls_ms\": {:.2},\n", tls));
        }
        out.push_str(&format!("    \"ttfb_ms\": {:.2},\n", report.timings.ttfb_ms));
        out.push_str(&format!("    \"total_ms\": {:.2}\n", report.timings.total_ms));
        out.push_str("  },\n");
        out.push_str(&format!("  \"bottleneck\": {:?},\n", report.bottleneck()));

        if let Some(ref tls) = report.tls {
            out.push_str("  \"tls\": {\n");
            out.push_str(&format!("    \"version\": {:?},\n", tls.version));
            match &tls.alpn {
                Some(a) => out.push_str(&format!("    \"alpn\": {:?},\n", a)),
                None => out.push_str("    \"alpn\": null,\n"),
            }
            out.push_str(&format!("    \"cipher\": {:?},\n", tls.cipher));
            out.push_str(&format!("    \"chain_len\": {},\n", tls.chain_len));
            out.push_str(&format!("    \"verified\": {}\n", tls.verified));
            out.push_str("  },\n");
        }

        if let Some(ref cert) = report.cert {
            out.push_str("  \"cert\": {\n");
            match &cert.subject_cn {
                Some(cn) => out.push_str(&format!("    \"subject_cn\": {:?},\n", cn)),
                None => out.push_str("    \"subject_cn\": null,\n"),
            }
            out.push_str(&format!("    \"issuer\": {:?},\n", cert.issuer));
            out.push_str(&format!("    \"san\": {:?},\n", cert.san_short));
            out.push_str(&format!("    \"not_before\": {:?},\n", cert.not_before));
            out.push_str(&format!("    \"not_after\": {:?},\n", cert.not_after));
            out.push_str(&format!("    \"days_left\": {},\n", cert.days_left));
            out.push_str(&format!("    \"sha256\": {:?}\n", cert.sha256_fp));
            out.push_str("  }\n");
        } else {
            out.push_str("  \"cert\": null\n");
        }

        out.push_str("}\n");
        out
    }
}
