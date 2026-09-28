use crate::downloader::Downloader;
use crate::extractor::Extractor;
use crate::models::{ApiResponse, DownloadRequest, DownloadResult, FetchRequest, PostInfo};
use axum::{
    extract::{Path as AxumPath, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rust_embed::RustEmbed;
use serde::Serialize;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    pub extractor: Arc<Extractor>,
    pub downloader: Arc<Downloader>,
}

#[derive(Serialize)]
pub struct SystemInfo {
    pub download_dir: String,
}

pub async fn run_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        extractor: Arc::new(Extractor::new()),
        downloader: Arc::new(Downloader::new()),
    };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/assets/*path", get(serve_asset))
        .route("/api/info", get(handle_info))
        .route("/api/fetch", post(handle_fetch))
        .route("/api/download", post(handle_download))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let address = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&address).await?;
    let local_url = format!("http://localhost:{}", port);

    println!("Web UI started on {}", local_url);
    let _ = open::that(&local_url);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn serve_index() -> impl IntoResponse {
    serve_embedded_file("index.html")
}

async fn serve_asset(AxumPath(path): AxumPath<String>) -> impl IntoResponse {
    serve_embedded_file(&path)
}

fn serve_embedded_file(path: &str) -> Response {
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, HeaderValue::from_str(mime.as_ref()).unwrap())],
                content.data.into_owned(),
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

async fn handle_info() -> Json<ApiResponse<SystemInfo>> {
    let download_dir = Downloader::get_default_download_directory()
        .to_string_lossy()
        .to_string();
    Json(ApiResponse::ok(SystemInfo { download_dir }))
}

async fn handle_fetch(
    State(state): State<AppState>,
    Json(payload): Json<FetchRequest>,
) -> Json<ApiResponse<PostInfo>> {
    match state.extractor.fetch_post(&payload.url, payload.cookie.as_deref()).await {
        Ok(post) => Json(ApiResponse::ok(post)),
        Err(err) => Json(ApiResponse::err(err)),
    }
}

async fn handle_download(
    State(state): State<AppState>,
    Json(payload): Json<DownloadRequest>,
) -> Json<ApiResponse<Vec<DownloadResult>>> {
    let dest_dir = Downloader::get_default_download_directory();
    let mut results = Vec::new();

    for item in payload.items {
        let destination = dest_dir.join(&item.filename);
        match state.downloader.stream_to_file(&item.url, &destination).await {
            Ok(_) => results.push(DownloadResult {
                filename: item.filename,
                path: destination.to_string_lossy().to_string(),
                success: true,
                error: None,
            }),
            Err(e) => results.push(DownloadResult {
                filename: item.filename,
                path: destination.to_string_lossy().to_string(),
                success: false,
                error: Some(e),
            }),
        }
    }

    Json(ApiResponse::ok(results))
}
