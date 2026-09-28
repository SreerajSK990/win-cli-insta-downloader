use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: String,
    pub media_type: MediaType,
    pub url: String,
    pub thumbnail_url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInfo {
    pub shortcode: String,
    pub caption: Option<String>,
    pub owner_username: Option<String>,
    pub owner_full_name: Option<String>,
    pub items: Vec<MediaItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FetchRequest {
    pub url: String,
    pub cookie: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DownloadItemRequest {
    pub url: String,
    pub filename: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DownloadRequest {
    pub items: Vec<DownloadItemRequest>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadResult {
    pub filename: String,
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}
