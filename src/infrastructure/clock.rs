use std::time::{Duration, Instant};
use crate::domain::UdocError;
use crate::ports::Clock;

pub struct TokioClock;

impl TokioClock {
    pub fn new() -> Self { Self }
}


impl Clock for TokioClock {
    fn now(&self) -> Instant { Instant::now() }

    async fn timeout<F, T>(&self, duration: Duration, future: F) -> Result<T, UdocError>
    where
        F: std::future::Future<Output = T> + Send,
        T: Send,
    {
        tokio::time::timeout(duration, future).await.map_err(|_| UdocError::timeout(format!("operation timed out after {:?}", duration)))
    }
}
