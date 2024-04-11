use super::ProxyServiceBackend;

pub(super) struct DockerServiceBackend;

impl ProxyServiceBackend for DockerServiceBackend {
    const IDENT: &'static str = "docker";
}