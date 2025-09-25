// Email service implementations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::application::ports::outbound::{EmailServicePort, EmailServiceError};

/// Email sent confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentEmail {
    pub message_id: String,
    pub to: String,
    pub subject: String,
    pub sent_at: chrono::DateTime<chrono::Utc>,
    pub status: EmailStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailStatus {
    Sent,
    Delivered,
    Failed,
    Bounced,
}

/// SendGrid email service implementation
pub struct SendGridEmailService {
    #[allow(dead_code)]
    api_key: String,
}

impl SendGridEmailService {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
    
    /// Send welcome email to user
    pub async fn send_welcome_email(&self, email: &str, name: &str) -> Result<(), EmailServiceError> {
        tracing::info!("Sending welcome email to {} ({})", email, name);
        Ok(())
    }
    
    /// Send password reset email
    pub async fn send_password_reset(&self, email: &str, reset_link: &str) -> Result<(), EmailServiceError> {
        tracing::info!("Sending password reset email to {} with link: {}", email, reset_link);
        Ok(())
    }
    
    /// Send payment confirmation email
    pub async fn send_payment_confirmation(&self, email: &str, amount: rust_decimal::Decimal, currency: &str) -> Result<(), EmailServiceError> {
        tracing::info!("Sending payment confirmation to {} for {} {}", email, amount, currency);
        Ok(())
    }
    
    /// Send role upgrade notification email
    pub async fn send_role_upgrade_notification(&self, email: &str, new_role: &str) -> Result<(), EmailServiceError> {
        tracing::info!("Sending role upgrade notification to {} for role: {}", email, new_role);
        Ok(())
    }
}

#[async_trait]
impl EmailServicePort for SendGridEmailService {
    type Error = EmailServiceError;

    async fn send_email(&self, to: &str, subject: &str, _body: &str) -> Result<(), Self::Error> {
        // Placeholder implementation
        tracing::info!("Sending email to {} with subject: {}", to, subject);
        Ok(())
    }

    async fn send_template_email(&self, to: &str, template: &str, _data: &serde_json::Value) -> Result<(), Self::Error> {
        // Placeholder implementation
        tracing::info!("Sending template email {} to {}", template, to);
        Ok(())
    }
}

/// Simple SMTP email service
pub struct SmtpEmailService {
    host: String,
    port: u16,
    #[allow(dead_code)]
    username: String,
    #[allow(dead_code)]
    password: String,
}

impl SmtpEmailService {
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
        }
    }
}

#[async_trait]
impl EmailServicePort for SmtpEmailService {
    type Error = EmailServiceError;

    async fn send_email(&self, to: &str, _subject: &str, _body: &str) -> Result<(), Self::Error> {
        // Placeholder implementation
        tracing::info!("Sending SMTP email to {} via {}:{}", to, self.host, self.port);
        Ok(())
    }

    async fn send_template_email(&self, to: &str, template: &str, _data: &serde_json::Value) -> Result<(), Self::Error> {
        // Placeholder implementation  
        tracing::info!("Sending SMTP template email {} to {}", template, to);
        Ok(())
    }
}