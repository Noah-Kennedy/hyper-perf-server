use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::header::CACHE_CONTROL;
use axum::http::{HeaderMap, StatusCode, Version};
use axum::routing::{get, post};
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use futures_util::StreamExt;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    cert: PathBuf,
    #[arg(short, long)]
    key: PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();

    let config = RustlsConfig::from_pem_file(args.cert, args.key)
        .await
        .unwrap();

    let app = Router::new()
        .route("/", get(hello).post(upload_void))
        .route("/void", post(upload_void));

    // run https server
    let addr = SocketAddr::from(([0, 0, 0, 0], 443));

    tracing::info!("listening on {}", addr);

    tokio::spawn(
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>()),
    )
    .await
    .unwrap()
    .unwrap();
}

// feel free to scream into the void
async fn upload_void(
    connect_info: ConnectInfo<SocketAddr>,
    version: Version,
    headers: HeaderMap,
    body: Body,
) -> StatusCode {
    let mut stream = body.into_data_stream();

    let span = tracing::info_span!(
        "void",
        headers = ?headers,
        version = ?version,
        addr = ?connect_info.0,
    );

    let _guard = span.enter();

    tracing::info!(message = "Started void upload");

    let mut len = 0;

    while let Some(frame) = stream.next().await {
        match frame {
            Err(err) => {
                tracing::error!("failed to read frame: {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            Ok(f) => {
                len += f.len();

                tracing::info!(len = len, message = "Void new frame");
            }
        }
    }

    tracing::info!(len = len, message = "Finished void upload");

    StatusCode::OK
}

#[axum::debug_handler]
async fn hello(
    connect_info: ConnectInfo<SocketAddr>,
    version: Version,
    headers: HeaderMap,
) -> (HeaderMap, &'static str) {
    tracing::info!(
        headers = ?headers,
        version = ?version,
        addr = ?connect_info.0,
        message="hello"
    );
    let mut headers = HeaderMap::new();

    headers.insert("Content-Type", "text/plain".parse().unwrap());
    headers.insert(CACHE_CONTROL, "no-cache".parse().unwrap());

    (headers, "Hello, World!")
}
