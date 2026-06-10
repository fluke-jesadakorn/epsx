use lazy_static::lazy_static;
use regex::Regex;
use url::Url;

/// Sanitizes chat content to ensure only verified internal links and safe external domains are allowed.
/// Replaces external links with redacted text.
pub fn sanitize_chat_content(content: &str) -> String {
    lazy_static! {
        // Matches markdown links: [label](url)
        static ref MD_LINK: Regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
        // Matches bare URLs (basic catch-all for http/https)
        static ref BARE_URL: Regex = Regex::new(r"(?i)\bhttps?://[^\s()<>]+(?:\([\w\d]+\)|([^[:punct:]\s]|/))").unwrap();
    }

    // Process markdown links first
    let with_safe_md_links = MD_LINK.replace_all(content, |caps: &regex::Captures| {
        let label = &caps[1];
        let url = &caps[2];

        // Ensure attachments are preserved literally
        if url.starts_with("[attachment:") {
            return caps[0].to_string();
        }

        if let Ok(parsed_url) = Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                if host == "localhost" || host.ends_with("epsx.io") {
                    return format!("[{}]({})", label, url);
                }
            }
        } else if url.starts_with('/') {
            // Allow relative links (e.g., /chat/123)
            return format!("[{}]({})", label, url);
        }

        "*[External Link Removed]*".to_string()
    }).to_string();

    // Process bare URLs that weren't inside markdown links
    BARE_URL.replace_all(&with_safe_md_links, |caps: &regex::Captures| {
        let url = &caps[0];
        if let Ok(parsed_url) = Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                if host == "localhost" || host.ends_with("epsx.io") {
                    return url.to_string();
                }
            }
        }
        "[External Link Removed]".to_string()
    }).to_string()
}
