// Event Metadata Value Object

use serde::{Deserialize, Serialize};

/// Priority levels for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Metadata associated with events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub priority: EventPriority,
    pub ttl_minutes: Option<u64>,
    pub retry_count: u32,
    pub tags: Vec<String>,
}