use std::{collections::HashMap, sync::Arc};

use pingora::{server::ShutdownWatch, services::background::BackgroundService};
use tokio::{
    sync::{mpsc::Receiver, oneshot, RwLock},
    time::Instant,
};
use tracing::*;

#[derive(Debug)]
pub(super) struct ServiceStarter {
    pub(super) services_starter: RwLock<Receiver<(String, oneshot::Sender<String>)>>,
    pub(super) services_state: Arc<RwLock<HashMap<String, Instant>>>,
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
                // TODO: get info + backend in DB

                {
                    let services = self.services_state.read().await;
                    // TODO: check service status
                    if services.contains_key(&required_service) {
                        debug!("service {required_service} already started");
                    } else {
                        debug!("starting service {required_service}");
                    }
                    // TODO: start service
                }
                self.services_state.write().await.insert(required_service.clone(), Instant::now());

                // this should never panic because we just inserted the sender
                started.send(format!("{required_service}:80")).unwrap();
            }
        }

        info!("service starter shutting down");
    }
}
