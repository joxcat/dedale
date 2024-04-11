mod docker;
pub(super) use docker::*;
use tracing::warn;

pub(super) trait ProxyServiceBackend {
    const IDENT: &'static str;

    /// Should return true if the service is running
    /// by default it always returns false
    async fn status(service: &str) -> pingora::Result<bool> {
        warn!("get status of {service} in {} status is not implemented", Self::IDENT);
        // Will always return false
        Ok(false)
    }
    /// Should start the service
    /// must be callable multiple times without error
    async fn start(service: &str) -> pingora::Result<String> {
        todo!("start {service} of {} is not implemented", Self::IDENT);
    }
    /// Should stop the service
    /// must be callable multiple times without error
    async fn stop(service: &str) -> pingora::Result<()> {
        todo!("stop {service} of {} is not implemented", Self::IDENT);
    }
}