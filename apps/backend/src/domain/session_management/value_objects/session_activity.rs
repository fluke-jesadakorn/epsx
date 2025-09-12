use crate::domain::shared_kernel::value_objects::SessionId;
// Session Activity Value Object
// Tracks activities and interactions within a session

use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};


/// Session activity tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivity {
    /// Session this activity belongs to
    pub session_id: SessionId,
    
    /// Activity history (limited size for memory efficiency)
    pub activities: VecDeque<ActivityEntry>,
    
    /// Activity metrics
    pub metrics: ActivityMetrics,
    
    /// Configuration
    pub max_activities: usize,
    pub track_detailed_activities: bool,
}

impl SessionActivity {
    /// Create new session activity tracker
    pub fn new(session_id: SessionId) -> Self {
        Self {
            session_id,
            activities: VecDeque::new(),
            metrics: ActivityMetrics::new(),
            max_activities: 1000, // Keep last 1000 activities
            track_detailed_activities: true,
        }
    }
    
    /// Record a new activity
    pub fn record_activity(&mut self, activity_type: ActivityType, details: Option<String>) {
        let activity = ActivityEntry {
            activity_type: activity_type.clone(),
            timestamp: Utc::now(),
            details,
            ip_address: None, // Would be set by caller
            user_agent: None, // Would be set by caller
        };
        
        // Add to activity list
        self.activities.push_back(activity);
        
        // Maintain size limit
        if self.activities.len() > self.max_activities {
            self.activities.pop_front();
        }
        
        // Update metrics
        self.metrics.record_activity(&activity_type);
    }
    
    /// Record activity with client info
    pub fn record_activity_with_client(&mut self, 
        activity_type: ActivityType, 
        details: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>
    ) {
        let activity = ActivityEntry {
            activity_type: activity_type.clone(),
            timestamp: Utc::now(),
            details,
            ip_address,
            user_agent,
        };
        
        self.activities.push_back(activity);
        
        if self.activities.len() > self.max_activities {
            self.activities.pop_front();
        }
        
        self.metrics.record_activity(&activity_type);
    }
    
    /// Get recent activities (last N)
    pub fn get_recent_activities(&self, count: usize) -> Vec<&ActivityEntry> {
        self.activities
            .iter()
            .rev()
            .take(count)
            .collect()
    }
    
    /// Get activities by type
    pub fn get_activities_by_type(&self, activity_type: &ActivityType) -> Vec<&ActivityEntry> {
        self.activities
            .iter()
            .filter(|activity| &activity.activity_type == activity_type)
            .collect()
    }
    
    /// Get activities in time range
    pub fn get_activities_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&ActivityEntry> {
        self.activities
            .iter()
            .filter(|activity| activity.timestamp >= start && activity.timestamp <= end)
            .collect()
    }
    
    /// Check for suspicious activity patterns
    pub fn detect_suspicious_patterns(&self) -> Vec<SuspiciousPattern> {
        let mut patterns = vec![];
        
        // Pattern 1: Too many API calls in short time
        let recent_api_calls = self.get_activities_in_range(
            Utc::now() - Duration::minutes(5),
            Utc::now()
        ).into_iter()
            .filter(|a| matches!(a.activity_type, ActivityType::ApiCall(_)))
            .count();
            
        if recent_api_calls > 100 {
            patterns.push(SuspiciousPattern::HighFrequencyApiCalls {
                count: recent_api_calls,
                timeframe_minutes: 5,
            });
        }
        
        // Pattern 2: Multiple failed authentications
        let recent_failures = self.get_activities_by_type(&ActivityType::AuthenticationFailed)
            .into_iter()
            .filter(|a| a.timestamp > Utc::now() - Duration::minutes(10))
            .count();
            
        if recent_failures > 3 {
            patterns.push(SuspiciousPattern::MultipleAuthFailures {
                count: recent_failures,
                timeframe_minutes: 10,
            });
        }
        
        // Pattern 3: Unusual access patterns
        let unique_ips_recent = self.activities
            .iter()
            .filter(|a| a.timestamp > Utc::now() - Duration::hours(1))
            .filter_map(|a| a.ip_address.as_ref())
            .collect::<std::collections::HashSet<_>>()
            .len();
            
        if unique_ips_recent > 5 {
            patterns.push(SuspiciousPattern::MultipleIpAddresses {
                count: unique_ips_recent,
                timeframe_hours: 1,
            });
        }
        
        patterns
    }
    
    /// Get activity summary
    pub fn get_summary(&self) -> ActivitySummary {
        let total_activities = self.activities.len();
        let unique_ips = self.activities
            .iter()
            .filter_map(|a| a.ip_address.as_ref())
            .collect::<std::collections::HashSet<_>>()
            .len();
            
        let most_recent = self.activities.back().map(|a| a.timestamp);
        let oldest_tracked = self.activities.front().map(|a| a.timestamp);
        
        ActivitySummary {
            session_id: self.session_id.clone(),
            total_activities,
            unique_ip_addresses: unique_ips,
            most_recent_activity: most_recent,
            oldest_tracked_activity: oldest_tracked,
            activity_metrics: self.metrics.clone(),
            suspicious_patterns: self.detect_suspicious_patterns(),
        }
    }
    
    /// Clear old activities beyond retention period
    pub fn cleanup_old_activities(&mut self, retention_hours: u32) {
        let cutoff = Utc::now() - Duration::hours(retention_hours as i64);
        
        while let Some(front) = self.activities.front() {
            if front.timestamp < cutoff {
                self.activities.pop_front();
            } else {
                break;
            }
        }
    }
}

/// Individual activity entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub activity_type: ActivityType,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Types of activities that can be tracked
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    /// User authentication
    Authentication,
    AuthenticationFailed,
    TokenRefresh,
    
    /// API interactions
    ApiCall(String), // Include endpoint/resource
    DataAccess(String), // Include data type accessed
    
    /// User interface interactions
    PageView(String),
    ButtonClick(String),
    FormSubmission(String),
    
    /// System events
    SessionCreated,
    SessionTerminated,
    PermissionChanged,
    SecurityAlert,
    
    /// Application-specific
    TradeExecuted,
    AnalyticsQuery,
    NotificationSent,
    AdminActionPerformed(String),
    
    /// Generic activity with custom type
    Custom(String),
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityType::Authentication => write!(f, "authentication"),
            ActivityType::AuthenticationFailed => write!(f, "auth_failed"),
            ActivityType::TokenRefresh => write!(f, "token_refresh"),
            ActivityType::ApiCall(endpoint) => write!(f, "api_call:{}", endpoint),
            ActivityType::DataAccess(data_type) => write!(f, "data_access:{}", data_type),
            ActivityType::PageView(page) => write!(f, "page_view:{}", page),
            ActivityType::ButtonClick(button) => write!(f, "button_click:{}", button),
            ActivityType::FormSubmission(form) => write!(f, "form_submit:{}", form),
            ActivityType::SessionCreated => write!(f, "session_created"),
            ActivityType::SessionTerminated => write!(f, "session_terminated"),
            ActivityType::PermissionChanged => write!(f, "permission_changed"),
            ActivityType::SecurityAlert => write!(f, "security_alert"),
            ActivityType::TradeExecuted => write!(f, "trade_executed"),
            ActivityType::AnalyticsQuery => write!(f, "analytics_query"),
            ActivityType::NotificationSent => write!(f, "notification_sent"),
            ActivityType::AdminActionPerformed(action) => write!(f, "admin_action:{}", action),
            ActivityType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// Activity metrics for session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityMetrics {
    /// Total activity counts by type
    pub activity_counts: HashMap<String, u64>,
    
    /// Time-based metrics
    pub first_activity: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
    
    /// Rate metrics (activities per minute)
    pub peak_activity_rate: f64,
    pub average_activity_rate: f64,
    
    /// Security metrics
    pub failed_auth_attempts: u32,
    pub security_alerts: u32,
    
    /// Application metrics
    pub api_calls: u64,
    pub page_views: u64,
    pub data_accesses: u64,
}

impl ActivityMetrics {
    pub fn new() -> Self {
        Self {
            activity_counts: HashMap::new(),
            first_activity: None,
            last_activity: None,
            peak_activity_rate: 0.0,
            average_activity_rate: 0.0,
            failed_auth_attempts: 0,
            security_alerts: 0,
            api_calls: 0,
            page_views: 0,
            data_accesses: 0,
        }
    }
    
    pub fn record_activity(&mut self, activity_type: &ActivityType) {
        let now = Utc::now();
        
        // Update activity counts
        let type_name = activity_type.to_string();
        *self.activity_counts.entry(type_name).or_insert(0) += 1;
        
        // Update time-based metrics
        if self.first_activity.is_none() {
            self.first_activity = Some(now);
        }
        self.last_activity = Some(now);
        
        // Update specific metrics
        match activity_type {
            ActivityType::AuthenticationFailed => self.failed_auth_attempts += 1,
            ActivityType::SecurityAlert => self.security_alerts += 1,
            ActivityType::ApiCall(_) => self.api_calls += 1,
            ActivityType::PageView(_) => self.page_views += 1,
            ActivityType::DataAccess(_) => self.data_accesses += 1,
            _ => {}
        }
        
        // Recalculate rates (simplified - would be more sophisticated in production)
        if let (Some(first), Some(last)) = (self.first_activity, self.last_activity) {
            let duration_minutes = (last - first).num_minutes().max(1);
            let total_activities: u64 = self.activity_counts.values().sum();
            self.average_activity_rate = total_activities as f64 / duration_minutes as f64;
        }
    }
}

/// Suspicious activity patterns detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuspiciousPattern {
    HighFrequencyApiCalls {
        count: usize,
        timeframe_minutes: u32,
    },
    MultipleAuthFailures {
        count: usize,
        timeframe_minutes: u32,
    },
    MultipleIpAddresses {
        count: usize,
        timeframe_hours: u32,
    },
    UnusualDataAccess {
        accessed_resources: Vec<String>,
        timeframe_minutes: u32,
    },
    BotLikeActivity {
        pattern_description: String,
        confidence: f64,
    },
}

/// Activity summary for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub session_id: SessionId,
    pub total_activities: usize,
    pub unique_ip_addresses: usize,
    pub most_recent_activity: Option<DateTime<Utc>>,
    pub oldest_tracked_activity: Option<DateTime<Utc>>,
    pub activity_metrics: ActivityMetrics,
    pub suspicious_patterns: Vec<SuspiciousPattern>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::authentication::SessionId;
    
    #[test]
    fn create_session_activity() {
        let session_id = SessionId::generate();
        let activity = SessionActivity::new(session_id.clone());
        
        assert_eq!(activity.session_id, session_id);
        assert_eq!(activity.activities.len(), 0);
        assert_eq!(activity.metrics.activity_counts.len(), 0);
    }
    
    #[test]
    fn record_activities() {
        let session_id = SessionId::generate();
        let activity = SessionActivity::new(session_id);
        
        activity.record_activity(ActivityType::Authentication, Some("Login attempt".to_string()));
        activity.record_activity(ActivityType::ApiCall("analytics".to_string()), None);
        activity.record_activity(ActivityType::PageView("dashboard".to_string()), None);
        
        assert_eq!(activity.activities.len(), 3);
        assert_eq!(activity.metrics.api_calls, 1);
        assert_eq!(activity.metrics.page_views, 1);
        assert!(activity.metrics.first_activity.is_some());
        assert!(activity.metrics.last_activity.is_some());
    }
    
    #[test]
    fn activity_size_limit() {
        let session_id = SessionId::generate();
        let activity = SessionActivity::new(session_id);
        activity.max_activities = 5; // Small limit for testing
        
        // Add more activities than limit
        for i in 0..10 {
            activity.record_activity(
                ActivityType::ApiCall(format!("endpoint_{}", i)), 
                None
            );
        }
        
        // Should only keep the last 5
        assert_eq!(activity.activities.len(), 5);
        
        // Should have the most recent ones
        let recent = activity.get_recent_activities(5);
        assert_eq!(recent.len(), 5);
    }
    
    #[test]
    fn suspicious_pattern_detection() {
        let session_id = SessionId::generate();
        let activity = SessionActivity::new(session_id);
        
        // Add multiple auth failures
        for _ in 0..5 {
            activity.record_activity(ActivityType::AuthenticationFailed, None);
        }
        
        let patterns = activity.detect_suspicious_patterns();
        assert!(!patterns.is_empty());
        
        // Should detect multiple auth failures
        assert!(patterns.iter().any(|p| matches!(p, SuspiciousPattern::MultipleAuthFailures { .. })));
    }
    
    #[test]
    fn activity_filtering() {
        let session_id = SessionId::generate();
        let activity = SessionActivity::new(session_id);
        
        // Add different types of activities
        activity.record_activity(ActivityType::Authentication, None);
        activity.record_activity(ActivityType::ApiCall("test".to_string()), None);
        activity.record_activity(ActivityType::Authentication, None);
        
        // Filter by type
        let auth_activities = activity.get_activities_by_type(&ActivityType::Authentication);
        assert_eq!(auth_activities.len(), 2);
        
        let api_activities = activity.get_activities_by_type(&ActivityType::ApiCall("test".to_string()));
        assert_eq!(api_activities.len(), 1);
    }
}