use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use serde_json::json;
use std::sync::Arc;

use crate::app::ports::services::{EmailSvc, EmailServiceError};
use crate::config::Config;

/// Simple email service implementation using SendGrid
pub struct SendGridEmailService {
    client: Client,
    config: Arc<Config>,
}

impl SendGridEmailService {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn from_config(config: Arc<Config>) -> Self {
        Self::new(config)
    }

    async fn send_email(
        &self,
        to_email: &str,
        to_name: &str,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), EmailServiceError> {
        let payload = json!({
            "personalizations": [{
                "to": [{
                    "email": to_email,
                    "name": to_name
                }]
            }],
            "from": {
                "email": self.config.email.from_email,
                "name": self.config.email.from_name
            },
            "subject": subject,
            "content": [
                {
                    "type": "text/plain",
                    "value": text_content
                },
                {
                    "type": "text/html",
                    "value": html_content
                }
            ]
        });

        let response = self
            .client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header("Authorization", format!("Bearer {}", self.config.email.sendgrid_api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| EmailServiceError::ExternalError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EmailServiceError::DeliveryFailed(error_text));
        }

        Ok(())
    }
}

#[async_trait]
impl EmailSvc for SendGridEmailService {
    async fn send_welcome_email(&self, email: &str, name: &str) -> Result<(), EmailServiceError> {
        let subject = format!("Welcome to {}", self.config.branding.platform_name);
        let welcome_message = self.config.branding.welcome_message_template
            .replace("{}", &self.config.branding.platform_name)
            .replace("{}", name);
        let html_content = format!(
            r#"
            <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <h1 style="color: #1a1a1a;">{}</h1>
                <p>Thank you for joining our EPS growth-based trading platform.</p>
                <p>Get started by exploring your dashboard and discovering trading opportunities based on EPS growth strategies.</p>
                <div style="margin: 30px 0;">
                    <a href="{}" 
                       style="background-color: #007bff; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px;">
                        Go to Dashboard
                    </a>
                </div>
                <p style="color: #666; font-size: 14px;">
                    If you have any questions, feel free to contact our support team.
                </p>
            </div>
            "#,
            welcome_message, self.config.branding.dashboard_url
        );

        let text_content = format!(
            "{}\n\nThank you for joining our EPS growth-based trading platform. Get started by exploring your dashboard at {}\n\nIf you have any questions, feel free to contact our support team at {}.",
            welcome_message, self.config.branding.dashboard_url, self.config.branding.support_email
        );

        self.send_email(email, name, &subject, &html_content, &text_content)
            .await
    }

    async fn send_password_reset(&self, email: &str, reset_link: &str) -> Result<(), EmailServiceError> {
        let subject = "Reset Your EPSX Password";
        let html_content = format!(
            r#"
            <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <h1 style="color: #1a1a1a;">Password Reset Request</h1>
                <p>You requested a password reset for your EPSX account.</p>
                <p>Click the button below to reset your password:</p>
                <div style="margin: 30px 0;">
                    <a href="{}" 
                       style="background-color: #dc3545; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px;">
                        Reset Password
                    </a>
                </div>
                <p style="color: #666; font-size: 14px;">
                    If you didn't request this password reset, please ignore this email. 
                    This link will expire in 1 hour for security reasons.
                </p>
                <p style="color: #666; font-size: 12px;">
                    If the button doesn't work, copy and paste this link into your browser:<br>
                    {}
                </p>
            </div>
            "#,
            reset_link, reset_link
        );

        let text_content = format!(
            "Password Reset Request\n\nYou requested a password reset for your EPSX account.\n\nClick this link to reset your password: {}\n\nIf you didn't request this password reset, please ignore this email. This link will expire in 1 hour for security reasons.",
            reset_link
        );

        self.send_email(email, "EPSX User", subject, &html_content, &text_content)
            .await
    }

    async fn send_payment_confirmation(
        &self,
        email: &str,
        amount: Decimal,
        currency: &str,
    ) -> Result<(), EmailServiceError> {
        let subject = "Payment Confirmation - EPSX";
        let html_content = format!(
            r#"
            <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <h1 style="color: #28a745;">Payment Confirmed!</h1>
                <p>Your payment has been successfully processed.</p>
                <div style="background-color: #f8f9fa; padding: 20px; border-radius: 5px; margin: 20px 0;">
                    <h3 style="margin: 0;">Payment Details</h3>
                    <p><strong>Amount:</strong> {} {}</p>
                    <p><strong>Status:</strong> Confirmed</p>
                    <p><strong>Date:</strong> {}</p>
                </div>
                <p>Your account has been updated with the new subscription tier.</p>
                <div style="margin: 30px 0;">
                    <a href="https://epsx.com/dashboard" 
                       style="background-color: #007bff; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px;">
                        View Dashboard
                    </a>
                </div>
            </div>
            "#,
            amount,
            currency.to_uppercase(),
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        let text_content = format!(
            "Payment Confirmed!\n\nYour payment has been successfully processed.\n\nPayment Details:\nAmount: {} {}\nStatus: Confirmed\nDate: {}\n\nYour account has been updated with the new subscription tier. View your dashboard at https://epsx.com/dashboard",
            amount,
            currency.to_uppercase(),
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_email(email, "EPSX User", subject, &html_content, &text_content)
            .await
    }

    async fn send_role_upgrade_notification(
        &self,
        email: &str,
        new_role: &str,
    ) -> Result<(), EmailServiceError> {
        let subject = "Account Upgrade - EPSX";
        let role_display = match new_role {
            "premium" => "Premium",
            "pro" => "Pro",
            "admin" => "Administrator",
            _ => new_role,
        };

        let html_content = format!(
            r#"
            <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <h1 style="color: #28a745;">Account Upgraded!</h1>
                <p>Congratulations! Your EPSX account has been upgraded to <strong>{}</strong> tier.</p>
                <div style="background-color: #f8f9fa; padding: 20px; border-radius: 5px; margin: 20px 0;">
                    <h3 style="margin: 0;">New Features Available:</h3>
                    <ul>
                        <li>Advanced EPS analysis tools</li>
                        <li>Real-time market data</li>
                        <li>Priority customer support</li>
                        <li>Enhanced trading strategies</li>
                    </ul>
                </div>
                <div style="margin: 30px 0;">
                    <a href="https://epsx.com/dashboard" 
                       style="background-color: #007bff; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px;">
                        Explore New Features
                    </a>
                </div>
            </div>
            "#,
            role_display
        );

        let text_content = format!(
            "Account Upgraded!\n\nCongratulations! Your EPSX account has been upgraded to {} tier.\n\nNew Features Available:\n- Advanced EPS analysis tools\n- Real-time market data\n- Priority customer support\n- Enhanced trading strategies\n\nExplore your new features at https://epsx.com/dashboard",
            role_display
        );

        self.send_email(email, "EPSX User", subject, &html_content, &text_content)
            .await
    }
}

/// Mock email service for testing and development
pub struct MockEmailService {
    pub sent_emails: std::sync::Arc<std::sync::Mutex<Vec<SentEmail>>>,
}

#[derive(Debug, Clone)]
pub struct SentEmail {
    pub to: String,
    pub subject: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MockEmailService {
    pub fn new() -> Self {
        Self {
            sent_emails: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn get_sent_emails(&self) -> Vec<SentEmail> {
        self.sent_emails.lock().unwrap().clone()
    }
}

#[async_trait]
impl EmailSvc for MockEmailService {
    async fn send_welcome_email(&self, email: &str, name: &str) -> Result<(), EmailServiceError> {
        let sent_email = SentEmail {
            to: email.to_string(),
            subject: "Welcome to EPSX Trading Platform".to_string(),
            content: format!("Welcome, {}!", name),
            timestamp: chrono::Utc::now(),
        };

        self.sent_emails.lock().unwrap().push(sent_email);
        Ok(())
    }

    async fn send_password_reset(&self, email: &str, reset_link: &str) -> Result<(), EmailServiceError> {
        let sent_email = SentEmail {
            to: email.to_string(),
            subject: "Reset Your EPSX Password".to_string(),
            content: format!("Reset link: {}", reset_link),
            timestamp: chrono::Utc::now(),
        };

        self.sent_emails.lock().unwrap().push(sent_email);
        Ok(())
    }

    async fn send_payment_confirmation(
        &self,
        email: &str,
        amount: Decimal,
        currency: &str,
    ) -> Result<(), EmailServiceError> {
        let sent_email = SentEmail {
            to: email.to_string(),
            subject: "Payment Confirmation - EPSX".to_string(),
            content: format!("Payment of {} {} confirmed", amount, currency),
            timestamp: chrono::Utc::now(),
        };

        self.sent_emails.lock().unwrap().push(sent_email);
        Ok(())
    }

    async fn send_role_upgrade_notification(
        &self,
        email: &str,
        new_role: &str,
    ) -> Result<(), EmailServiceError> {
        let sent_email = SentEmail {
            to: email.to_string(),
            subject: "Account Upgrade - EPSX".to_string(),
            content: format!("Upgraded to {} role", new_role),
            timestamp: chrono::Utc::now(),
        };

        self.sent_emails.lock().unwrap().push(sent_email);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_email_service() {
        let service = MockEmailService::new();
        
        service.send_welcome_email("test@example.com", "Test User").await.unwrap();
        service.send_password_reset("test@example.com", "http://example.com/reset").await.unwrap();
        
        let sent_emails = service.get_sent_emails();
        assert_eq!(sent_emails.len(), 2);
        assert_eq!(sent_emails[0].to, "test@example.com");
        assert_eq!(sent_emails[1].subject, "Reset Your EPSX Password");
    }
}