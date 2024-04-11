use std::{collections::HashMap, sync::Arc};

use pingora::{server::ShutdownWatch, services::background::BackgroundService};
use tokio::{
    sync::{mpsc::Receiver, oneshot, RwLock},
    time::Instant,
};
use tracing::*;

#[derive(Debug)]
pub(crate) struct ServiceStarter {
    pub(crate) services_starter: RwLock<Receiver<(String, oneshot::Sender<String>)>>,
    pub(crate) services_state: Arc<RwLock<HashMap<String, Instant>>>,
}

#[async_trait::async_trait]
impl BackgroundService for ServiceStarter {
    async fn start(&self, shutdown: ShutdownWatch) {
        info!("service starter starting");

        while !*shutdown.borrow() {
            while let Some((required_service, started)) =
                self.services_starter.write().await.recv().await
            {
                debug!("got request to start service {required_service}");
                // TODO: Timeout
                // TODO: is managed?

                let mut services = self.services_state.write().await;
                if services.contains_key(&required_service) {
                    debug!("service {required_service} already started");
                } else {
                    debug!("starting service {required_service}");
                }
                services.insert(required_service.clone(), Instant::now());
                drop(services);

                // this should never panic because we just inserted the sender
                started.send(format!("{required_service}:80")).unwrap();
            }
        }

        info!("service starter shutting down");
    }
}
