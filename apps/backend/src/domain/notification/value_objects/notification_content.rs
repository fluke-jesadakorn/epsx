use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Notification Content Value Object
/// Encapsulates notification title and body with validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationContent {
    title: String,
    body: String,
    urgency: ContentUrgency,
}

impl NotificationContent {
    /// Create new notification content with validation
    pub fn new(title: String, body: String) -> Result<Self, String> {
        let title = title.trim().to_string();
        let body = body.trim().to_string();

        // Title validation
        if title.is_empty() {
            return Err("Notification title cannot be empty".to_string());
        }

        if title.len() > 200 {
            return Err("Notification title cannot exceed 200 characters".to_string());
        }

        // Body validation
        if body.is_empty() {
            return Err("Notification body cannot be empty".to_string());
        }

        if body.len() > 2000 {
            return Err("Notification body cannot exceed 2000 characters".to_string());
        }

        // Check for potentially harmful content
        if Self::contains_suspicious_content(&title) || Self::contains_suspicious_content(&body) {
            return Err("Notification content contains suspicious patterns".to_string());
        }

        Ok(Self {
            title,
            body,
            urgency: ContentUrgency::Normal, // Default urgency
        })
    }

    /// Create notification content with specific urgency level
    pub fn with_urgency(title: String, body: String, urgency: ContentUrgency) -> Result<Self, String> {
        let mut content = Self::new(title, body)?;
        content.urgency = urgency;
        Ok(content)
    }

    /// Get the title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the body
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Get the urgency level
    pub fn urgency(&self) -> &ContentUrgency {
        &self.urgency
    }

    /// Get content length (title + body)
    pub fn total_length(&self) -> usize {
        self.title.len() + self.body.len()
    }

    /// Check if content is appropriate for mobile push notifications (shorter)
    pub fn is_mobile_friendly(&self) -> bool {
        self.title.len() <= 50 && self.body.len() <= 150
    }

    /// Get truncated version for mobile notifications
    pub fn mobile_version(&self) -> Self {
        let truncated_title = if self.title.len() > 50 {
            format!("{}...", &self.title[..47])
        } else {
            self.title.clone()
        };

        let truncated_body = if self.body.len() > 150 {
            format!("{}...", &self.body[..147])
        } else {
            self.body.clone()
        };

        Self {
            title: truncated_title,
            body: truncated_body,
            urgency: self.urgency,
        }
    }

    /// Get preview text (first line of body or truncated version)
    pub fn preview(&self, max_length: usize) -> String {
        let first_line = self.body.lines().next().unwrap_or("");
        if first_line.len() <= max_length {
            first_line.to_string()
        } else {
            format!("{}...", &first_line[..max_length.saturating_sub(3)])
        }
    }

    /// Check if content contains searchable keywords
    pub fn contains_keyword(&self, keyword: &str) -> bool {
        let keyword_lower = keyword.to_lowercase();
        self.title.to_lowercase().contains(&keyword_lower)
            || self.body.to_lowercase().contains(&keyword_lower)
    }

    /// Get content type based on patterns
    pub fn inferred_urgency(&self) -> ContentUrgency {
        let content = format!("{} {}", self.title, self.body).to_lowercase();
        
        if content.contains("urgent") || content.contains("immediate") || content.contains("emergency") {
            ContentUrgency::Urgent
        } else if content.contains("important") || content.contains("attention") || content.contains("alert") {
            ContentUrgency::High
        } else if content.contains("reminder") || content.contains("notice") || content.contains("update") {
            ContentUrgency::Normal
        } else {
            ContentUrgency::Low
        }
    }

    /// Check for suspicious content patterns
    fn contains_suspicious_content(content: &str) -> bool {
        let suspicious_patterns = [
            "<script",
            "javascript:",
            "data:text/html",
            "vbscript:",
            "<iframe",
            "onclick=",
            "onerror=",
            "onload=",
        ];

        let content_lower = content.to_lowercase();
        suspicious_patterns.iter().any(|&pattern| content_lower.contains(pattern))
    }

    /// Sanitize content for safe display
    pub fn sanitized(&self) -> Self {
        Self {
            title: self.sanitize_text(&self.title),
            body: self.sanitize_text(&self.body),
            urgency: self.urgency,
        }
    }

    /// Basic HTML entity sanitization
    fn sanitize_text(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
}

/// Content urgency classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentUrgency {
    Low,
    Normal,
    High,
    Urgent,
}

impl ContentUrgency {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentUrgency::Low => "low",
            ContentUrgency::Normal => "normal",
            ContentUrgency::High => "high",
            ContentUrgency::Urgent => "urgent",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "low" => Ok(ContentUrgency::Low),
            "normal" => Ok(ContentUrgency::Normal),
            "high" => Ok(ContentUrgency::High),
            "urgent" => Ok(ContentUrgency::Urgent),
            _ => Err(format!("Invalid content urgency: {}", s)),
        }
    }
}

impl Display for NotificationContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.title, self.preview(50))
    }
}

impl TryFrom<(String, String)> for NotificationContent {
    type Error = String;

    fn try_from((title, body): (String, String)) -> Result<Self, Self::Error> {
        NotificationContent::new(title, body)
    }
}

impl TryFrom<(&str, &str)> for NotificationContent {
    type Error = String;

    fn try_from((title, body): (&str, &str)) -> Result<Self, Self::Error> {
        NotificationContent::new(title.to_string(), body.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_content() {
        let content = NotificationContent::new(
            "Test Title".to_string(),
            "This is a test notification body".to_string(),
        ).unwrap();

        assert_eq!(content.title(), "Test Title");
        assert_eq!(content.body(), "This is a test notification body");
    }

    #[test]
    fn test_empty_title() {
        let result = NotificationContent::new(
            "".to_string(),
            "Valid body".to_string(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_empty_body() {
        let result = NotificationContent::new(
            "Valid title".to_string(),
            "".to_string(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_title_too_long() {
        let long_title = "A".repeat(201);
        let result = NotificationContent::new(
            long_title,
            "Valid body".to_string(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("200 characters"));
    }

    #[test]
    fn test_body_too_long() {
        let long_body = "A".repeat(2001);
        let result = NotificationContent::new(
            "Valid title".to_string(),
            long_body,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("2000 characters"));
    }

    #[test]
    fn test_suspicious_content() {
        let result = NotificationContent::new(
            "Malicious <script>alert('xss')</script>".to_string(),
            "Valid body".to_string(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("suspicious"));
    }

    #[test]
    fn test_mobile_friendly() {
        let short_content = NotificationContent::new(
            "Short".to_string(),
            "Brief message".to_string(),
        ).unwrap();
        assert!(short_content.is_mobile_friendly());

        let long_content = NotificationContent::new(
            "This is a very long title that exceeds mobile limits".to_string(),
            "This is also a very long body that would not fit well on mobile devices and should be truncated for better user experience".to_string(),
        ).unwrap();
        assert!(!long_content.is_mobile_friendly());
    }

    #[test]
    fn test_mobile_version() {
        let content = NotificationContent::new(
            "This is a very long title that exceeds mobile limits and needs truncation".to_string(),
            "This is also a very long body that would not fit well on mobile devices and should be truncated for better user experience on smaller screens. We add some extra text here to make sure it surpasses 150 characters.".to_string(),
        ).unwrap();

        let mobile = content.mobile_version();
        assert!(mobile.title().len() <= 50);
        assert!(mobile.body().len() <= 150);
        assert!(mobile.title().ends_with("..."));
        assert!(mobile.body().ends_with("..."));
    }

    #[test]
    fn test_preview() {
        let content = NotificationContent::new(
            "Title".to_string(),
            "First line\nSecond line\nThird line".to_string(),
        ).unwrap();

        assert_eq!(content.preview(20), "First line");
        assert_eq!(content.preview(5), "Fi...");
    }

    #[test]
    fn test_keyword_search() {
        let content = NotificationContent::new(
            "Important Update".to_string(),
            "System maintenance scheduled".to_string(),
        ).unwrap();

        assert!(content.contains_keyword("important"));
        assert!(content.contains_keyword("SYSTEM"));
        assert!(!content.contains_keyword("urgent"));
    }

    #[test]
    fn test_urgency_inference() {
        let urgent = NotificationContent::new(
            "URGENT: System Down".to_string(),
            "Immediate action required".to_string(),
        ).unwrap();
        assert_eq!(urgent.inferred_urgency(), ContentUrgency::Urgent);

        let normal = NotificationContent::new(
            "Reminder: Meeting Tomorrow".to_string(),
            "Don't forget about the meeting".to_string(),
        ).unwrap();
        assert_eq!(normal.inferred_urgency(), ContentUrgency::Normal);

        let low = NotificationContent::new(
            "Welcome to the platform".to_string(),
            "Thanks for joining us".to_string(),
        ).unwrap();
        assert_eq!(low.inferred_urgency(), ContentUrgency::Low);
    }

    #[test]
    fn test_sanitization() {
        let content = NotificationContent::new(
            "Title with <tags>".to_string(),
            "Body with \"quotes\" & ampersands".to_string(),
        ).unwrap();

        let sanitized = content.sanitized();
        assert!(sanitized.title().contains("&lt;tags&gt;"));
        assert!(sanitized.body().contains("&quot;quotes&quot;"));
        assert!(sanitized.body().contains("&amp;"));
    }
}