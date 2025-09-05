// Event Channel Value Objects

use serde::{Deserialize, Serialize};

/// Event channel for organizing events
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventChannel(String);

impl EventChannel {
    pub fn new(channel: String) -> Self {
        Self(channel)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Subscription topic for filtering events
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionTopic(String);

impl SubscriptionTopic {
    pub fn new(topic: String) -> Self {
        Self(topic)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}