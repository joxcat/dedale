use std::{collections::HashMap, sync::Arc, time::Duration};

use pingora::{server::ShutdownWatch, services::background::BackgroundService};
use tokio::{
    sync::RwLock,
    time::{sleep, Instant},
};
use tracing::*;

#[derive(Debug)]
pub(crate) struct ServiceStopper {
    pub(crate) services_state: Arc<RwLock<HashMap<String, Instant>>>,
}
#[async_trait::async_trait]
impl BackgroundService for ServiceStopper {
    async fn start(&self, shutdown: ShutdownWatch) {
        info!("service stopper starting");

        while !*shutdown.borrow() {
            self.services_state
                .write()
                .await
                .retain(|service, instant| {
                    let now = Instant::now();
                    if now - *instant > Duration::from_secs(30) {
                        debug!("stopping service {service}");
                        false
                    } else {
                        true
                    }
                });
            sleep(Duration::from_secs(10)).await;
        }

        info!("service stopper shutting down");
    }
}
