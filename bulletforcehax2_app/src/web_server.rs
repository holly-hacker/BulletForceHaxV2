use std::{convert::Infallible, net::SocketAddr};

use hyper::{Body, Request, Response, Server};
use tower::{make::Shared, steer::Steer, util::BoxCloneService};
use tracing::{debug, error};

type WebService = BoxCloneService<Request<Body>, Response<Body>, Infallible>;

pub struct WebServer {
    port: u16,
    services: Vec<(&'static str, WebService)>,
    fallback_service: WebService,
}

impl WebServer {
    pub fn new(
        port: u16,
        services: Vec<(&'static str, WebService)>,
        fallback_service: WebService,
    ) -> Self {
        Self {
            port,
            services,
            fallback_service,
        }
    }

    pub fn start_server(self) {
        tokio::spawn(async move { self.block_on_server().await });
    }

    async fn block_on_server(self) {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

        let mut services = self
            .services
            .iter()
            .map(|s| s.1.clone())
            .collect::<Vec<WebService>>();
        services.push(self.fallback_service.clone());

        let paths = self.services.into_iter().map(|s| s.0).collect::<Vec<_>>();
        let service = Steer::new(services, move |req: &Request<Body>, _: &[_]| {
            let path = req.uri().path();
            for (i, prefix) in paths.iter().enumerate() {
                if path.starts_with(prefix) {
                    return i;
                }
            }
            paths.len()
        });

        let server = Server::bind(&addr).serve(Shared::new(service));

        debug!("http server created");
        if let Err(e) = server.await {
            error!("server error: {}", e);
        }
    }
}
