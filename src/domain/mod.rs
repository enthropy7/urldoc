mod report;
mod timing;
mod http;
mod tls;
mod cert;
mod redirect;
mod target;
mod error;

pub use report::Report;
pub use timing::{TimingBreakdown, HopTiming};
pub use http::HttpSummary;
pub use tls::TlsSummary;
pub use cert::CertSummary;
pub use redirect::RedirectHop;
pub use target::{ResolvedTarget, IpFamily};
pub use error::{ErrorClass, UdocError};
