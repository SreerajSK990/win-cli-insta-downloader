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
