use axum::body::Body;
use axum::http::header::CACHE_CONTROL;
use axum::http::{HeaderMap, StatusCode};
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

    tokio::spawn(axum_server::bind_rustls(addr, config).serve(app.into_make_service()))
        .await
        .unwrap()
        .unwrap();
}

// feel free to scream into the void
async fn upload_void(body: Body) -> StatusCode {
    let mut stream = body.into_data_stream();

    while let Some(frame) = stream.next().await {
        if let Err(err) = frame {
            tracing::error!("failed to read frame: {}", err);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    StatusCode::OK
}

async fn hello() -> (HeaderMap, &'static str) {
    let mut headers = HeaderMap::new();

    headers.insert("Content-Type", "text/plain".parse().unwrap());
    headers.insert(CACHE_CONTROL, "no-cache".parse().unwrap());

    (headers, "Hello, World!")
}
