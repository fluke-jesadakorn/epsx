use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::app::ports::repositories::*;
use super::notification_service::NotificationService;

pub struct ExpirationChecker {
    permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    user_repo: Arc<dyn UserRepo>,
    audit_repo: Arc<dyn AuditRepo>,
    notification_service: Arc<NotificationService>,
}

#[derive(Debug)]
pub struct ExpirationCheckResult {
    pub processed_count: usize,
    pub expired_count: usize,
    pub notification_count: usize,
    pub grace_period_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub struct ExpiringAssignment {
    pub user_id: Uuid,
    pub permission_profile_id: Uuid,
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
                    error!("Failed to process expiring assignment for user {}: {}", assignment.user_id, e);
                    result.errors.push(format!("User {}: {}", assignment.user_id, e));
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
            .map_err(ExpirationError::DatabaseError)?;

        let mut expiring_assignments = Vec::new();

        for assignment in assignments {
            // Get user details
            let user = match self.user_repo.find_by_id(&assignment.user_id).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    warn!("User not found for assignment: {}", assignment.user_id);
                    continue;
                },
                Err(e) => {
                    error!("Failed to get user {}: {}", assignment.user_id, e);
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

            let days_until_expiration = (assignment.expires_at - Utc::now()).num_days();

            expiring_assignments.push(ExpiringAssignment {
                user_id: assignment.user_id,
                permission_profile_id: assignment.permission_profile_id,
                expires_at: assignment.expires_at,
                days_until_expiration,
                profile_name: profile.name,
                user_email: user.email,
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
            .map_err(ExpirationError::DatabaseError)?;

        // Send expiration notification
        let message = format!(
            "Your {} permission profile has expired and has been automatically removed. \
            If you need continued access, please contact support or upgrade your plan.",
            assignment.profile_name
        );

        self.notification_service
            .send_user_notification(&assignment.user_email, "Permission Profile Expired", &message)
            .await
            .map_err(ExpirationError::NotificationError)?;

        // Log the expiration
        self.audit_repo
            .log_permission_revocation(
                &assignment.user_id, 
                &assignment.permission_profile_id, 
                "expiration_checker", 
                "Automatic expiration"
            )
            .await
            .map_err(ExpirationError::AuditError)?;

        info!("Expired assignment processed successfully for user {}", assignment.user_id);
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
            .map_err(ExpirationError::NotificationError)?;

        // Also send to admin for critical notifications
        let admin_message = format!(
            "Critical expiration alert: User {} ({}) has {} profile expiring in {} day(s)",
            assignment.user_email, assignment.user_id, assignment.profile_name, assignment.days_until_expiration
        );

        self.notification_service
            .send_admin_notification("Critical Permission Expiration", &admin_message)
            .await
            .map_err(ExpirationError::NotificationError)?;

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
            .map_err(ExpirationError::NotificationError)?;

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
            .map_err(ExpirationError::NotificationError)?;

        Ok(())
    }

    async fn evaluate_grace_period_extension(&self, assignment: &ExpiringAssignment) -> Result<bool, ExpirationError> {
        // Grace period logic: extend if user has been active and has a good payment history
        
        // Check if user has been active recently (last 7 days)
        let is_active = self.user_repo
            .is_user_active_since(&assignment.user_id, Utc::now() - Duration::days(7))
            .await
            .map_err(ExpirationError::DatabaseError)?;

        // Check if user has a good payment history (no failed payments in last 90 days)
        let has_good_payment_history = self.user_repo
            .has_good_payment_history(&assignment.user_id, 90)
            .await
            .unwrap_or(false); // Default to false if check fails

        // Only extend for active users with good payment history
        let should_extend = is_active && has_good_payment_history;

        if should_extend {
            info!("Grace period extension approved for user {} profile {}", assignment.user_id, assignment.profile_name);
        }

        Ok(should_extend)
    }

    async fn extend_assignment_grace_period(&self, assignment: &ExpiringAssignment) -> Result<(), ExpirationError> {
        let grace_period = Duration::days(7); // 7-day grace period
        let new_expiration = assignment.expires_at + grace_period;

        self.permission_profile_repo
            .extend_assignment_expiration(&assignment.user_id, &assignment.permission_profile_id, new_expiration)
            .await
            .map_err(ExpirationError::DatabaseError)?;

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
            .map_err(ExpirationError::NotificationError)?;

        // Log the extension
        self.audit_repo
            .log_permission_assignment(
                &assignment.user_id, 
                &assignment.permission_profile_id, 
                "expiration_checker", 
                &format!("Grace period extension: 7 days, new expiration: {}", new_expiration)
            )
            .await
            .map_err(ExpirationError::AuditError)?;

        info!("Grace period extended for user {} profile {}", assignment.user_id, assignment.profile_name);
        Ok(())
    }

    pub async fn check_specific_user_expirations(&self, user_id: &Uuid) -> Result<Vec<ExpiringAssignment>, ExpirationError> {
        info!("Checking expirations for specific user: {}", user_id);
        
        let assignments = self.permission_profile_repo
            .find_user_assignments_with_expiration(user_id)
            .await
            .map_err(ExpirationError::DatabaseError)?;

        let mut expiring_assignments = Vec::new();

        for assignment in assignments {
            if let Some(expires_at) = assignment.expires_at {
                let days_until_expiration = (expires_at - Utc::now()).num_days();
                
                // Only include assignments expiring within 30 days
                if days_until_expiration <= 30 {
                    // Get user and profile details
                    if let (Ok(Some(user)), Ok(Some(profile))) = (
                        self.user_repo.find_by_id(user_id).await,
                        self.permission_profile_repo.find_by_id(&assignment.permission_profile_id).await
                    ) {
                        expiring_assignments.push(ExpiringAssignment {
                            user_id: *user_id,
                            permission_profile_id: assignment.permission_profile_id,
                            expires_at,
                            days_until_expiration,
                            profile_name: profile.name,
                            user_email: user.email,
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