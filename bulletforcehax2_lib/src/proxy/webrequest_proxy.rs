use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use futures_util::lock::Mutex;
use hyper::body::to_bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Client, Request, Response};
use tower::util::BoxCloneService;
use tower::{Service, ServiceBuilder, ServiceExt};
use tower_http::cors::CorsLayer;
use tower_http::decompression::DecompressionLayer;
use tracing::{debug, error, trace, warn};

use crate::hax::HaxState;

pub fn create_service(
    shared_state: Arc<Mutex<HaxState>>,
) -> BoxCloneService<Request<Body>, Response<Body>, Infallible> {
    let service = ServiceBuilder::new()
        .layer(CorsLayer::permissive())
        .service_fn(move |req| web_request_proxy_service(req, shared_state.clone()));

    BoxCloneService::new(service)
}

#[tracing::instrument(name = "WebRequestProxy", level = "info", skip_all, fields(uri = req.uri().query().unwrap_or("")))]
async fn web_request_proxy_service(
    req: Request<Body>,
    state: Arc<Mutex<HaxState>>,
) -> Result<Response<Body>, Infallible> {
    match web_request_proxy(req, state).await {
        Ok(r) => Ok(r),
        Err(e) => {
            error!("Error result while handling proxied request {e:?}");
            Ok(Response::builder()
                .status(500)
                .body(format!("Error while handling request: {e:?}").into())
                .expect("should be able to create basic response"))
        }
    }
}

async fn web_request_proxy(
    req: Request<Body>,
    state: Arc<Mutex<HaxState>>,
) -> anyhow::Result<Response<Body>> {
    debug!("Received {} request", req.method());
    let (parts_req, body) = req.into_parts();

    let proxied_uri = match parts_req.uri.query() {
        Some(x) => x,
        None => {
            warn!("Received request did not have query string");
            return Ok(Response::new("No uri found in query string".into()));
        }
    };

    let proxied_uri = hyper::Uri::from_str(proxied_uri)?;

    if proxied_uri.host() == Some("pref-events.cloud.unity3d.com")
        || proxied_uri.host() == Some("cdp.cloud.unity3d.com")
    {
        debug!("Preventing unity logging");
        return Ok(Response::builder().body("Unity event blocked".into())?);
    }

    let mut body_bytes = to_bytes(body).await?.to_vec();
    if matches!(parts_req.method.as_str(), "GET" | "POST") {
        let request_hook_res =
            HaxState::webrequest_hook_onrequest(state.clone(), &proxied_uri, &mut body_bytes);
        if let Err(error) = request_hook_res {
            error!("Error during request hook: {error}");
        }
    }

    let mut builder = Request::builder()
        .method(&parts_req.method)
        .uri(&proxied_uri)
        .version(parts_req.version);

    if let Some(content_type) = parts_req.headers.get(CONTENT_TYPE) {
        builder = builder.header(CONTENT_TYPE, content_type);
    }

    let req = builder.body(body_bytes.into())?;

    let client = Client::builder().build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());
    let mut client = ServiceBuilder::new()
        .layer(DecompressionLayer::new())
        .service(client);

    let response_result = client.ready().await?.call(req).await;

    match response_result {
        Ok(response) => {
            let (parts_resp, body) = response.into_parts();
            // TODO: want to handle HEAD cleaner, this feels like a hack
            let body_bytes = if matches!(parts_req.method.as_str(), "GET" | "POST" | "PUT") {
                let mut body_bytes = to_bytes(body)
                    .await
                    .map_err(|e| anyhow::anyhow!("error reading proxied response body: {e:?}"))?
                    .to_vec();
                trace!(
                    "got response from remote with body size {}",
                    body_bytes.len()
                );

                let request_hook_res = HaxState::webrequest_hook_onresponse(
                    state.clone(),
                    &proxied_uri,
                    &mut body_bytes,
                );
                if let Err(error) = request_hook_res {
                    error!("Error during request hook: {error}");
                }

                body_bytes
            } else {
                vec![]
            };

            let mut response = Response::builder();

            if let Some(content_type) = parts_resp.headers.get(CONTENT_TYPE) {
                response = response.header(CONTENT_TYPE, content_type);
            }

            Ok(response.body(body_bytes.into())?)
        }
        Err(error) => {
            error!("Error during proxied HTTP request: {error}");
            Ok(Response::builder()
                .status(500)
                .body(format!("Error: {error}").into())?)
        }
    }
}
