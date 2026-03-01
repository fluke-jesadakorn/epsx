use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bucket {
    Chat,
    News,
    Notifications,
    Public,
}

impl Bucket {
    pub fn as_str(&self) -> &'static str {
        match self {
            Bucket::Chat => "chat",
            Bucket::News => "news",
            Bucket::Notifications => "notifications",
            Bucket::Public => "public",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "chat" => Some(Bucket::Chat),
            "news" => Some(Bucket::News),
            "notifications" => Some(Bucket::Notifications),
            "public" => Some(Bucket::Public),
            _ => None,
        }
    }

    pub fn all() -> &'static [Bucket] {
        &[Bucket::Chat, Bucket::News, Bucket::Notifications, Bucket::Public]
    }
}

impl fmt::Display for Bucket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UploadResult {
    pub key: String,
    pub url: String,
    pub thumb_url: Option<String>,
    pub mime: String,
    pub size: usize,
    pub original_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    pub key: String,
    pub url: String,
    pub size: i64,
    pub last_modified: Option<String>,
}
