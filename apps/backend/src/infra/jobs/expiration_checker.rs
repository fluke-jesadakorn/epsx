use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use tracing::{info, error, warn};

use crate::app::ports::repositories::*;
use crate::dom::values::identifiers::UserId;
use crate::dom::entities::permission_profile::PermissionProfileId;
use super::notification_service::NotificationService;

pub struct ExpirationChecker {
    permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    user_repo: Arc<dyn UserRepo>,
    audit_repo: Arc<dyn AuditRepo>,
    notification_service: Arc<NotificationService>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExpirationCheckResult {
    pub processed_count: usize,
    pub expired_count: usize,
    pub notification_count: usize,
    pub grace_period_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExpiringAssignment {
    pub user_id: UserId,
    pub permission_profile_id: PermissionProfileId,
    pub expires_at: DateTime<Utc>,
    pub days_until_expiration: i64,
    pub profile_name: String,
    pub user_email: String,
}

impl ExpirationChecker {
    pub fn new(
        permission_profile_repo: Arc<dyn PermissionProfileRepo>,
        user_repo: Arc<dyn UserRepo>,
        audit_repo: Arc<dyn AuditRepo>,
        notification_service: Arc<NotificationService>,
    ) -> Self {
        Self {
            permission_profile_repo,
            user_repo,
            audit_repo,
            notification_service,
        }
    }

    pub async fn check_and_handle_expirations(&self) -> Result<ExpirationCheckResult, ExpirationError> {
        info!("Starting comprehensive expiration check");
        
        let mut result = ExpirationCheckResult {
            processed_count: 0,
            expired_count: 0,
            notification_count: 0,
            grace_period_count: 0,
            errors: Vec::new(),
        };

        // Get all assignments expiring in the next 30 days
        let expiring_assignments = self.get_expiring_assignments(30).await?;
        result.processed_count = expiring_assignments.len();
        
        info!("Found {} assignments to process for expiration", result.processed_count);

        for assignment in expiring_assignments {
            match self.process_expiring_assignment(&assignment, &mut result).await {
                Ok(()) => {},
                Err(e) => {
                    error!("Failed to process expiring assignment for user {}: {}", assignment.user_id.value(), e);
                    result.errors.push(format!("User {}: {}", assignment.user_id.value(), e));
                }
            }
        }

        // Log result summary
        if let Err(e) = self.audit_repo.log_system_event(
            "expiration_check_completed", 
            &serde_json::to_string(&result).unwrap_or_default()
        ).await {
            error!("Failed to log expiration check result: {}", e);
        }

        info!("Expiration check completed: {} processed, {} expired, {} notified, {} in grace period", 
              result.processed_count, result.expired_count, result.notification_count, result.grace_period_count);

        Ok(result)
    }

    async fn get_expiring_assignments(&self, days_ahead: i64) -> Result<Vec<ExpiringAssignment>, ExpirationError> {
        let cutoff_date = Utc::now() + Duration::days(days_ahead);
        
        let assignments = self.permission_profile_repo
            .find_assignments_expiring_before(cutoff_date)
            .await
            .map_err(|e| ExpirationError::DatabaseError(e.to_string()))?;

        let mut expiring_assignments = Vec::new();

        for assignment in assignments {
            // Get user details
            let user = match self.user_repo.find_by_id(&assignment.user_id).await {
                Ok(user) => user,
                Err(e) => {
                    error!("Failed to get user {}: {}", assignment.user_id.value(), e);
                    continue;
                }
            };

            // Get permission profile details
            let profile = match self.permission_profile_repo.find_by_id(&assignment.permission_profile_id).await {
                Ok(Some(profile)) => profile,
                Ok(None) => {
                    warn!("Permission profile not found: {}", assignment.permission_profile_id);
                    continue;
                },
                Err(e) => {
                    error!("Failed to get permission profile {}: {}", assignment.permission_profile_id, e);
                    continue;
                }
            };

            let days_until_expiration = if let Some(expires_at) = assignment.expires_at {
                (expires_at - Utc::now()).num_days()
            } else {
                // No expiration, skip this assignment
                continue;
            };

            expiring_assignments.push(ExpiringAssignment {
                user_id: assignment.user_id.clone(),
                permission_profile_id: assignment.permission_profile_id.clone(),
                expires_at: assignment.expires_at.unwrap_or_else(|| Utc::now()),
                days_until_expiration,
                profile_name: profile.name().to_string(),
                user_email: user.email().to_string(),
            });
        }

        Ok(expiring_assignments)
    }

    async fn process_expiring_assignment(
        &self, 
        assignment: &ExpiringAssignment, 
        result: &mut ExpirationCheckResult
    ) -> Result<(), ExpirationError> {
        
        if assignment.days_until_expiration <= 0 {
            // Already expired - remove assignment and notify
            self.handle_expired_assignment(assignment).await?;
            result.expired_count += 1;
        } else if assignment.days_until_expiration <= 3 {
            // Critical expiration warning (3 days or less)
            self.send_critical_expiration_warning(assignment).await?;
            result.notification_count += 1;
        } else if assignment.days_until_expiration <= 7 {
            // Warning expiration notification (7 days or less)
            self.send_expiration_warning(assignment).await?;
            result.notification_count += 1;
        } else if assignment.days_until_expiration <= 14 {
            // Advance notice (14 days or less)
            self.send_expiration_notice(assignment).await?;
            result.notification_count += 1;
        }

        // Check if this assignment should be extended (grace period logic)
        if assignment.days_until_expiration <= 7 {
            if let Ok(should_extend) = self.evaluate_grace_period_extension(assignment).await {
                if should_extend {
                    self.extend_assignment_grace_period(assignment).await?;
                    result.grace_period_count += 1;
                }
            }
        }

        Ok(())
    }

    async fn handle_expired_assignment(&self, assignment: &ExpiringAssignment) -> Result<(), ExpirationError> {
        info!("Processing expired assignment: user {} profile {}", assignment.user_id, assignment.profile_name);

        // Remove the expired assignment
        self.permission_profile_repo
            .revoke_assignment(&assignment.user_id, &assignment.permission_profile_id)
            .await
            .map_err(|e| ExpirationError::DatabaseError(e.to_string()))?;

        // Send expiration notification
        let message = format!(
            "Your {} permission profile has expired and has been automatically removed. \
            If you need continued access, please contact support or upgrade your plan.",
            assignment.profile_name
        );

        self.notification_service
            .send_user_notification(&assignment.user_email, "Permission Profile Expired", &message)
            .await
            .map_err(|e| ExpirationError::NotificationError(e.to_string()))?;

        // Log the expiration
        if let Err(e) = self.audit_repo
            .log_system_event(
                "permission_revoked", 
                &format!("User: {}, Profile: {}, Reason: Automatic expiration", 
                    assignment.user_id.value(), assignment.permission_profile_id.value())
            )
            .await {
            error!("Failed to log permission revocation: {}", e);
        }

        info!("Expired assignment processed successfully for user {}", assignment.user_id.value());
        Ok(())
    }

    async fn send_critical_expiration_warning(&self, assignment: &ExpiringAssignment) -> Result<(), ExpirationError> {
        let message = format!(
            "URGENT: Your {} permission profile will expire in {} day(s) on {}. \
            Please renew your subscription immediately to avoid service interruption. \
            Contact support if you need assistance.",
            assignment.profile_name,
            assignment.days_until_expiration,
            assignment.expires_at.format("%Y-%m-%d")
        );

        self.notification_service
            .send_user_notification(&assignment.user_email, "URGENT: Permission Profile Expiring Soon", &message)
            .await
            .map_err(|e| ExpirationError::NotificationError(e.to_string()))?;

        // Also send to admin for critical notifications
        let admin_message = format!(
            "Critical expiration alert: User {} ({}) has {} profile expiring in {} day(s)",
            assignment.user_email, assignment.user_id.value(), assignment.profile_name, assignment.days_until_expiration
        );

        self.notification_service
            .send_admin_notification("Critical Permission Expiration", &admin_message)
            .await
            .map_err(|e| ExpirationError::NotificationError(e.to_string()))?;

        Ok(())
    }

    async fn send_expiration_warning(&self, assignment: &ExpiringAssignment) -> Result<(), ExpirationError> {
        let message = format!(
            "Warning: Your {} permission profile will expire in {} day(s) on {}. \
            Please renew your subscription to continue enjoying premium features.",
            assignment.profile_name,
            assignment.days_until_expiration,
            assignment.expires_at.format("%Y-%m-%d")
        );

        self.notification_service
            .send_user_notification(&assignment.user_email, "Permission Profile Expiring Soon", &message)
            .await
            .map_err(|e| ExpirationError::NotificationError(e.to_string()))?;

        Ok(())
    }

    async fn send_expiration_notice(&self, assignment: &ExpiringAssignment) -> Result<(), ExpirationError> {
        let message = format!(
            "Notice: Your {} permission profile will expire in {} day(s) on {}. \
            Consider renewing your subscription to maintain access to premium features.",
            assignment.profile_name,
            assignment.days_until_expiration,
            assignment.expires_at.format("%Y-%m-%d")
        );

        self.notification_service
            .send_user_notification(&assignment.user_email, "Permission Profile Expiration Notice", &message)
            .await
            .map_err(|e| ExpirationError::NotificationError(e.to_string()))?;

        Ok(())
    }

    async fn evaluate_grace_period_extension(&self, assignment: &ExpiringAssignment) -> Result<bool, ExpirationError> {
        // Grace period logic: extend if user has been active and has a good payment history
        
        // For now, use simple logic - can be extended later
        let is_active = true; // Placeholder - would check user activity
        let has_good_payment_history = true; // Placeholder - would check payment history

        // Only extend for active users with good payment history
        let should_extend = is_active && has_good_payment_history;

        if should_extend {
            info!("Grace period extension approved for user {} profile {}", assignment.user_id.value(), assignment.profile_name);
        }

        Ok(should_extend)
    }

    async fn extend_assignment_grace_period(&self, assignment: &ExpiringAssignment) -> Result<(), ExpirationError> {
        let grace_period = Duration::days(7); // 7-day grace period
        let new_expiration = assignment.expires_at + grace_period;

        self.permission_profile_repo
            .extend_assignment_expiration(&assignment.user_id, &assignment.permission_profile_id, new_expiration)
            .await
            .map_err(|e| ExpirationError::DatabaseError(e.to_string()))?;

        // Notify user about grace period
        let message = format!(
            "Good news! Your {} permission profile has been extended by 7 days as a grace period. \
            New expiration date: {}. Please renew your subscription during this time.",
            assignment.profile_name,
            new_expiration.format("%Y-%m-%d")
        );

        self.notification_service
            .send_user_notification(&assignment.user_email, "Permission Profile Grace Period Extended", &message)
            .await
            .map_err(|e| ExpirationError::NotificationError(e.to_string()))?;

        // Log the extension
        if let Err(e) = self.audit_repo
            .log_system_event(
                "permission_extended", 
                &format!("User: {}, Profile: {}, Grace period extension: 7 days, new expiration: {}", 
                    assignment.user_id.value(), assignment.permission_profile_id.value(), new_expiration)
            )
            .await {
            error!("Failed to log permission extension: {}", e);
        }

        info!("Grace period extended for user {} profile {}", assignment.user_id.value(), assignment.profile_name);
        Ok(())
    }

    pub async fn check_specific_user_expirations(&self, user_id: &UserId) -> Result<Vec<ExpiringAssignment>, ExpirationError> {
        info!("Checking expirations for specific user: {}", user_id);
        
        let assignments = self.permission_profile_repo
            .find_user_assignments_with_expiration(user_id)
            .await
            .map_err(|e| ExpirationError::DatabaseError(e.to_string()))?;

        let mut expiring_assignments = Vec::new();

        for assignment in assignments {
            if let Some(expires_at) = assignment.expires_at {
                let days_until_expiration = (expires_at - Utc::now()).num_days();
                
                // Only include assignments expiring within 30 days
                if days_until_expiration <= 30 {
                    // Get user and profile details
                    if let (Ok(user), Ok(Some(profile))) = (
                        self.user_repo.find_by_id(user_id).await,
                        self.permission_profile_repo.find_by_id(&assignment.permission_profile_id).await
                    ) {
                        expiring_assignments.push(ExpiringAssignment {
                            user_id: user_id.clone(),
                            permission_profile_id: assignment.permission_profile_id.clone(),
                            expires_at,
                            days_until_expiration,
                            profile_name: profile.name().to_string(),
                            user_email: user.email().to_string(),
                        });
                    }
                }
            }
        }

        Ok(expiring_assignments)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExpirationError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Notification error: {0}")]
    NotificationError(String),
    
    #[error("Audit error: {0}")]
    AuditError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}