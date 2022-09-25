use anyhow::Result;
use hyper::body::to_bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Client, Request, Response, Server};
use std::net::SocketAddr;
use std::str::FromStr;
use tower::make::Shared;
use tower::{Service, ServiceBuilder, ServiceExt};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::decompression::DecompressionLayer;

pub async fn block_on_server() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 48897));

    let service = ServiceBuilder::new()
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .service_fn(hello_world_service);

    let server = Server::bind(&addr).serve(Shared::new(service));

    println!("http server created");
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn hello_world_service(
    req: Request<Body>,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    println!("Received web request {} {}", req.method(), req.uri());
    let (parts_req, body) = req.into_parts();

    let proxied_uri = match parts_req.uri.query() {
        Some(x) => x,
        None => {
            println!("Received request did not have query string");
            return Ok(Response::new("No uri found in query string".into()));
        }
    };

    let proxied_uri = hyper::Uri::from_str(proxied_uri)?;

    if proxied_uri.host() == Some("pref-events.cloud.unity3d.com")
        || proxied_uri.host() == Some("cdp.cloud.unity3d.com")
    {
        println!("Preventing unity logging");
        return Ok(Response::builder().body("Unity event blocked".into())?);
    }

    let mut body_bytes = to_bytes(body).await?.to_vec();
    if matches!(parts_req.method.as_str(), "GET" | "POST") {
        let request_hook_res = request_hook(&proxied_uri, &mut body_bytes);
        if let Err(error) = request_hook_res {
            println!("Error during request hook: {error}");
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
            let mut body_bytes = to_bytes(body).await?.to_vec();

            if matches!(parts_req.method.as_str(), "GET" | "POST") {
                let request_hook_res = response_hook(&proxied_uri, &mut body_bytes);
                if let Err(error) = request_hook_res {
                    println!("Error during request hook: {error}");
                }
            }

            let mut response = Response::builder();

            if let Some(content_type) = parts_resp.headers.get(CONTENT_TYPE) {
                response = response.header(CONTENT_TYPE, content_type);
            }

            Ok(response.body(body_bytes.into())?)
        }
        Err(error) => {
            println!("Error during proxied HTTP request: {error}");
            Ok(Response::builder()
                .status(500)
                .body(format!("Error: {error}").into())?)
        }
    }
}

fn request_hook(_url: &hyper::Uri, _bytes: &mut Vec<u8>) -> Result<()> {
    Ok(())
}

fn response_hook(url: &hyper::Uri, bytes: &mut Vec<u8>) -> Result<()> {
    match url.path() {
        "/OnlineAccountSystem/get-promotional-multipliers.php" => {
            *bytes = r#"{"credsMult":2,"xpMult":2}"#.into();
            println!("Rewrote promotional multiplier");
        }
        _ => (),
    }
    Ok(())
}
