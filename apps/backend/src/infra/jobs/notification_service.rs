use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::app::ports::repositories::AuditRepo;

pub struct NotificationService {
    email_service: Box<dyn EmailProvider>,
    sms_service: Option<Box<dyn SmsProvider>>,
    audit_repo: Box<dyn AuditRepo>,
    config: NotificationConfig,
}

#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub admin_emails: Vec<String>,
    pub from_email: String,
    pub from_name: String,
    pub enable_email: bool,
    pub enable_sms: bool,
    pub rate_limit_per_minute: u32,
    pub template_base_url: Option<String>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            admin_emails: vec!["admin@company.com".to_string()],
            from_email: "noreply@company.com".to_string(),
            from_name: "EPSX System".to_string(),
            enable_email: true,
            enable_sms: false,
            rate_limit_per_minute: 60,
            template_base_url: None,
        }
    }
}

#[derive(Debug)]
pub struct NotificationTemplate {
    pub subject_template: String,
    pub body_template: String,
    pub template_type: NotificationType,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    PermissionExpiring,
    PermissionExpired,
    PermissionAssigned,
    PermissionRevoked,
    AdminAlert,
    SystemNotification,
    GracePeriodExtended,
}

#[derive(Debug)]
pub struct NotificationResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
}

impl NotificationService {
    pub fn new(
        email_service: Box<dyn EmailProvider>,
        sms_service: Option<Box<dyn SmsProvider>>,
        audit_repo: Box<dyn AuditRepo>,
        config: NotificationConfig,
    ) -> Self {
        Self {
            email_service,
            sms_service,
            audit_repo,
            config,
        }
    }

    pub async fn send_user_notification(
        &self,
        email: &str,
        subject: &str,
        message: &str,
    ) -> Result<NotificationResult, NotificationError> {
        self.send_email_notification(email, subject, message, NotificationType::SystemNotification).await
    }

    pub async fn send_admin_notification(
        &self,
        subject: &str,
        message: &str,
    ) -> Result<Vec<NotificationResult>, NotificationError> {
        let mut results = Vec::new();
        
        for admin_email in &self.config.admin_emails {
            match self.send_email_notification(admin_email, subject, message, NotificationType::AdminAlert).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Failed to send admin notification to {}: {}", admin_email, e);
                    results.push(NotificationResult {
                        success: false,
                        message_id: None,
                        error: Some(e.to_string()),
                        delivered_at: None,
                    });
                }
            }
        }
        
        Ok(results)
    }

    pub async fn send_permission_expiration_notification(
        &self,
        email: &str,
        profile_name: &str,
        days_until_expiration: i64,
        expires_at: DateTime<Utc>,
    ) -> Result<NotificationResult, NotificationError> {
        let template = self.get_expiration_template(days_until_expiration);
        
        let mut context = HashMap::new();
        context.insert("profile_name".to_string(), profile_name.to_string());
        context.insert("days_until_expiration".to_string(), days_until_expiration.to_string());
        context.insert("expiration_date".to_string(), expires_at.format("%Y-%m-%d").to_string());
        context.insert("user_email".to_string(), email.to_string());

        let subject = self.render_template(&template.subject_template, &context)?;
        let body = self.render_template(&template.body_template, &context)?;

        self.send_email_notification(email, &subject, &body, template.template_type).await
    }

    pub async fn send_permission_assignment_notification(
        &self,
        email: &str,
        profile_name: &str,
        assigned_permissions: &[String],
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<NotificationResult, NotificationError> {
        let expiration_text = if let Some(expires) = expires_at {
            format!(" This assignment expires on {}.", expires.format("%Y-%m-%d"))
        } else {
            " This assignment does not expire.".to_string()
        };

        let subject = format!("New Permission Profile Assigned: {}", profile_name);
        let body = format!(
            "Hello,\n\n\
            You have been assigned the '{}' permission profile with the following permissions:\n\n\
            {}\n\n\
            {}\n\n\
            You can now access the features and functionality included with this profile.\n\n\
            If you have any questions, please contact support.\n\n\
            Best regards,\n\
            EPSX Team",
            profile_name,
            assigned_permissions.join("\n- "),
            expiration_text
        );

        self.send_email_notification(email, &subject, &body, NotificationType::PermissionAssigned).await
    }

    pub async fn send_permission_revocation_notification(
        &self,
        email: &str,
        profile_name: &str,
        reason: &str,
    ) -> Result<NotificationResult, NotificationError> {
        let subject = format!("Permission Profile Removed: {}", profile_name);
        let body = format!(
            "Hello,\n\n\
            Your '{}' permission profile has been removed.\n\n\
            Reason: {}\n\n\
            If you believe this was done in error or need to restore access, \
            please contact support immediately.\n\n\
            Best regards,\n\
            EPSX Team",
            profile_name,
            reason
        );

        self.send_email_notification(email, &subject, &body, NotificationType::PermissionRevoked).await
    }

    pub async fn send_bulk_notification(
        &self,
        recipients: &[String],
        subject: &str,
        message: &str,
        notification_type: NotificationType,
    ) -> Result<Vec<NotificationResult>, NotificationError> {
        let mut results = Vec::new();
        
        // Rate limiting check
        if recipients.len() > self.config.rate_limit_per_minute as usize {
            return Err(NotificationError::RateLimitExceeded(
                format!("Cannot send {} notifications, limit is {}", recipients.len(), self.config.rate_limit_per_minute)
            ));
        }
        
        for email in recipients {
            match self.send_email_notification(email, subject, message, notification_type.clone()).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Failed to send notification to {}: {}", email, e);
                    results.push(NotificationResult {
                        success: false,
                        message_id: None,
                        error: Some(e.to_string()),
                        delivered_at: None,
                    });
                }
            }
        }
        
        Ok(results)
    }

    async fn send_email_notification(
        &self,
        email: &str,
        subject: &str,
        message: &str,
        notification_type: NotificationType,
    ) -> Result<NotificationResult, NotificationError> {
        if !self.config.enable_email {
            return Err(NotificationError::ServiceDisabled("Email notifications are disabled".to_string()));
        }

        info!("Sending email notification to {}: {}", email, subject);

        let email_request = EmailRequest {
            to: email.to_string(),
            subject: subject.to_string(),
            body: message.to_string(),
            from_email: self.config.from_email.clone(),
            from_name: self.config.from_name.clone(),
            notification_type: notification_type.clone(),
        };

        match self.email_service.send_email(&email_request).await {
            Ok(result) => {
                info!("Email sent successfully to {}, message_id: {:?}", email, result.message_id);
                
                // Log successful notification
                if let Err(e) = self.audit_repo.log_notification_sent(
                    email, 
                    subject, 
                    &format!("{:?}", notification_type),
                    result.message_id.as_deref()
                ).await {
                    error!("Failed to log notification: {}", e);
                }
                
                Ok(result)
            },
            Err(e) => {
                error!("Failed to send email to {}: {}", email, e);
                
                // Log failed notification
                if let Err(audit_err) = self.audit_repo.log_notification_failed(
                    email, 
                    subject, 
                    &format!("{:?}", notification_type),
                    &e.to_string()
                ).await {
                    error!("Failed to log notification failure: {}", audit_err);
                }
                
                Err(NotificationError::EmailDeliveryFailed(e.to_string()))
            }
        }
    }

    fn get_expiration_template(&self, days_until_expiration: i64) -> NotificationTemplate {
        if days_until_expiration <= 0 {
            NotificationTemplate {
                subject_template: "Permission Profile Expired: {profile_name}".to_string(),
                body_template: "Hello,\n\nYour '{profile_name}' permission profile has expired and has been automatically removed.\n\nIf you need continued access, please contact support or upgrade your plan.\n\nBest regards,\nEPSX Team".to_string(),
                template_type: NotificationType::PermissionExpired,
            }
        } else if days_until_expiration <= 3 {
            NotificationTemplate {
                subject_template: "URGENT: Permission Profile Expiring in {days_until_expiration} day(s) - {profile_name}".to_string(),
                body_template: "Hello,\n\nURGENT: Your '{profile_name}' permission profile will expire in {days_until_expiration} day(s) on {expiration_date}.\n\nPlease renew your subscription immediately to avoid service interruption.\n\nContact support if you need assistance.\n\nBest regards,\nEPSX Team".to_string(),
                template_type: NotificationType::PermissionExpiring,
            }
        } else if days_until_expiration <= 7 {
            NotificationTemplate {
                subject_template: "Warning: Permission Profile Expiring Soon - {profile_name}".to_string(),
                body_template: "Hello,\n\nWarning: Your '{profile_name}' permission profile will expire in {days_until_expiration} day(s) on {expiration_date}.\n\nPlease renew your subscription to continue enjoying premium features.\n\nBest regards,\nEPSX Team".to_string(),
                template_type: NotificationType::PermissionExpiring,
            }
        } else {
            NotificationTemplate {
                subject_template: "Notice: Permission Profile Expiration Reminder - {profile_name}".to_string(),
                body_template: "Hello,\n\nNotice: Your '{profile_name}' permission profile will expire in {days_until_expiration} day(s) on {expiration_date}.\n\nConsider renewing your subscription to maintain access to premium features.\n\nBest regards,\nEPSX Team".to_string(),
                template_type: NotificationType::PermissionExpiring,
            }
        }
    }

    fn render_template(&self, template: &str, context: &HashMap<String, String>) -> Result<String, NotificationError> {
        let mut rendered = template.to_string();
        
        for (key, value) in context {
            let placeholder = format!("{{{}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }
        
        // Check for any remaining unreplaced placeholders
        if rendered.contains('{') && rendered.contains('}') {
            warn!("Template may have unreplaced placeholders: {}", rendered);
        }
        
        Ok(rendered)
    }

    pub async fn get_notification_preferences(&self, user_id: &Uuid) -> Result<NotificationPreferences, NotificationError> {
        // This would typically fetch from database, for now return defaults
        Ok(NotificationPreferences {
            email_enabled: true,
            sms_enabled: false,
            expiration_reminders: true,
            assignment_notifications: true,
            admin_alerts: false,
        })
    }

    pub async fn update_notification_preferences(
        &self, 
        user_id: &Uuid, 
        preferences: &NotificationPreferences
    ) -> Result<(), NotificationError> {
        // Log preference update
        if let Err(e) = self.audit_repo.log_system_event(
            "notification_preferences_updated", 
            &format!("User: {}, Preferences: {:?}", user_id, preferences)
        ).await {
            error!("Failed to log preference update: {}", e);
        }
        
        info!("Notification preferences updated for user {}", user_id);
        Ok(())
    }
}

#[derive(Debug)]
pub struct EmailRequest {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub from_email: String,
    pub from_name: String,
    pub notification_type: NotificationType,
}

#[derive(Debug)]
pub struct NotificationPreferences {
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub expiration_reminders: bool,
    pub assignment_notifications: bool,
    pub admin_alerts: bool,
}

#[async_trait::async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send_email(&self, request: &EmailRequest) -> Result<NotificationResult, EmailError>;
    async fn verify_email(&self, email: &str) -> Result<bool, EmailError>;
}

#[async_trait::async_trait]
pub trait SmsProvider: Send + Sync {
    async fn send_sms(&self, phone: &str, message: &str) -> Result<NotificationResult, SmsError>;
}

// Simple email provider implementation for development
pub struct SimpleEmailProvider {
    pub simulate_delivery: bool,
}

impl SimpleEmailProvider {
    pub fn new(simulate_delivery: bool) -> Self {
        Self { simulate_delivery }
    }
}

#[async_trait::async_trait]
impl EmailProvider for SimpleEmailProvider {
    async fn send_email(&self, request: &EmailRequest) -> Result<NotificationResult, EmailError> {
        if self.simulate_delivery {
            info!("Simulating email delivery to: {}", request.to);
            info!("Subject: {}", request.subject);
            info!("Body: {}", request.body);
            
            Ok(NotificationResult {
                success: true,
                message_id: Some(format!("sim_{}", Uuid::new_v4())),
                error: None,
                delivered_at: Some(Utc::now()),
            })
        } else {
            // In production, this would integrate with actual email service (SendGrid, SES, etc.)
            Err(EmailError::ConfigurationError("No email service configured".to_string()))
        }
    }

    async fn verify_email(&self, email: &str) -> Result<bool, EmailError> {
        // Basic email format validation
        Ok(email.contains('@') && email.contains('.'))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Email delivery failed: {0}")]
    EmailDeliveryFailed(String),
    
    #[error("SMS delivery failed: {0}")]
    SmsDeliveryFailed(String),
    
    #[error("Service disabled: {0}")]
    ServiceDisabled(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Template rendering error: {0}")]
    TemplateError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Invalid email format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SmsError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid phone number: {0}")]
    InvalidPhoneNumber(String),
}