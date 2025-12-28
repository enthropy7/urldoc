use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    Input,
    Dns,
    Tcp,
    Tls,
    Http,
    Timeout,
    Other,
}

impl ErrorClass {
    pub fn exit_code(&self) -> i32 {
        match self {
            ErrorClass::Input => 2,
            ErrorClass::Dns => 3,
            ErrorClass::Tcp => 4,
            ErrorClass::Tls => 5,
            ErrorClass::Http => 6,
            ErrorClass::Timeout => 7,
            ErrorClass::Other => 1,
        }
    }

    pub fn tag(&self) -> &'static str {
        match self {
            ErrorClass::Input => "INPUT",
            ErrorClass::Dns => "DNS",
            ErrorClass::Tcp => "TCP",
            ErrorClass::Tls => "TLS",
            ErrorClass::Http => "HTTP",
            ErrorClass::Timeout => "TIMEOUT",
            ErrorClass::Other => "ERROR",
        }
    }
}

#[derive(Debug)]
pub struct UdocError {
    pub class: ErrorClass,
    pub message: String,
}

impl UdocError {
    pub fn new(class: ErrorClass, message: impl Into<String>) -> Self {
        Self { class, message: message.into() }
    }

    pub fn input(msg: impl Into<String>) -> Self { Self::new(ErrorClass::Input, msg) }
    pub fn dns(msg: impl Into<String>) -> Self { Self::new(ErrorClass::Dns, msg) }
    pub fn tcp(msg: impl Into<String>) -> Self { Self::new(ErrorClass::Tcp, msg) }
    pub fn tls(msg: impl Into<String>) -> Self { Self::new(ErrorClass::Tls, msg) }
    pub fn http(msg: impl Into<String>) -> Self { Self::new(ErrorClass::Http, msg) }
    pub fn timeout(msg: impl Into<String>) -> Self { Self::new(ErrorClass::Timeout, msg) }
    pub fn other(msg: impl Into<String>) -> Self { Self::new(ErrorClass::Other, msg) }

    pub fn format_stderr(&self) -> String {
        format!("error[{}]: {}", self.class.tag(), self.message)
    }
}

impl fmt::Display for UdocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_stderr())
    }
}

impl std::error::Error for UdocError {}
