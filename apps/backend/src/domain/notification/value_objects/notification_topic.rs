use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::collections::HashSet;

/// Notification Topic Value Object
/// Represents email topics for broadcasting notifications to plans of users
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationTopic {
    name: String,
    display_name: String,
    description: Option<String>,
    category: TopicCategory,
    access_level: AccessLevel,
    target_permissions: HashSet<String>,
    is_active: bool,
}

impl NotificationTopic {
    /// Create new notification topic with validation
    pub fn new(
        name: String,
        display_name: String,
        description: Option<String>,
        category: TopicCategory,
    ) -> Result<Self, String> {
        let name = name.trim().to_lowercase();
        let display_name = display_name.trim().to_string();

        // Name validation
        if name.is_empty() {
            return Err("Topic name cannot be empty".to_string());
        }

        if name.len() > 100 {
            return Err("Topic name cannot exceed 100 characters".to_string());
        }

        if !Self::is_valid_topic_name(&name) {
            return Err("Topic name contains invalid characters. Use only letters, numbers, hyphens, and underscores".to_string());
        }

        // Display name validation
        if display_name.is_empty() {
            return Err("Topic display name cannot be empty".to_string());
        }

        if display_name.len() > 200 {
            return Err("Topic display name cannot exceed 200 characters".to_string());
        }

        // Description validation
        if let Some(ref desc) = description {
            if desc.len() > 500 {
                return Err("Topic description cannot exceed 500 characters".to_string());
            }
        }

        // Determine access level based on category
        let access_level = Self::default_access_level(&category);

        Ok(Self {
            name,
            display_name,
            description,
            category,
            access_level,
            target_permissions: HashSet::new(),
            is_active: true,
        })
    }

    /// Create system topic (high privilege)
    pub fn system_topic(name: String, display_name: String) -> Result<Self, String> {
        let mut topic = Self::new(name, display_name, None, TopicCategory::System)?;
        topic.access_level = AccessLevel::System;
        Ok(topic)
    }

    /// Create admin topic
    pub fn admin_topic(name: String, display_name: String, description: String) -> Result<Self, String> {
        let mut topic = Self::new(name, display_name, Some(description), TopicCategory::Administrative)?;
        topic.access_level = AccessLevel::Admin;
        topic.target_permissions.insert("admin:*:*".to_string());
        Ok(topic)
    }

    /// Create public topic (accessible to all users)
    pub fn public_topic(name: String, display_name: String, description: String) -> Result<Self, String> {
        Self::new(name, display_name, Some(description), TopicCategory::General)
    }

    /// Create broadcast topic (for system-wide announcements)
    pub fn broadcast_topic() -> Result<Self, String> {
        Self::system_topic("broadcast".to_string(), "System Broadcast".to_string())
    }

    /// Reconstruct topic from name (simplified reconstruction for database lookups)
    /// This is a temporary helper - full topics should be loaded from database
    pub fn from_name(name: String) -> Result<Self, String> {
        // Simplified reconstruction - in production, load full topic from database
        if name == "broadcast" {
            return Self::broadcast_topic();
        }

        // Default reconstruction for other topics
        Self::new(
            name.clone(),
            name.clone(), // Use name as display name temporarily
            None,
            TopicCategory::General,
        )
    }

    /// Get topic name (used for email topic name)
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Get description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get topic category
    pub fn category(&self) -> &TopicCategory {
        &self.category
    }

    /// Get access level
    pub fn access_level(&self) -> &AccessLevel {
        &self.access_level
    }

    /// Get target permissions
    pub fn target_permissions(&self) -> &HashSet<String> {
        &self.target_permissions
    }

    /// Check if topic is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Activate topic
    pub fn activate(&mut self) {
        self.is_active = true;
    }

    /// Deactivate topic
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Add required permission
    pub fn add_permission_requirement(&mut self, permission: String) -> Result<(), String> {
        if permission.is_empty() {
            return Err("Permission cannot be empty".to_string());
        }
        
        self.target_permissions.insert(permission);
        Ok(())
    }

    /// Remove permission requirement
    pub fn remove_permission_requirement(&mut self, permission: &str) {
        self.target_permissions.remove(permission);
    }

    /// Check if user with permissions can subscribe to this topic
    pub fn can_user_subscribe(&self, user_permissions: &HashSet<String>) -> bool {
        if !self.is_active {
            return false;
        }

        match self.access_level {
            AccessLevel::Public => true,
            AccessLevel::User => !user_permissions.is_empty(),
            AccessLevel::Premium => {
                user_permissions.iter().any(|p| {
                    p.contains("premium") || p.contains("gold") || p.contains("platinum")
                })
            }
            AccessLevel::Admin => {
                let perms: Vec<String> = user_permissions.iter().cloned().collect();
                epsx_contracts::permissions::has_admin_platform_permission(&perms)
            }
            AccessLevel::System => false, // System topics are not user-subscribable
        }
    }

    /// Check if topic matches notification type and priority
    pub fn is_suitable_for_notification(&self, notification_type: &str, _priority: &str) -> bool {
        match self.category {
            TopicCategory::System => notification_type == "system",
            TopicCategory::Security => notification_type == "security",
            TopicCategory::Administrative => notification_type == "admin",
            TopicCategory::Marketing => notification_type == "marketing",
            TopicCategory::Feature => notification_type == "feature",
            TopicCategory::General => true, // General topics accept all notifications
            TopicCategory::UserSegment => true,
        }
    }

    /// Get estimated subscriber count category
    pub fn estimated_subscriber_scale(&self) -> SubscriberScale {
        match self.category {
            TopicCategory::System | TopicCategory::Administrative => SubscriberScale::Small,
            TopicCategory::Security => SubscriberScale::Medium,
            TopicCategory::Feature => SubscriberScale::Large,
            TopicCategory::Marketing | TopicCategory::General => SubscriberScale::Large,
            TopicCategory::UserSegment => SubscriberScale::Medium,
        }
    }

    /// Get email-compliant topic name
    pub fn email_topic_name(&self) -> String {
        // Email topic names must be safe for email headers
        let mut email_name = self.name.clone();
        
        // Replace invalid characters
        email_name = email_name.replace(' ', "_");
        email_name = email_name.chars()
            .map(|c| if c.is_alphanumeric() || "-.~_%".contains(c) { c } else { '_' })
            .collect();
            
        // Ensure it doesn't start with a number or special character
        if email_name.chars().next().is_none_or(|c| !c.is_ascii_alphabetic()) {
            email_name = format!("topic_{}", email_name);
        }
        
        email_name
    }

    /// Validate topic name format
    fn is_valid_topic_name(name: &str) -> bool {
        if name.is_empty() || name.len() > 100 {
            return false;
        }

        // Must start with letter
        if !name.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
            return false;
        }

        // Only lowercase letters, numbers, hyphens, underscores
        name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    }

    /// Get default access level for category
    fn default_access_level(category: &TopicCategory) -> AccessLevel {
        match category {
            TopicCategory::System => AccessLevel::System,
            TopicCategory::Security => AccessLevel::User,
            TopicCategory::Administrative => AccessLevel::Admin,
            TopicCategory::Marketing => AccessLevel::Public,
            TopicCategory::Feature => AccessLevel::User,
            TopicCategory::General => AccessLevel::Public,
            TopicCategory::UserSegment => AccessLevel::User,
        }
    }
}

/// Topic categories for organization and routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopicCategory {
    System,         // System-wide notifications
    Security,       // Security alerts and updates
    Administrative, // Admin-only notifications
    Marketing,      // Marketing and promotional content
    Feature,        // Feature updates and announcements
    General,        // General notifications
    UserSegment,    // Specific user segment targeting
}

impl TopicCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TopicCategory::System => "system",
            TopicCategory::Security => "security",
            TopicCategory::Administrative => "administrative",
            TopicCategory::Marketing => "marketing",
            TopicCategory::Feature => "feature",
            TopicCategory::General => "general",
            TopicCategory::UserSegment => "user_segment",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TopicCategory::System => "System",
            TopicCategory::Security => "Security",
            TopicCategory::Administrative => "Administrative",
            TopicCategory::Marketing => "Marketing",
            TopicCategory::Feature => "Features",
            TopicCategory::General => "General",
            TopicCategory::UserSegment => "User Segment",
        }
    }
}

/// Access levels for topic subscription
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    Public,  // Anyone can subscribe
    User,    // Authenticated users only
    Premium, // Premium users only
    Admin,   // Admin users only
    System,  // System use only (not user-subscribable)
}

impl AccessLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessLevel::Public => "public",
            AccessLevel::User => "user",
            AccessLevel::Premium => "premium",
            AccessLevel::Admin => "admin",
            AccessLevel::System => "system",
        }
    }
}

/// Expected subscriber scale for capacity planning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriberScale {
    Small,  // < 1000 subscribers
    Medium, // 1000-10000 subscribers
    Large,  // > 10000 subscribers
}

impl SubscriberScale {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriberScale::Small => "small",
            SubscriberScale::Medium => "medium",
            SubscriberScale::Large => "large",
        }
    }
}

impl Display for NotificationTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.display_name, self.name)
    }
}

impl TryFrom<(String, String)> for NotificationTopic {
    type Error = String;

    fn try_from((name, display_name): (String, String)) -> Result<Self, Self::Error> {
        NotificationTopic::new(name, display_name, None, TopicCategory::General)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_topic() {
        let topic = NotificationTopic::new(
            "test_topic".to_string(),
            "Test Topic".to_string(),
            Some("A test topic".to_string()),
            TopicCategory::General,
        ).unwrap();

        assert_eq!(topic.name(), "test_topic");
        assert_eq!(topic.display_name(), "Test Topic");
        assert!(topic.is_active());
    }

    #[test]
    fn test_invalid_topic_names() {
        // Empty name
        assert!(NotificationTopic::new(
            "".to_string(),
            "Display".to_string(),
            None,
            TopicCategory::General,
        ).is_err());

        // Name starting with number
        assert!(NotificationTopic::new(
            "123invalid".to_string(),
            "Display".to_string(),
            None,
            TopicCategory::General,
        ).is_err());

        // Invalid characters
        assert!(NotificationTopic::new(
            "invalid topic!".to_string(),
            "Display".to_string(),
            None,
            TopicCategory::General,
        ).is_err());
    }

    #[test]
    fn test_system_topic() {
        let topic = NotificationTopic::system_topic(
            "system_alerts".to_string(),
            "System Alerts".to_string(),
        ).unwrap();

        assert_eq!(topic.access_level(), &AccessLevel::System);
        assert_eq!(topic.category(), &TopicCategory::System);
    }

    #[test]
    fn test_admin_topic() {
        let topic = NotificationTopic::admin_topic(
            "admin_notifications".to_string(),
            "Admin Notifications".to_string(),
            "Administrative updates".to_string(),
        ).unwrap();

        assert_eq!(topic.access_level(), &AccessLevel::Admin);
        assert!(topic.target_permissions().contains("admin:*:*"));
    }

    #[test]
    fn test_user_subscription_access() {
        let public_topic = NotificationTopic::public_topic(
            "public_news".to_string(),
            "Public News".to_string(),
            "Public announcements".to_string(),
        ).unwrap();

        let admin_topic = NotificationTopic::admin_topic(
            "admin_alerts".to_string(),
            "Admin Alerts".to_string(),
            "Admin notifications".to_string(),
        ).unwrap();

        let user_permissions = vec!["user:profile:read".to_string()].into_iter().collect();
        let admin_permissions = vec!["admin:users:manage".to_string()].into_iter().collect();

        assert!(public_topic.can_user_subscribe(&user_permissions));
        assert!(public_topic.can_user_subscribe(&admin_permissions));
        
        assert!(!admin_topic.can_user_subscribe(&user_permissions));
        assert!(admin_topic.can_user_subscribe(&admin_permissions));
    }

    #[test]
    fn test_email_topic_name() {
        let topic = NotificationTopic::new(
            "test_topic-123".to_string(),
            "Test Topic".to_string(),
            None,
            TopicCategory::General,
        ).unwrap();

        let email_name = topic.email_topic_name();
        assert!(email_name.chars().all(|c| c.is_alphanumeric() || "-.~_%".contains(c)));
        assert!(email_name.chars().next().unwrap().is_ascii_alphabetic());
    }

    #[test]
    fn test_permission_management() {
        let mut topic = NotificationTopic::new(
            "premium_features".to_string(),
            "Premium Features".to_string(),
            None,
            TopicCategory::Feature,
        ).unwrap();

        topic.add_permission_requirement("premium:*:*".to_string()).unwrap();
        topic.add_permission_requirement("gold:*:*".to_string()).unwrap();

        assert_eq!(topic.target_permissions().len(), 2);
        assert!(topic.target_permissions().contains("premium:*:*"));

        topic.remove_permission_requirement("gold:*:*");
        assert_eq!(topic.target_permissions().len(), 1);
    }

    #[test]
    fn test_notification_suitability() {
        let system_topic = NotificationTopic::system_topic(
            "system_maintenance".to_string(),
            "System Maintenance".to_string(),
        ).unwrap();

        let marketing_topic = NotificationTopic::new(
            "promotions".to_string(),
            "Promotions".to_string(),
            None,
            TopicCategory::Marketing,
        ).unwrap();

        assert!(system_topic.is_suitable_for_notification("system", "high"));
        assert!(!system_topic.is_suitable_for_notification("marketing", "normal"));

        assert!(marketing_topic.is_suitable_for_notification("marketing", "low"));
        assert!(!marketing_topic.is_suitable_for_notification("system", "urgent"));
    }

    #[test]
    fn test_subscriber_scale() {
        let system_topic = NotificationTopic::system_topic(
            "system_alerts".to_string(),
            "System Alerts".to_string(),
        ).unwrap();

        let marketing_topic = NotificationTopic::new(
            "promotions".to_string(),
            "Promotions".to_string(),
            None,
            TopicCategory::Marketing,
        ).unwrap();

        assert_eq!(system_topic.estimated_subscriber_scale(), SubscriberScale::Small);
        assert_eq!(marketing_topic.estimated_subscriber_scale(), SubscriberScale::Large);
    }
}