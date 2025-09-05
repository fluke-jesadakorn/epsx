// Session Management Value Objects
// Focused on session persistence, grouping, and lifecycle management

pub mod session_metadata;
pub mod session_collection;
pub mod session_activity;
pub mod session_history;
pub mod device_info;
pub mod suspicious_pattern;
pub mod session_collection_error;

// Re-export all value objects
pub use session_metadata::{SessionMetadata, SessionStatus, IpAddressInfo};
pub use session_collection::{SessionCollection, SessionCollectionSummary};
pub use session_activity::{SessionActivity, ActivityType, ActivityMetrics};
pub use session_history::{SessionHistory, HistoryEntry, HistoryEventType};
pub use device_info::{DeviceInfo, DeviceType, DeviceFingerprint};
pub use suspicious_pattern::{SuspiciousPattern, PatternType, SeverityLevel, Evidence, EvidenceType, SuspiciousPatternError};
pub use session_collection_error::{SessionCollectionError, ErrorCategory, ContextualSessionCollectionError, ErrorContext};