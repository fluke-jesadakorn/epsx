// ============================================================================
// ADMIN SYSTEM MESSAGE SETTINGS ENDPOINTS
// ============================================================================
// Advanced admin endpoints for system message settings and notification management

use axum::{
    extract::{Path, Query, State},
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

use crate::dom::values::UserId;
use crate::infra::db::diesel::types::DevicePlatform;
use crate::infra::services::fcm_push_service::{FcmMessage, FcmPriority, FcmBatchResult};
use super::AuthenticatedUser;
use crate::infra::container::AppContainer;
use crate::core::errors::AppResult;

// ============================================================================
// SYSTEM MESSAGE SETTINGS TYPES
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMessageSettings {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: FcmPriority,
    pub template: String,
    pub description: String,
    pub category: SystemMessageCategory,
    pub triggers: Vec<SystemMessageTrigger>,
    pub delivery_channels: Vec<DeliveryChannel>,
    pub schedule: Option<MessageSchedule>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationTemplate {
    pub id: String,
    pub name: String,
    pub title_template: String,
    pub body_template: String,
    pub data_template: Option<HashMap<String, String>>,
    pub category: SystemMessageCategory,
    pub priority: FcmPriority,
    pub enabled: bool,
    pub usage_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SystemMessageCategory {
    Security,
    System,
    UserManagement,
    Performance,
    Maintenance,
    Marketing,
    Analytics,
    Billing,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SystemMessageTrigger {
    UserRegistration,
    SecurityAlert,
    SystemMaintenance,
    PerformanceIssue,
    BillingEvent,
    AnalyticsReport,
    Manual,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeliveryChannel {
    FCMPush,
    Email,
    InApp,
    Webhook,
    SMS,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageSchedule {
    pub schedule_type: ScheduleType,
    pub cron_expression: Option<String>,
    pub immediate: bool,
    pub delay_minutes: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ScheduleType {
    Immediate,
    Scheduled,
    Recurring,
    Conditional,
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSystemMessageRequest {
    pub name: String,
    pub template: String,
    pub description: String,
    pub category: SystemMessageCategory,
    pub priority: Option<FcmPriority>,
    pub triggers: Vec<SystemMessageTrigger>,
    pub delivery_channels: Vec<DeliveryChannel>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSystemMessageRequest {
    pub name: Option<String>,
    pub template: Option<String>,
    pub description: Option<String>,
    pub category: Option<SystemMessageCategory>,
    pub priority: Option<FcmPriority>,
    pub triggers: Option<Vec<SystemMessageTrigger>>,
    pub delivery_channels: Option<Vec<DeliveryChannel>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TestSystemMessageRequest {
    pub template_id: Option<String>,
    pub title: String,
    pub body: String,
    pub test_user_ids: Vec<String>,
    pub data: Option<HashMap<String, String>>,
    pub priority: Option<FcmPriority>,
}

#[derive(Debug, Deserialize)]
pub struct BulkSystemMessageRequest {
    pub template_id: String,
    pub variables: HashMap<String, String>,
    pub target: BulkMessageTarget,
    pub priority: Option<FcmPriority>,
    pub schedule: Option<MessageSchedule>,
}

#[derive(Debug, Deserialize)]
pub enum BulkMessageTarget {
    All,
    ActiveUsers,
    Platform(DevicePlatform),
    UserList(Vec<String>),
    Conditional(String), // JSON condition
}

#[derive(Debug, Serialize)]
pub struct SystemMessageResponse {
    pub success: bool,
    pub message: String,
    pub system_message: Option<SystemMessageSettings>,
}

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub success: bool,
    pub message: String,
    pub template: Option<NotificationTemplate>,
}

#[derive(Debug, Serialize)]
pub struct SystemMessageListResponse {
    pub messages: Vec<SystemMessageSettings>,
    pub total: u32,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct TemplateListResponse {
    pub templates: Vec<NotificationTemplate>,
    pub total: u32,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct TestMessageResponse {
    pub success: bool,
    pub message: String,
    pub sent_count: u32,
    pub failed_count: u32,
    pub results: Vec<TestMessageResult>,
}

#[derive(Debug, Serialize)]
pub struct TestMessageResult {
    pub user_id: String,
    pub success: bool,
    pub message: String,
    pub delivery_details: Option<FcmBatchResult>,
}

#[derive(Debug, Serialize)]
pub struct SystemNotificationStats {
    pub total_messages: u32,
    pub active_messages: u32,
    pub total_templates: u32,
    pub messages_sent_today: u32,
    pub messages_sent_this_week: u32,
    pub most_used_category: String,
    pub most_used_template: Option<String>,
}

// ============================================================================
// SYSTEM MESSAGE MANAGEMENT ENDPOINTS
// ============================================================================

/// Get all system message settings
pub async fn get_system_messages(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<HashMap<String, String>>,
) -> AppResult<Json<SystemMessageListResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} getting system messages", user.user_id);

    let page = query.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let limit = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(20);

    // For now, return mock data - in production this would query the database
    let mock_messages = vec![
        SystemMessageSettings {
            id: "sm_001".to_string(),
            name: "User Registration Welcome".to_string(),
            enabled: true,
            priority: FcmPriority::Normal,
            template: "Welcome to EPSX, {{user_name}}! Your account is now active.".to_string(),
            description: "Welcome message for new user registrations".to_string(),
            category: SystemMessageCategory::UserManagement,
            triggers: vec![SystemMessageTrigger::UserRegistration],
            delivery_channels: vec![DeliveryChannel::FCMPush, DeliveryChannel::Email],
            schedule: Some(MessageSchedule {
                schedule_type: ScheduleType::Immediate,
                cron_expression: None,
                immediate: true,
                delay_minutes: None,
            }),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
        SystemMessageSettings {
            id: "sm_002".to_string(),
            name: "Security Alert".to_string(),
            enabled: true,
            priority: FcmPriority::High,
            template: "🚨 Security Alert: {{alert_type}} detected for {{user_email}}".to_string(),
            description: "High priority security alerts for suspicious activities".to_string(),
            category: SystemMessageCategory::Security,
            triggers: vec![SystemMessageTrigger::SecurityAlert],
            delivery_channels: vec![DeliveryChannel::FCMPush, DeliveryChannel::Email, DeliveryChannel::InApp],
            schedule: Some(MessageSchedule {
                schedule_type: ScheduleType::Immediate,
                cron_expression: None,
                immediate: true,
                delay_minutes: None,
            }),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    Ok(Json(SystemMessageListResponse {
        messages: mock_messages,
        total: 2,
        page,
        limit,
    }))
}

/// Create a new system message setting
pub async fn create_system_message(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateSystemMessageRequest>,
) -> AppResult<Json<SystemMessageResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} creating system message: {}", user.user_id, request.name);

    let system_message = SystemMessageSettings {
        id: format!("sm_{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
        name: request.name,
        enabled: request.enabled,
        priority: request.priority.unwrap_or(FcmPriority::Normal),
        template: request.template,
        description: request.description,
        category: request.category,
        triggers: request.triggers,
        delivery_channels: request.delivery_channels,
        schedule: Some(MessageSchedule {
            schedule_type: ScheduleType::Immediate,
            cron_expression: None,
            immediate: true,
            delay_minutes: None,
        }),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    // In production, save to database here

    Ok(Json(SystemMessageResponse {
        success: true,
        message: "System message created successfully".to_string(),
        system_message: Some(system_message),
    }))
}

/// Update system message setting
pub async fn update_system_message(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(message_id): Path<String>,
    Json(_request): Json<UpdateSystemMessageRequest>,
) -> AppResult<Json<SystemMessageResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} updating system message: {}", user.user_id, message_id);

    // In production, update in database
    Ok(Json(SystemMessageResponse {
        success: true,
        message: "System message updated successfully".to_string(),
        system_message: None,
    }))
}

/// Delete system message setting
pub async fn delete_system_message(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(message_id): Path<String>,
) -> AppResult<Json<SystemMessageResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} deleting system message: {}", user.user_id, message_id);

    // In production, delete from database
    Ok(Json(SystemMessageResponse {
        success: true,
        message: "System message deleted successfully".to_string(),
        system_message: None,
    }))
}

/// Test system message with specific users
pub async fn test_system_message(
    State(container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<TestSystemMessageRequest>,
) -> AppResult<Json<TestMessageResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} testing system message to {} users", user.user_id, request.test_user_ids.len());

    let fcm_push_service = container.fcm_push_service();
    let priority = request.priority.unwrap_or(FcmPriority::Normal);
    
    let mut results = Vec::new();
    let mut sent_count = 0;
    let mut failed_count = 0;

    for user_id_str in request.test_user_ids {
        match Uuid::parse_str(&user_id_str) {
            Ok(user_uuid) => {
                let user_id = UserId(user_uuid);
                let mut message = FcmMessage::simple_notification(request.title.clone(), request.body.clone());
                message.priority = priority.clone();
                
                if let Some(data) = &request.data {
                    for (key, value) in data {
                        if message.data.is_none() {
                            message.data = Some(HashMap::new());
                        }
                        if let Some(ref mut data_map) = message.data {
                            data_map.insert(key.clone(), value.clone());
                        }
                    }
                }

                match fcm_push_service.send_to_user(&user_id, &message, None).await {
                    Ok(batch_result) => {
                        sent_count += batch_result.successful;
                        failed_count += batch_result.failed;
                        
                        results.push(TestMessageResult {
                            user_id: user_id_str,
                            success: batch_result.successful > 0,
                            message: format!("Sent to {}/{} devices", batch_result.successful, batch_result.total_sent),
                            delivery_details: Some(batch_result),
                        });
                    }
                    Err(e) => {
                        failed_count += 1;
                        results.push(TestMessageResult {
                            user_id: user_id_str,
                            success: false,
                            message: format!("Failed: {}", e),
                            delivery_details: None,
                        });
                    }
                }
            }
            Err(_) => {
                failed_count += 1;
                results.push(TestMessageResult {
                    user_id: user_id_str,
                    success: false,
                    message: "Invalid user ID format".to_string(),
                    delivery_details: None,
                });
            }
        }
    }

    Ok(Json(TestMessageResponse {
        success: sent_count > 0,
        message: format!("Test completed: {}/{} users reached", sent_count, sent_count + failed_count),
        sent_count,
        failed_count,
        results,
    }))
}

/// Send bulk system message
pub async fn send_bulk_system_message(
    State(container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<BulkSystemMessageRequest>,
) -> AppResult<Json<TestMessageResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} sending bulk system message with template: {}", user.user_id, request.template_id);

    let fcm_push_service = container.fcm_push_service();
    let priority = request.priority.unwrap_or(FcmPriority::Normal);

    // In production, load template from database and render with variables
    let title = "System Notification";
    let body = "This is a bulk system notification";

    let mut message = FcmMessage::simple_notification(title.to_string(), body.to_string());
    message.priority = priority;

    match request.target {
        BulkMessageTarget::All => {
            // Send to all users
            match fcm_push_service.send_broadcast(&message, Some(1000), None).await {
                Ok(batch_result) => {
                    Ok(Json(TestMessageResponse {
                        success: batch_result.successful > 0,
                        message: format!("Broadcast sent to {}/{} devices", batch_result.successful, batch_result.total_sent),
                        sent_count: batch_result.successful,
                        failed_count: batch_result.failed,
                        results: vec![],
                    }))
                }
                Err(e) => {
                    Ok(Json(TestMessageResponse {
                        success: false,
                        message: format!("Broadcast failed: {}", e),
                        sent_count: 0,
                        failed_count: 1,
                        results: vec![],
                    }))
                }
            }
        }
        BulkMessageTarget::UserList(user_ids) => {
            // Send to specific users
            let mut sent_count = 0;
            let mut failed_count = 0;

            for user_id_str in user_ids {
                if let Ok(user_uuid) = Uuid::parse_str(&user_id_str) {
                    let user_id = UserId(user_uuid);
                    match fcm_push_service.send_to_user(&user_id, &message, None).await {
                        Ok(batch_result) => sent_count += batch_result.successful,
                        Err(_) => failed_count += 1,
                    }
                }
            }

            Ok(Json(TestMessageResponse {
                success: sent_count > 0,
                message: format!("Bulk message sent to {} users", sent_count),
                sent_count,
                failed_count,
                results: vec![],
            }))
        }
        _ => {
            Ok(Json(TestMessageResponse {
                success: false,
                message: "Target type not yet implemented".to_string(),
                sent_count: 0,
                failed_count: 0,
                results: vec![],
            }))
        }
    }
}

/// Get system notification statistics
pub async fn get_system_notification_stats(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
) -> AppResult<Json<SystemNotificationStats>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} getting system notification stats", user.user_id);

    // In production, calculate from database
    Ok(Json(SystemNotificationStats {
        total_messages: 15,
        active_messages: 12,
        total_templates: 8,
        messages_sent_today: 127,
        messages_sent_this_week: 892,
        most_used_category: "Security".to_string(),
        most_used_template: Some("User Registration Welcome".to_string()),
    }))
}