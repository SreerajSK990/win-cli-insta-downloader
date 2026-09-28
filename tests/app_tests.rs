use insta::downloader::Downloader;
use insta::extractor::Extractor;
use insta::models::{MediaType, PostInfo};

#[test]
fn test_extract_shortcode_from_post_url() {
    let url = "https://www.instagram.com/p/DB123456789/?utm_source=ig_web_copy_link";
    let code = Extractor::extract_shortcode(url).unwrap();
    assert_eq!(code, "DB123456789");
}

#[test]
fn test_extract_shortcode_from_reel_url() {
    let url = "https://instagram.com/reel/C8_xyz123";
    let code = Extractor::extract_shortcode(url).unwrap();
    assert_eq!(code, "C8_xyz123");
}

#[test]
fn test_extract_shortcode_from_reels_plural_url() {
    let url = "https://www.instagram.com/reels/DA-987abc/";
    let code = Extractor::extract_shortcode(url).unwrap();
    assert_eq!(code, "DA-987abc");
}

#[test]
fn test_extract_shortcode_direct_string() {
    let raw = "Cxyz9876_";
    let code = Extractor::extract_shortcode(raw).unwrap();
    assert_eq!(code, "Cxyz9876_");
}

#[test]
fn test_invalid_url_returns_error() {
    let invalid = "https://example.com/not-instagram";
    assert!(Extractor::extract_shortcode(invalid).is_err());
}

#[test]
fn test_build_filename_single_video() {
    let filename = Downloader::build_filename("C12345", 1, 1, &MediaType::Video);
    assert_eq!(filename, "insta_C12345.mp4");
}

#[test]
fn test_build_filename_carousel_image() {
    let filename = Downloader::build_filename("C12345", 3, 10, &MediaType::Image);
    assert_eq!(filename, "insta_C12345_03.jpg");
}

#[test]
fn test_post_info_serialization() {
    let info = PostInfo {
        shortcode: "TEST1234".to_string(),
        caption: Some("Test caption".to_string()),
        owner_username: Some("testuser".to_string()),
        owner_full_name: Some("Test User".to_string()),
        items: vec![],
    };

    let serialized = serde_json::to_string(&info).unwrap();
    let deserialized: PostInfo = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.shortcode, "TEST1234");
    assert_eq!(deserialized.owner_username.as_deref(), Some("testuser"));
}

#[test]
fn test_parse_desktop_html_polaris_carousel() {
    let mock_html = r#"
    <!DOCTYPE html><html><head></head><body>
    <script type="application/json">
    {"data":{"xig_polaris_media":{"code":"DdY_hGCCZ3A","if_not_gated_logged_out":{"caption":{"text":"Moonlight"},"user":{"username":"celistima","full_name":"Celistima"},"carousel_media":[{"pk":"111","original_width":1440,"original_height":2558,"display_uri":"https://thumb1","image_versions2":{"candidates":[{"url":"https://highres1"}]}},{"pk":"222","original_width":1440,"original_height":2558,"display_uri":"https://thumb2","image_versions2":{"candidates":[{"url":"https://highres2"}]}}]}}}}
    </script>
    </body></html>
    "#;

    let extractor = Extractor::new();
    let post = extractor.parse_desktop_html(mock_html, "DdY_hGCCZ3A").unwrap();

    assert_eq!(post.shortcode, "DdY_hGCCZ3A");
    assert_eq!(post.owner_username.as_deref(), Some("celistima"));
    assert_eq!(post.caption.as_deref(), Some("Moonlight"));
    assert_eq!(post.items.len(), 2);
    assert_eq!(post.items[0].url, "https://highres1");
    assert_eq!(post.items[0].thumbnail_url, "https://thumb1");
    assert_eq!(post.items[0].width, Some(1440));
    assert_eq!(post.items[0].height, Some(2558));
    assert_eq!(post.items[1].url, "https://highres2");
}

#[test]
fn test_parse_real_captured_page_if_present() {
    let path = std::path::PathBuf::from(r"C:\Users\sreerajsk\.gemini\antigravity\brain\a742fb1b-1a25-4481-8d90-e85552a66d89\scratch\curl_page.html");
    if path.exists() {
        let html = std::fs::read_to_string(path).unwrap();
        let extractor = Extractor::new();
        let post = extractor.parse_desktop_html(&html, "DdY_hGCCZ3A").unwrap();
        assert_eq!(post.shortcode, "DdY_hGCCZ3A");
        assert_eq!(post.owner_username.as_deref(), Some("celistima"));
        assert_eq!(post.items.len(), 10);
        assert_eq!(post.items[0].width, Some(1440));
        assert_eq!(post.items[0].height, Some(2558));
    }
}



