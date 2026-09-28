use crate::models::{DownloadResult, MediaItem, MediaType, PostInfo};
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct Downloader {
    client: reqwest::Client,
}

impl Downloader {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    pub fn get_default_download_directory() -> PathBuf {
        dirs::download_dir().unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn build_filename(shortcode: &str, index: usize, total: usize, media_type: &MediaType) -> String {
        let extension = match media_type {
            MediaType::Video => "mp4",
            MediaType::Image => "jpg",
        };

        if total <= 1 {
            format!("insta_{}.{}", shortcode, extension)
        } else {
            format!("insta_{}_{:02}.{}", shortcode, index, extension)
        }
    }

    pub async fn stream_to_file(&self, url: &str, destination: &Path) -> Result<u64, String> {
        let response = self
            .client
            .get(url)
            .header(
                reqwest::header::USER_AGENT,
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
            )
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Download failed with status: {}", response.status()));
        }

        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create download folder: {}", e))?;
        }

        let mut file = File::create(destination)
            .await
            .map_err(|e| format!("Failed to create destination file: {}", e))?;

        let mut stream = response.bytes_stream();
        let mut total_bytes: u64 = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Error while reading stream: {}", e))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Error while writing to file: {}", e))?;
            total_bytes += chunk.len() as u64;
        }

        file.flush()
            .await
            .map_err(|e| format!("Failed to flush file: {}", e))?;

        Ok(total_bytes)
    }

    pub async fn download_single(
        &self,
        item: &MediaItem,
        shortcode: &str,
        index: usize,
        total: usize,
        target_dir: &Path,
    ) -> DownloadResult {
        let filename = Self::build_filename(shortcode, index, total, &item.media_type);
        let destination = target_dir.join(&filename);

        match self.stream_to_file(&item.url, &destination).await {
            Ok(_) => DownloadResult {
                filename,
                path: destination.to_string_lossy().to_string(),
                success: true,
                error: None,
            },
            Err(e) => DownloadResult {
                filename,
                path: destination.to_string_lossy().to_string(),
                success: false,
                error: Some(e),
            },
        }
    }

    pub async fn download_post(&self, post: &PostInfo, custom_dir: Option<&Path>) -> Vec<DownloadResult> {
        let target_dir = match custom_dir {
            Some(dir) => dir.to_path_buf(),
            None => Self::get_default_download_directory(),
        };

        let total = post.items.len();
        let mut results = Vec::new();

        for (index, item) in post.items.iter().enumerate() {
            let result = self
                .download_single(item, &post.shortcode, index + 1, total, &target_dir)
                .await;
            results.push(result);
        }

        results
    }
}
