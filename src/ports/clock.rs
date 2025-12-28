use std::time::{Duration, Instant};
use crate::domain::UdocError;

pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;

    fn timeout<F, T>(&self, duration: Duration, future: F) -> impl std::future::Future<Output = Result<T, UdocError>> + Send
    where
        F: std::future::Future<Output = T> + Send,
        T: Send;
}
