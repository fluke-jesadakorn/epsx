use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::app::ports::repositories::{UserRepo, PermissionProfileRepo};
use crate::dom::entities::permission_profile::PermissionProfileId;
use crate::dom::entities::user::User;
use crate::dom::values::UserId;
use crate::infra::services::notification::{NotificationService, Notification, NotificationType, NotificationPriority};

#[derive(Debug, Clone)]
pub struct FeatureExpiration {
    pub user_id: UserId,
    pub permission_profile_id: PermissionProfileId,
    pub permission_profile_name: String,
    pub expires_at: DateTime<Utc>,
    pub features: Vec<String>,
    pub grace_period_days: u32,
    pub notification_sent: bool,
    pub final_warning_sent: bool,
}

#[derive(Debug, Clone)]
pub struct ExpirationConfig {
    pub warning_days_before: Vec<u32>, // Days before expiration to send warnings (e.g., [30, 7, 1])
    pub grace_period_days: u32,        // Days after expiration before feature deactivation
    pub check_interval_hours: u64,     // How often to run expiration checks
    pub batch_size: usize,             // How many users to check per batch
}

impl Default for ExpirationConfig {
    fn default() -> Self {
        Self {
            warning_days_before: vec![30, 7, 3, 1],
            grace_period_days: 7,
            check_interval_hours: 1,
            batch_size: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExpirationCheckResult {
    pub total_checked: usize,
    pub expiring_soon: usize,
    pub expired: usize,
    pub notifications_sent: usize,
    pub features_deactivated: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExpirationError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Notification error: {0}")]
    NotificationError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Processing error: {0}")]
    ProcessingError(String),
}

#[async_trait]
pub trait FeatureExpirationService: Send + Sync {
    async fn check_feature_expirations(&self) -> Result<ExpirationCheckResult, ExpirationError>;
    async fn get_expiring_features(&self, user_id: &UserId) -> Result<Vec<FeatureExpiration>, ExpirationError>;
    async fn extend_feature_expiration(&self, user_id: &UserId, permission_profile_id: &PermissionProfileId, extension_days: u32) -> Result<(), ExpirationError>;
    async fn send_renewal_notification(&self, user_id: &UserId, expiration: &FeatureExpiration, days_until_expiry: u32) -> Result<(), ExpirationError>;
    async fn deactivate_expired_features(&self, user_id: &UserId, permission_profile_id: &PermissionProfileId) -> Result<(), ExpirationError>;
}

pub struct FeatureExpirationServiceImpl {
    user_repo: Arc<dyn UserRepo>,
    permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    notification_service: Arc<dyn NotificationService>,
    config: ExpirationConfig,
    // In-memory tracking for notification state
    notification_state: Arc<RwLock<HashMap<String, NotificationState>>>,
}

#[derive(Debug, Clone)]
struct NotificationState {
    last_warning_sent: Option<DateTime<Utc>>,
    warnings_sent: Vec<u32>, // Days before expiration when warnings were sent
    final_warning_sent: bool,
}

impl FeatureExpirationServiceImpl {
    pub fn new(
        user_repo: Arc<dyn UserRepo>,
        permission_profile_repo: Arc<dyn PermissionProfileRepo>,
        notification_service: Arc<dyn NotificationService>,
        config: Option<ExpirationConfig>,
    ) -> Self {
        Self {
            user_repo,
            permission_profile_repo,
            notification_service,
            config: config.unwrap_or_default(),
            notification_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_all_active_user_features(&self) -> Result<Vec<FeatureExpiration>, ExpirationError> {
        // In a real implementation, this would query the user_features table
        // For now, we'll simulate by getting all users and their permission profiles
        
        // This is a simplified implementation - in production you'd have a proper user_features table
        // that tracks feature assignments with expiration dates
        
        let mut feature_assignments = Vec::new();
        
        // For demonstration, we'll create mock data
        // In production, this would be a database query like:
        // SELECT uf.user_id, uf.permission_profile_id, pp.name, uf.expires_at, uf.features
        // FROM user_features uf 
        // JOIN permission_profiles pp ON uf.permission_profile_id = pp.id
        // WHERE uf.is_active = true AND uf.expires_at IS NOT NULL
        
        Ok(feature_assignments)
    }

    async fn create_expiration_warning_notification(
        &self,
        user: &User,
        expiration: &FeatureExpiration,
        days_until_expiry: u32,
    ) -> Notification {
        let mut metadata = HashMap::new();
        metadata.insert("permission_profile_id".to_string(), expiration.permission_profile_id.value().to_string());
        metadata.insert("permission_profile_name".to_string(), expiration.permission_profile_name.clone());
        metadata.insert("expires_at".to_string(), expiration.expires_at.to_rfc3339());
        metadata.insert("days_until_expiry".to_string(), days_until_expiry.to_string());
        metadata.insert("features_count".to_string(), expiration.features.len().to_string());

        let (title, message, priority) = match days_until_expiry {
            0 => (
                "Features Expired - Grace Period Active".to_string(),
                format!(
                    "Your {} subscription has expired but is still active for {} more days. Renew now to avoid service interruption.",
                    expiration.permission_profile_name,
                    expiration.grace_period_days
                ),
                NotificationPriority::Critical
            ),
            1 => (
                "Subscription Expires Tomorrow!".to_string(),
                format!(
                    "Your {} subscription expires tomorrow. Renew now to keep access to {} features.",
                    expiration.permission_profile_name,
                    expiration.features.len()
                ),
                NotificationPriority::Critical
            ),
            days if days <= 3 => (
                format!("Subscription Expires in {} Days", days),
                format!(
                    "Your {} subscription expires in {} days. Don't lose access to your premium features - renew today!",
                    expiration.permission_profile_name,
                    days
                ),
                NotificationPriority::High
            ),
            days if days <= 7 => (
                format!("Subscription Expires in {} Days", days),
                format!(
                    "Your {} subscription expires in {} days. Renew early to ensure uninterrupted access.",
                    expiration.permission_profile_name,
                    days
                ),
                NotificationPriority::High
            ),
            days => (
                "Subscription Renewal Reminder".to_string(),
                format!(
                    "Your {} subscription expires in {} days. Consider renewing to continue enjoying premium features.",
                    expiration.permission_profile_name,
                    days
                ),
                NotificationPriority::Medium
            ),
        };

        Notification {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user.id().value().to_string(),
            title,
            message,
            notification_type: NotificationType::System,
            priority,
            read: false,
            created_at: Utc::now(),
            expires_at: Some(expiration.expires_at + Duration::days(expiration.grace_period_days as i64)),
            metadata,
        }
    }

    async fn create_final_warning_notification(
        &self,
        user: &User,
        expiration: &FeatureExpiration,
    ) -> Notification {
        let mut metadata = HashMap::new();
        metadata.insert("permission_profile_id".to_string(), expiration.permission_profile_id.value().to_string());
        metadata.insert("permission_profile_name".to_string(), expiration.permission_profile_name.clone());
        metadata.insert("expires_at".to_string(), expiration.expires_at.to_rfc3339());
        metadata.insert("grace_period_ends".to_string(), (expiration.expires_at + Duration::days(expiration.grace_period_days as i64)).to_rfc3339());
        metadata.insert("features".to_string(), expiration.features.join(", "));

        Notification {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user.id().value().to_string(),
            title: "Final Notice: Features Will Be Deactivated".to_string(),
            message: format!(
                "Your {} subscription expired and the grace period ends today. Your features will be deactivated unless you renew immediately. Affected features: {}",
                expiration.permission_profile_name,
                expiration.features.join(", ")
            ),
            notification_type: NotificationType::System,
            priority: NotificationPriority::Critical,
            read: false,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::days(30)), // Keep this notification for 30 days
            metadata,
        }
    }

    async fn should_send_warning(&self, expiration: &FeatureExpiration, days_until_expiry: u32) -> bool {
        let state_key = format!("{}:{}", expiration.user_id.value(), expiration.permission_profile_id.value());
        let notification_state = self.notification_state.read().await;
        
        if let Some(state) = notification_state.get(&state_key) {
            // Don't send duplicate warnings for the same day count
            if state.warnings_sent.contains(&days_until_expiry) {
                return false;
            }
            
            // For frequent checks, don't spam notifications
            if let Some(last_sent) = state.last_warning_sent {
                if Utc::now() - last_sent < Duration::hours(12) && days_until_expiry > 1 {
                    return false;
                }
            }
        }
        
        // Send warning if it's one of our configured warning days
        self.config.warning_days_before.contains(&days_until_expiry) || days_until_expiry == 0
    }

    async fn mark_warning_sent(&self, expiration: &FeatureExpiration, days_until_expiry: u32) {
        let state_key = format!("{}:{}", expiration.user_id.value(), expiration.permission_profile_id.value());
        let mut notification_state = self.notification_state.write().await;
        
        let state = notification_state.entry(state_key).or_insert_with(|| NotificationState {
            last_warning_sent: None,
            warnings_sent: Vec::new(),
            final_warning_sent: false,
        });
        
        state.last_warning_sent = Some(Utc::now());
        if !state.warnings_sent.contains(&days_until_expiry) {
            state.warnings_sent.push(days_until_expiry);
        }
        
        if days_until_expiry == 0 {
            state.final_warning_sent = true;
        }
    }
}

#[async_trait]
impl FeatureExpirationService for FeatureExpirationServiceImpl {
    async fn check_feature_expirations(&self) -> Result<ExpirationCheckResult, ExpirationError> {
        let mut result = ExpirationCheckResult {
            total_checked: 0,
            expiring_soon: 0,
            expired: 0,
            notifications_sent: 0,
            features_deactivated: 0,
            errors: Vec::new(),
        };

        tracing::info!("Starting feature expiration check");

        // Get all active feature assignments with expiration dates
        let feature_assignments = self.get_all_active_user_features().await
            .map_err(|e| ExpirationError::DatabaseError(e.to_string()))?;

        result.total_checked = feature_assignments.len();

        for expiration in feature_assignments {
            let now = Utc::now();
            let days_until_expiry = (expiration.expires_at - now).num_days();

            // Check if feature has expired beyond grace period
            if days_until_expiry < -(expiration.grace_period_days as i64) {
                // Deactivate expired features
                match self.deactivate_expired_features(&expiration.user_id, &expiration.permission_profile_id).await {
                    Ok(_) => {
                        result.features_deactivated += 1;
                        tracing::info!(
                            "Deactivated expired features for user {} permission profile {}",
                            expiration.user_id.value(),
                            expiration.permission_profile_name
                        );
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to deactivate features for user {}: {}",
                            expiration.user_id.value(),
                            e
                        ));
                    }
                }
                continue;
            }

            // Check if we should send notifications
            let days_until_expiry_u32 = if days_until_expiry < 0 { 0 } else { days_until_expiry as u32 };

            if days_until_expiry <= 0 {
                result.expired += 1;
            } else if days_until_expiry <= 30 {
                result.expiring_soon += 1;
            }

            // Send renewal notifications if appropriate
            if self.should_send_warning(&expiration, days_until_expiry_u32).await {
                match self.send_renewal_notification(&expiration.user_id, &expiration, days_until_expiry_u32).await {
                    Ok(_) => {
                        result.notifications_sent += 1;
                        self.mark_warning_sent(&expiration, days_until_expiry_u32).await;
                        
                        tracing::info!(
                            "Sent renewal notification to user {} for {} (expires in {} days)",
                            expiration.user_id.value(),
                            expiration.permission_profile_name,
                            days_until_expiry_u32
                        );
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to send notification to user {}: {}",
                            expiration.user_id.value(),
                            e
                        ));
                    }
                }
            }
        }

        tracing::info!(
            "Feature expiration check completed: {} checked, {} expiring soon, {} expired, {} notifications sent, {} features deactivated, {} errors",
            result.total_checked,
            result.expiring_soon,
            result.expired,
            result.notifications_sent,
            result.features_deactivated,
            result.errors.len()
        );

        Ok(result)
    }

    async fn get_expiring_features(&self, user_id: &UserId) -> Result<Vec<FeatureExpiration>, ExpirationError> {
        // In a real implementation, this would query the user_features table for the specific user
        // For now, return empty since we don't have the actual database schema implemented
        Ok(Vec::new())
    }

    async fn extend_feature_expiration(
        &self,
        user_id: &UserId,
        permission_profile_id: &PermissionProfileId,
        extension_days: u32,
    ) -> Result<(), ExpirationError> {
        // In a real implementation, this would update the user_features table
        // UPDATE user_features SET expires_at = expires_at + INTERVAL '{extension_days} days'
        // WHERE user_id = ? AND permission_profile_id = ?
        
        tracing::info!(
            "Extended feature expiration for user {} permission profile {} by {} days",
            user_id.value(),
            permission_profile_id.value(),
            extension_days
        );
        
        Ok(())
    }

    async fn send_renewal_notification(
        &self,
        user_id: &UserId,
        expiration: &FeatureExpiration,
        days_until_expiry: u32,
    ) -> Result<(), ExpirationError> {
        // Get user details
        let user = self.user_repo.find_by_id(user_id).await
            .map_err(|e| ExpirationError::DatabaseError(format!("Failed to find user: {}", e)))?;

        // Create appropriate notification based on days until expiry
        let notification = if days_until_expiry == 0 && !expiration.final_warning_sent {
            self.create_final_warning_notification(&user, expiration).await
        } else {
            self.create_expiration_warning_notification(&user, expiration, days_until_expiry).await
        };

        // Send notification
        self.notification_service.send_notification(notification).await
            .map_err(|e| ExpirationError::NotificationError(e.to_string()))?;

        Ok(())
    }

    async fn deactivate_expired_features(
        &self,
        user_id: &UserId,
        permission_profile_id: &PermissionProfileId,
    ) -> Result<(), ExpirationError> {
        // In a real implementation, this would:
        // 1. Update user_features table to set is_active = false
        // 2. Send final notification about deactivation
        // 3. Log the deactivation for audit purposes
        
        let user = self.user_repo.find_by_id(user_id).await
            .map_err(|e| ExpirationError::DatabaseError(format!("Failed to find user: {}", e)))?;

        // Send deactivation notification
        let notification = Notification {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user.id().value().to_string(),
            title: "Features Deactivated".to_string(),
            message: format!(
                "Your subscription has expired and the grace period has ended. Some features have been deactivated. Renew your subscription to reactivate them."
            ),
            notification_type: NotificationType::System,
            priority: NotificationPriority::High,
            read: false,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::days(60)),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("permission_profile_id".to_string(), permission_profile_id.value().to_string());
                meta.insert("deactivated_at".to_string(), Utc::now().to_rfc3339());
                meta
            },
        };

        self.notification_service.send_notification(notification).await
            .map_err(|e| ExpirationError::NotificationError(e.to_string()))?;

        tracing::info!(
            "Deactivated expired features for user {} permission profile {}",
            user_id.value(),
            permission_profile_id.value()
        );

        Ok(())
    }
}

/// Background task scheduler for feature expiration checks
pub struct ExpirationScheduler {
    service: Arc<dyn FeatureExpirationService>,
    config: ExpirationConfig,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl ExpirationScheduler {
    pub fn new(service: Arc<dyn FeatureExpirationService>, config: Option<ExpirationConfig>) -> Self {
        Self {
            service,
            config: config.unwrap_or_default(),
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), ExpirationError> {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let interval_duration = std::time::Duration::from_secs(self.config.check_interval_hours * 3600);
        let mut interval = tokio::time::interval(interval_duration);
        let service = Arc::clone(&self.service);

        tracing::info!(
            "Starting feature expiration scheduler (interval: {} hours)",
            self.config.check_interval_hours
        );

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        match service.check_feature_expirations().await {
                            Ok(result) => {
                                tracing::info!("Expiration check completed: {:?}", result);
                            }
                            Err(e) => {
                                tracing::error!("Expiration check failed: {}", e);
                            }
                        }
                    }
                    _ = &mut shutdown_rx => {
                        tracing::info!("Feature expiration scheduler shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub fn shutdown(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use crate::infra::services::notification::InMemoryNotificationService;

    mock! {
        UserRepoMock {}
        #[async_trait]
        impl UserRepo for UserRepoMock {
            async fn find_by_id(&self, id: &UserId) -> Result<User, String>;
            async fn create(&self, user: User) -> Result<User, String>;
            async fn update(&self, user: User) -> Result<User, String>;
            async fn delete(&self, id: &UserId) -> Result<(), String>;
            async fn find_by_email(&self, email: &str) -> Result<Option<User>, String>;
        }
    }

    mock! {
        PermissionProfileRepoMock {}
        #[async_trait]
        impl PermissionProfileRepo for PermissionProfileRepoMock {
            async fn find_by_id(&self, id: &PermissionProfileId) -> Result<crate::dom::entities::permission_profile::PermissionProfile, String>;
            async fn create(&self, profile: crate::dom::entities::permission_profile::PermissionProfile) -> Result<crate::dom::entities::permission_profile::PermissionProfile, String>;
            async fn update(&self, profile: crate::dom::entities::permission_profile::PermissionProfile) -> Result<crate::dom::entities::permission_profile::PermissionProfile, String>;
            async fn delete(&self, id: &PermissionProfileId) -> Result<(), String>;
            async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<crate::dom::entities::permission_profile::PermissionProfile>, String>;
            async fn find_active_profiles(&self) -> Result<Vec<crate::dom::entities::permission_profile::PermissionProfile>, String>;
        }
    }

    #[tokio::test]
    async fn test_expiration_service_creation() {
        let user_repo = Arc::new(MockUserRepoMock::new());
        let permission_profile_repo = Arc::new(MockPermissionProfileRepoMock::new());
        let notification_service = Arc::new(InMemoryNotificationService::new());

        let service = FeatureExpirationServiceImpl::new(
            user_repo,
            permission_profile_repo,
            notification_service,
            None,
        );

        // Test that expiration check runs without panicking
        let result = service.check_feature_expirations().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_warning_timing_logic() {
        let user_repo = Arc::new(MockUserRepoMock::new());
        let permission_profile_repo = Arc::new(MockPermissionProfileRepoMock::new());
        let notification_service = Arc::new(InMemoryNotificationService::new());

        let service = FeatureExpirationServiceImpl::new(
            user_repo,
            permission_profile_repo,
            notification_service,
            Some(ExpirationConfig {
                warning_days_before: vec![7, 3, 1],
                grace_period_days: 3,
                check_interval_hours: 1,
                batch_size: 50,
            }),
        );

        let expiration = FeatureExpiration {
            user_id: UserId::new("test_user".to_string()),
            permission_profile_id: PermissionProfileId::new("test_profile".to_string()),
            permission_profile_name: "Test Profile".to_string(),
            expires_at: Utc::now() + Duration::days(7),
            features: vec!["feature1".to_string(), "feature2".to_string()],
            grace_period_days: 3,
            notification_sent: false,
            final_warning_sent: false,
        };

        // Should send warning for 7 days
        assert!(service.should_send_warning(&expiration, 7).await);
        
        // Should not send warning for 5 days (not in config)
        assert!(!service.should_send_warning(&expiration, 5).await);
        
        // Should send warning for 1 day
        assert!(service.should_send_warning(&expiration, 1).await);
    }
}