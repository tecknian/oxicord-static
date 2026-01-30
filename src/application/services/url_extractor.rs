#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_markdown_images() {
        let content = "Here is an image ![alt text](https://example.com/image.png)";
        let urls = UrlExtractor::extract_image_urls(content);
        assert_eq!(urls, vec!["https://example.com/image.png"]);
    }

    #[test]
    fn test_extract_direct_images() {
        let content = "Check this out https://example.com/pic.jpg cool right?";
        let urls = UrlExtractor::extract_image_urls(content);
        assert_eq!(urls, vec!["https://example.com/pic.jpg"]);
    }

    #[test]
    fn test_extract_mixed_images() {
        let content = "![img](https://a.com/1.png) and https://b.com/2.jpg";
        let urls = UrlExtractor::extract_image_urls(content);
        assert!(urls.contains(&"https://a.com/1.png".to_string()));
        assert!(urls.contains(&"https://b.com/2.jpg".to_string()));
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn test_deduplication() {
        let content = "https://a.com/1.png and https://a.com/1.png";
        let urls = UrlExtractor::extract_image_urls(content);
        assert_eq!(urls.len(), 1);
    }

    #[test]
    fn test_no_images() {
        let content = "Just some text with no images.";
        let urls = UrlExtractor::extract_image_urls(content);
        assert!(urls.is_empty());
    }
}

use regex::Regex;
use std::sync::LazyLock;

pub struct UrlExtractor;

impl UrlExtractor {
    pub fn extract_image_urls(content: &str) -> Vec<String> {
        static MD_IMAGE_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"!\[[^\]]*\]\((https?://[^)]+)\)").unwrap());

        static DIRECT_IMAGE_RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"(?:^|\s)(https?://[^\s]+\.(?:png|jpg|jpeg|gif|webp)(?:\?[^\s]*)?)(?:\s|$)")
                .unwrap()
        });

        if !content.contains("http") {
            return Vec::new();
        }

        let mut urls: Vec<String> = Vec::new();

        for cap in MD_IMAGE_RE.captures_iter(content) {
            if let Some(url) = cap.get(1) {
                let url_str = url.as_str().to_owned();
                if !urls.contains(&url_str) {
                    urls.push(url_str);
                }
            }
        }

        for cap in DIRECT_IMAGE_RE.captures_iter(content) {
            if let Some(url) = cap.get(1) {
                let url_str = url.as_str().to_owned();
                if !urls.contains(&url_str) {
                    urls.push(url_str);
                }
            }
        }

        urls
    }
}
