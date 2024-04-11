use std::{
    collections::HashMap,
    net::ToSocketAddrs,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use pingora::{
    self,
    proxy::{http_proxy_service, ProxyHttp, Session},
    server::Server,
    services::background::background_service,
    upstreams::peer::HttpPeer,
    ErrorType::InvalidHTTPHeader,
};
use service_starter::ServiceStarter;
use service_stopper::ServiceStopper;
use tokio::sync::{
    mpsc::{channel, Sender},
    oneshot, RwLock,
};
use tracing::*;

mod service_starter;
mod service_stopper;

#[derive(Debug)]
pub struct Proxy {
    services_starter: Sender<(String, oneshot::Sender<String>)>,
    concurrent_req_count: AtomicU64,
    default_service: Option<String>,
}

impl Proxy {
    pub fn run(listen_url: &str, default_service: Option<&str>) -> pingora::Result<()> {
        let mut server = Server::new(None)?;
        server.bootstrap();

        let (tx_need_service, rx_need_service) = channel(1024);
        let services_state = Arc::new(RwLock::new(HashMap::new()));

        let mut proxy = http_proxy_service(
            &server.configuration,
            Proxy {
                services_starter: tx_need_service,
                concurrent_req_count: AtomicU64::new(0),
                default_service: default_service.map(str::to_string),
            },
        );
        proxy.add_tcp(listen_url);
        info!("listening on {listen_url}");

        let start_service_handler = background_service(
            "service-starter",
            ServiceStarter {
                services_starter: RwLock::new(rx_need_service),
                services_state: services_state.clone(),
            },
        );
        let stop_service_handler =
            background_service("service-stopper", ServiceStopper { services_state });

        server.add_service(proxy);
        server.add_service(start_service_handler);
        server.add_service(stop_service_handler);
        server.run_forever();

        info!("proxy shutting down");

        Ok(())
    }
}

#[derive(Debug)]
pub struct ProxyCtx {
    pub host: Option<String>,
    pub tx_service_started: Option<oneshot::Sender<String>>,
    pub rx_service_started: Option<oneshot::Receiver<String>>,
}

#[async_trait::async_trait]
impl ProxyHttp for Proxy {
    type CTX = ProxyCtx;

    fn new_ctx(&self) -> Self::CTX {
        let (tx_service_started, rx_service_started) = oneshot::channel();
        Self::CTX {
            host: None,
            tx_service_started: Some(tx_service_started),
            rx_service_started: Some(rx_service_started),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool> {
        debug!(
            "got a request {}",
            self.concurrent_req_count.fetch_add(1, Ordering::SeqCst)
        );

        if let Some(host) = session.get_header("host").and_then(|h| {
            h.to_str()
                .ok()
                .and_then(|h| if h.contains(":") { None } else { Some(h) })
        }) {
            let host = host.to_string();
            ctx.host = Some(host.clone());
            // this function will run only once after initialization
            // so we can safely take and unwrap the sender
            self.services_starter
                .send((host, ctx.tx_service_started.take().unwrap()))
                .await
                .unwrap();
        }

        Ok(false)
    }

    #[tracing::instrument(skip_all)]
    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let host = if let Some(ref host) = ctx.host {
            debug!("waiting for {host} to be ready");
            // this function will run only once after initialization
            // so we can safely take and unwrap the receiver
            // TODO: Timeout
            let host = ctx.rx_service_started.take().unwrap().await.unwrap();
            debug!("done waiting for {host} to be ready");

            host
        } else if let Some(ref host) = self.default_service {
            host.clone()
        } else {
            return Err(pingora::Error::explain(
                InvalidHTTPHeader,
                "no host header and no default service",
            ));
        };

        let peer = Box::new(HttpPeer::new(
            host.to_socket_addrs()
                .map(|mut s| s.next())
                .ok()
                .flatten()
                .ok_or_else(|| pingora::Error::explain(InvalidHTTPHeader, "invalid host"))?,
            false,
            host,
        ));

        Ok(peer)
    }

    #[tracing::instrument(skip_all)]
    async fn logging(
        &self,
        _session: &mut Session,
        _e: Option<&pingora::Error>,
        _ctx: &mut Self::CTX,
    ) where
        Self::CTX: Send + Sync,
    {
        debug!(
            "end of request {}",
            self.concurrent_req_count.fetch_sub(1, Ordering::SeqCst)
        );
    }
}
