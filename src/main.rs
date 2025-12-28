use std::process::ExitCode;
use udoc::application::{GenerateReportUseCase, Config};
use udoc::infrastructure::{HickoryDnsResolver, HybridHttpClient, PrettyRenderer, JsonRenderer, RustlsTlsHandshaker, TokioClock, TokioTcpDialer};
use udoc::ports::Renderer;

fn main() -> ExitCode {
    rustls::crypto::ring::default_provider().install_default().ok();

    let args: Vec<String> = std::env::args().collect();

    let (url, json_mode) = match parse_args(&args) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{}", msg);
            return ExitCode::from(2);
        }
    };

    let config = Config::from_env().with_json(json_mode);

    let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error[ERROR]: failed to create runtime: {}", e);
            return ExitCode::from(1);
        }
    };

    rt.block_on(async_main(&url, config))
}

fn parse_args(args: &[String]) -> Result<(String, bool), String> {
    let mut url = None;
    let mut json = false;

    for arg in args.iter().skip(1) {
        if arg == "--json" || arg == "-j" {
            json = true;
        } else if arg == "--help" || arg == "-h" {
            return Err(usage());
        } else if arg.starts_with('-') {
            return Err(format!("unknown option: {}\n\n{}", arg, usage()));
        } else if url.is_none() {
            url = Some(arg.clone());
        } else {
            return Err(format!("unexpected argument: {}\n\n{}", arg, usage()));
        }
    }

    match url {
        Some(u) => Ok((u, json)),
        None => Err(usage()),
    }
}

fn usage() -> String {
    "usage: udoc [--json] <URL>\n\n\
    Prints connection report: DNS/TCP/TLS/TTFB timings + cert summary.\n\n\
    Options:\n  \
      --json, -j    Output as JSON\n\n\
    Environment:\n  \
      UDOC_TIMEOUT     Request timeout (e.g. 5s, 3000ms) [default: 5s]\n  \
      UDOC_MAX_REDIRS  Max redirects [default: 10]\n  \
      UDOC_BODY_LIMIT  Body preview limit in bytes [default: 32768]\n  \
      UDOC_REPEAT      Repeat count for stats [default: 1]".to_string()
}

async fn async_main(url: &str, config: Config) -> ExitCode {
    let dns = match HickoryDnsResolver::new() {
        Ok(d) => d,
        Err(e) => { eprintln!("{}", e); return ExitCode::from(e.class.exit_code() as u8); }
    };

    let tls = match RustlsTlsHandshaker::new() {
        Ok(t) => t,
        Err(e) => { eprintln!("{}", e); return ExitCode::from(e.class.exit_code() as u8); }
    };

    let json_output = config.json_output;
    let use_case = GenerateReportUseCase::new(dns, TokioTcpDialer::new(), tls, HybridHttpClient::new(), TokioClock::new(), config);

    match use_case.execute(url).await {
        Ok(report) => {
            if json_output {
                print!("{}", JsonRenderer::new().render(&report));
            } else {
                print!("{}", PrettyRenderer::new().render(&report));
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::from(e.class.exit_code() as u8)
        }
    }
}
