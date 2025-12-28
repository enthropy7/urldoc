use std::time::Duration;

pub struct Config {
    pub timeout: Duration,
    pub max_redirects: usize,
    pub body_limit: usize,
    pub repeat: usize,
    pub json_output: bool,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            timeout: parse_duration_env("UDOC_TIMEOUT", Duration::from_secs(5)),
            max_redirects: parse_usize_env("UDOC_MAX_REDIRS", 10),
            body_limit: parse_usize_env("UDOC_BODY_LIMIT", 32 * 1024),
            repeat: parse_usize_env("UDOC_REPEAT", 1),
            json_output: false,
        }
    }

    pub fn with_json(mut self, json: bool) -> Self {
        self.json_output = json;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}

fn parse_duration_env(key: &str, default: Duration) -> Duration {
    std::env::var(key).ok().and_then(|v| {
        let v = v.trim();
        if let Some(s) = v.strip_suffix("ms") {
            s.parse::<u64>().ok().map(Duration::from_millis)
        } else if let Some(s) = v.strip_suffix('s') {
            s.parse::<u64>().ok().map(Duration::from_secs)
        } else {
            v.parse::<u64>().ok().map(Duration::from_secs)
        }
    }).unwrap_or(default)
}

fn parse_usize_env(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.trim().parse().ok()).unwrap_or(default)
}

