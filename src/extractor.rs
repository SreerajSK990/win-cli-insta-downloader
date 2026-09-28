use crate::models::{MediaItem, MediaType, PostInfo};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, COOKIE, ORIGIN, REFERER, USER_AGENT};
use serde_json::Value;

pub struct Extractor {
    client: reqwest::Client,
}

impl Extractor {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    pub fn extract_shortcode(input: &str) -> Result<String, String> {
        let trimmed = input.trim();
        let pattern = Regex::new(r"(?:https?://)?(?:www\.)?instagram\.com/(?:[a-zA-Z0-9_.-]+/)?(?:p|reel|reels|tv)/([A-Za-z0-9_-]+)").unwrap();
        if let Some(captures) = pattern.captures(trimmed) {
            if let Some(matched) = captures.get(1) {
                return Ok(matched.as_str().to_string());
            }
        }

        let direct_pattern = Regex::new(r"^[A-Za-z0-9_-]{5,}$").unwrap();
        if direct_pattern.is_match(trimmed) {
            return Ok(trimmed.to_string());
        }

        Err("Could not find a valid Instagram post shortcode in URL".to_string())
    }

    pub async fn fetch_post(&self, url_or_shortcode: &str, cookie: Option<&str>) -> Result<PostInfo, String> {
        let shortcode = Self::extract_shortcode(url_or_shortcode)?;

        if cookie.is_some() {
            if let Ok(info) = self.fetch_via_graphql(&shortcode, "8845758582119845", cookie).await {
                if !info.items.is_empty() {
                    return Ok(info);
                }
            }
            if let Ok(info) = self.fetch_via_graphql(&shortcode, "10015901848480474", cookie).await {
                if !info.items.is_empty() {
                    return Ok(info);
                }
            }
        }

        if let Ok(info) = self.fetch_via_crawler(&shortcode, cookie).await {
            if !info.items.is_empty() {
                return Ok(info);
            }
        }

        if let Ok(info) = self.fetch_via_graphql(&shortcode, "8845758582119845", cookie).await {
            if !info.items.is_empty() {
                return Ok(info);
            }
        }

        if let Ok(info) = self.fetch_via_embed(&shortcode, cookie).await {
            if !info.items.is_empty() {
                return Ok(info);
            }
        }

        if let Ok(info) = self.fetch_via_info_api(&shortcode, cookie).await {
            if !info.items.is_empty() {
                return Ok(info);
            }
        }

        Err("Failed to extract media. The post may be private, deleted, or blocked by Instagram.".to_string())
    }

    async fn fetch_via_crawler(&self, shortcode: &str, cookie: Option<&str>) -> Result<PostInfo, String> {
        let post_url = format!("https://www.instagram.com/p/{}/", shortcode);
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("facebookexternalhit/1.1 (+http://www.facebook.com/externalhit_uatext.php)"),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
        );

        if let Some(cookie_val) = cookie {
            if let Ok(val) = HeaderValue::from_str(cookie_val) {
                headers.insert(COOKIE, val);
            }
        }

        let response = self
            .client
            .get(&post_url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Crawler request failed: {}", response.status()));
        }

        let html = response.text().await.map_err(|e| e.to_string())?;

        let video_re = Regex::new(r#"<meta\s+(?:property|name)=["']og:video(?:|:secure_url)["']\s+content=["']([^"']+)["']"#).unwrap();
        let video_re_rev = Regex::new(r#"<meta\s+content=["']([^"']+)["']\s+(?:property|name)=["']og:video(?:|:secure_url)["']"#).unwrap();

        let image_re = Regex::new(r#"<meta\s+(?:property|name)=["']og:image["']\s+content=["']([^"']+)["']"#).unwrap();
        let image_re_rev = Regex::new(r#"<meta\s+content=["']([^"']+)["']\s+(?:property|name)=["']og:image["']"#).unwrap();

        let desc_re = Regex::new(r#"<meta\s+(?:property|name)=["'](?:og:description|description)["']\s+content=["']([^"']+)["']"#).unwrap();
        let desc_re_rev = Regex::new(r#"<meta\s+content=["']([^"']+)["']\s+(?:property|name)=["'](?:og:description|description)["']"#).unwrap();

        let title_re = Regex::new(r#"<meta\s+(?:property|name)=["']og:title["']\s+content=["']([^"']+)["']"#).unwrap();
        let title_re_rev = Regex::new(r#"<meta\s+content=["']([^"']+)["']\s+(?:property|name)=["']og:title["']"#).unwrap();

        let video_url = video_re
            .captures(&html)
            .or_else(|| video_re_rev.captures(&html))
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().replace("&amp;", "&"));

        let image_url = image_re
            .captures(&html)
            .or_else(|| image_re_rev.captures(&html))
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().replace("&amp;", "&"));

        let title_raw = title_re
            .captures(&html)
            .or_else(|| title_re_rev.captures(&html))
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().replace("&amp;", "&"));

        let desc_raw = desc_re
            .captures(&html)
            .or_else(|| desc_re_rev.captures(&html))
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().replace("&amp;", "&"));

        let mut items = Vec::new();

        if let Some(v_url) = video_url {
            let thumb = image_url.clone().unwrap_or_else(|| v_url.clone());
            items.push(MediaItem {
                id: format!("{}_video", shortcode),
                media_type: MediaType::Video,
                url: v_url,
                thumbnail_url: thumb,
                width: None,
                height: None,
            });
        } else if let Some(i_url) = image_url.clone() {
            items.push(MediaItem {
                id: format!("{}_image", shortcode),
                media_type: MediaType::Image,
                url: i_url.clone(),
                thumbnail_url: i_url,
                width: None,
                height: None,
            });
        }

        if items.is_empty() {
            return Err("No OpenGraph media tags found in response".to_string());
        }

        let owner_username = title_raw.as_deref().and_then(|t| {
            let user_re = Regex::new(r#"@([a-zA-Z0-9_.]+)"#).unwrap();
            user_re.captures(t).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
        });

        let caption = desc_raw.map(|d| {
            let quote_re = Regex::new(r#":\s*"([^"]+)""#).unwrap();
            let raw_text = if let Some(cap) = quote_re.captures(&d) {
                if let Some(m) = cap.get(1) {
                    m.as_str().to_string()
                } else {
                    d
                }
            } else {
                d
            };
            raw_text
                .replace("&quot;", "\"")
                .replace("&amp;", "&")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&#064;", "@")
                .replace("&#x27;", "'")
        });

        Ok(PostInfo {
            shortcode: shortcode.to_string(),
            caption,
            owner_username,
            owner_full_name: None,
            items,
        })
    }

    async fn fetch_via_graphql(&self, shortcode: &str, doc_id: &str, cookie: Option<&str>) -> Result<PostInfo, String> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(
            "X-IG-App-ID",
            HeaderValue::from_static("936619743392459"),
        );
        headers.insert(
            "X-ASBD-ID",
            HeaderValue::from_static("359341"),
        );
        headers.insert(ORIGIN, HeaderValue::from_static("https://www.instagram.com"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));
        headers.insert(REFERER, HeaderValue::from_static("https://www.instagram.com/"));
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

        if let Some(cookie_val) = cookie {
            if let Ok(val) = HeaderValue::from_str(cookie_val) {
                headers.insert(COOKIE, val);
            }
        }

        let variables = serde_json::json!({
            "shortcode": shortcode,
            "fetch_tagged_user_count": null,
            "hoisted_comment_id": null,
            "hoisted_feature_comment_id": null
        });

        let form_params = [
            ("doc_id", doc_id),
            ("variables", &variables.to_string()),
        ];

        let response = self
            .client
            .post("https://www.instagram.com/graphql/query")
            .headers(headers)
            .form(&form_params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("GraphQL request failed with status: {}", response.status()));
        }

        let json_body: Value = response.json().await.map_err(|e| e.to_string())?;
        self.parse_graphql_json(&json_body, shortcode)
    }

    fn parse_graphql_json(&self, json: &Value, fallback_shortcode: &str) -> Result<PostInfo, String> {
        let media = json
            .pointer("/data/xdt_shortcode_media")
            .or_else(|| json.pointer("/data/shortcode_media"))
            .ok_or_else(|| "No media object found in GraphQL response".to_string())?;

        let shortcode = media
            .get("shortcode")
            .and_then(|v| v.as_str())
            .unwrap_or(fallback_shortcode)
            .to_string();

        let caption = media
            .pointer("/edge_media_to_caption/edges/0/node/text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let owner_username = media
            .pointer("/owner/username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let owner_full_name = media
            .pointer("/owner/full_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut items = Vec::new();

        if let Some(children) = media.pointer("/edge_sidecar_to_children/edges").and_then(|v| v.as_array()) {
            for (index, edge) in children.iter().enumerate() {
                if let Some(node) = edge.get("node") {
                    if let Some(item) = self.parse_single_node(node, &shortcode, index + 1) {
                        items.push(item);
                    }
                }
            }
        }

        if items.is_empty() {
            if let Some(item) = self.parse_single_node(media, &shortcode, 1) {
                items.push(item);
            }
        }

        if items.is_empty() {
            return Err("No downloadable media found in post".to_string());
        }

        Ok(PostInfo {
            shortcode,
            caption,
            owner_username,
            owner_full_name,
            items,
        })
    }

    fn parse_single_node(&self, node: &Value, shortcode: &str, index: usize) -> Option<MediaItem> {
        let is_video = node.get("is_video").and_then(|v| v.as_bool()).unwrap_or(false);
        let id = node
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}_{}", shortcode, index));

        let display_url = node
            .get("display_url")
            .and_then(|v| v.as_str())
            .or_else(|| node.get("display_src").and_then(|v| v.as_str()))?
            .to_string();

        let (media_type, target_url) = if is_video {
            let video_url = node
                .get("video_url")
                .and_then(|v| v.as_str())
                .unwrap_or(&display_url)
                .to_string();
            (MediaType::Video, video_url)
        } else {
            (MediaType::Image, display_url.clone())
        };

        let width = node
            .pointer("/dimensions/width")
            .and_then(|v| v.as_u64())
            .map(|w| w as u32);

        let height = node
            .pointer("/dimensions/height")
            .and_then(|v| v.as_u64())
            .map(|h| h as u32);

        Some(MediaItem {
            id,
            media_type,
            url: target_url,
            thumbnail_url: display_url,
            width,
            height,
        })
    }

    async fn fetch_via_embed(&self, shortcode: &str, cookie: Option<&str>) -> Result<PostInfo, String> {
        let embed_url = format!("https://www.instagram.com/p/{}/embed/captioned/", shortcode);
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(REFERER, HeaderValue::from_static("https://www.instagram.com/"));

        if let Some(cookie_val) = cookie {
            if let Ok(val) = HeaderValue::from_str(cookie_val) {
                headers.insert(COOKIE, val);
            }
        }

        let response = self
            .client
            .get(&embed_url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Embed request failed: {}", response.status()));
        }

        let html = response.text().await.map_err(|e| e.to_string())?;
        self.parse_embed_html(&html, shortcode)
    }

    fn parse_embed_html(&self, html: &str, shortcode: &str) -> Result<PostInfo, String> {
        let mut items = Vec::new();

        let video_regex = Regex::new(r#"class="EmbeddedMediaVideo"[^>]*src="([^"]+)""#).unwrap();
        let fallback_video_regex = Regex::new(r#"<video[^>]*src="([^"]+)""#).unwrap();
        let image_regex = Regex::new(r#"class="EmbeddedMediaImage"[^>]*src="([^"]+)""#).unwrap();
        let caption_regex = Regex::new(r#"class="Caption"[^>]*>(.*?)</div>"#).unwrap();

        let mut video_urls = Vec::new();
        for cap in video_regex.captures_iter(html) {
            if let Some(url_match) = cap.get(1) {
                let decoded = url_match.as_str().replace("&amp;", "&");
                if !video_urls.contains(&decoded) {
                    video_urls.push(decoded);
                }
            }
        }

        if video_urls.is_empty() {
            for cap in fallback_video_regex.captures_iter(html) {
                if let Some(url_match) = cap.get(1) {
                    let decoded = url_match.as_str().replace("&amp;", "&");
                    if !video_urls.contains(&decoded) {
                        video_urls.push(decoded);
                    }
                }
            }
        }

        let mut image_urls = Vec::new();
        for cap in image_regex.captures_iter(html) {
            if let Some(url_match) = cap.get(1) {
                let decoded = url_match.as_str().replace("&amp;", "&");
                if !image_urls.contains(&decoded) {
                    image_urls.push(decoded);
                }
            }
        }

        for (index, video_url) in video_urls.iter().enumerate() {
            let thumb = image_urls.get(index).cloned().unwrap_or_else(|| video_url.clone());
            items.push(MediaItem {
                id: format!("{}_v{}", shortcode, index + 1),
                media_type: MediaType::Video,
                url: video_url.clone(),
                thumbnail_url: thumb,
                width: None,
                height: None,
            });
        }

        if items.is_empty() {
            for (index, image_url) in image_urls.iter().enumerate() {
                items.push(MediaItem {
                    id: format!("{}_img{}", shortcode, index + 1),
                    media_type: MediaType::Image,
                    url: image_url.clone(),
                    thumbnail_url: image_url.clone(),
                    width: None,
                    height: None,
                });
            }
        }

        let caption = caption_regex.captures(html).and_then(|cap| {
            cap.get(1).map(|m| {
                let raw = m.as_str();
                let strip_tags = Regex::new(r"<[^>]*>").unwrap();
                strip_tags.replace_all(raw, "").trim().to_string()
            })
        });

        if items.is_empty() {
            return Err("Could not extract any media elements from embed view".to_string());
        }

        Ok(PostInfo {
            shortcode: shortcode.to_string(),
            caption,
            owner_username: None,
            owner_full_name: None,
            items,
        })
    }

    async fn fetch_via_info_api(&self, shortcode: &str, cookie: Option<&str>) -> Result<PostInfo, String> {
        let endpoint = format!("https://www.instagram.com/p/{}/?__a=1&__d=dis", shortcode);
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(
            "X-IG-App-ID",
            HeaderValue::from_static("936619743392459"),
        );
        headers.insert(REFERER, HeaderValue::from_static("https://www.instagram.com/"));

        if let Some(cookie_val) = cookie {
            if let Ok(val) = HeaderValue::from_str(cookie_val) {
                headers.insert(COOKIE, val);
            }
        }

        let response = self
            .client
            .get(&endpoint)
            .headers(headers)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Info API failed: {}", response.status()));
        }

        let json_body: Value = response.json().await.map_err(|e| e.to_string())?;
        if let Ok(post) = self.parse_graphql_json(&json_body, shortcode) {
            return Ok(post);
        }

        if let Some(items_arr) = json_body.get("items").and_then(|v| v.as_array()) {
            if let Some(first_item) = items_arr.first() {
                return self.parse_graphql_json(first_item, shortcode);
            }
        }

        Err("Failed to parse info API response".to_string())
    }
}
