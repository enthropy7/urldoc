use std::fmt::Write;
use crate::domain::{CertSummary, UdocError};
use x509_parser::prelude::*;
use chrono::{DateTime, Utc};

pub fn parse_certificate(der: &[u8]) -> Result<CertSummary, UdocError> {
    let (_, cert) = X509Certificate::from_der(der)
        .map_err(|e| UdocError::tls(format!("failed to parse certificate: {}", e)))?;

    let subject_cn = cert.subject().iter_common_name().next()
        .and_then(|cn| cn.as_str().ok())
        .map(|s| s.to_string());

    let issuer = cert.issuer().iter_common_name().next()
        .and_then(|cn| cn.as_str().ok())
        .map(|s| s.to_string())
        .or_else(|| cert.issuer().iter_organization().next().and_then(|o| o.as_str().ok()).map(|s| s.to_string()))
        .unwrap_or_else(|| "Unknown".to_string());

    let san_list: Vec<String> = cert.subject_alternative_name().ok().flatten()
        .map(|san| san.value.general_names.iter()
            .filter_map(|gn| match gn { GeneralName::DNSName(dns) => Some(dns.to_string()), _ => None })
            .collect())
        .unwrap_or_default();

    let san_short = if san_list.is_empty() {
        String::new()
    } else if san_list.len() == 1 {
        san_list[0].clone()
    } else {
        format!("{} (+{})", san_list[0], san_list.len() - 1)
    };

    let not_before = format_asn1_time(cert.validity().not_before);
    let not_after = format_asn1_time(cert.validity().not_after);
    let days_left = compute_days_left(&cert.validity().not_after);
    let sha256_fp = compute_sha256_fingerprint(der);

    Ok(CertSummary { subject_cn, issuer, san_short, not_before, not_after, days_left, sha256_fp })
}

fn format_asn1_time(time: ASN1Time) -> String {
    DateTime::from_timestamp(time.timestamp(), 0)
        .map(|dt: DateTime<Utc>| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn compute_days_left(not_after: &ASN1Time) -> i64 {
    (not_after.timestamp() - Utc::now().timestamp()) / 86400
}

fn compute_sha256_fingerprint(der: &[u8]) -> String {
    let digest = sha256(der);
    let mut result = String::with_capacity(95);
    for (i, byte) in digest.iter().enumerate() {
        if i > 0 { result.push(':'); }
        let _ = write!(&mut result, "{:02x}", byte);
    }
    result
}

fn sha256(data: &[u8]) -> [u8; 32] {
    use ring::digest::{Context, SHA256};
    let mut ctx = Context::new(&SHA256);
    ctx.update(data);
    let d = ctx.finish();
    let mut out = [0u8; 32];
    out.copy_from_slice(d.as_ref());
    out
}
