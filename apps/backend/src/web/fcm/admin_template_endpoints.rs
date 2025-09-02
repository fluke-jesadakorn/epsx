// ============================================================================
// ADMIN NOTIFICATION TEMPLATE ENDPOINTS
// ============================================================================
// Template management endpoints for system notifications

use axum::{
    extract::{Path, Query, State},
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

use super::admin_system_endpoints::{
    NotificationTemplate, SystemMessageCategory, TemplateResponse, TemplateListResponse
};
use super::AuthenticatedUser;
use crate::infra::container::AppContainer;
use crate::core::errors::AppResult;
use crate::infra::services::fcm_push_service::FcmPriority;

// ============================================================================
// TEMPLATE MANAGEMENT REQUEST TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub title_template: String,
    pub body_template: String,
    pub data_template: Option<HashMap<String, String>>,
    pub category: SystemMessageCategory,
    pub priority: Option<FcmPriority>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub title_template: Option<String>,
    pub body_template: Option<String>,
    pub data_template: Option<HashMap<String, String>>,
    pub category: Option<SystemMessageCategory>,
    pub priority: Option<FcmPriority>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RenderTemplateRequest {
    pub template_id: String,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct RenderedTemplateResponse {
    pub success: bool,
    pub title: String,
    pub body: String,
    pub data: Option<HashMap<String, String>>,
    pub preview_html: String,
}

#[derive(Debug, Serialize)]
pub struct TemplateStatsResponse {
    pub total_templates: u32,
    pub active_templates: u32,
    pub most_used: Option<String>,
    pub templates_by_category: HashMap<String, u32>,
    pub usage_stats: Vec<TemplateUsageStat>,
}

#[derive(Debug, Serialize)]
pub struct TemplateUsageStat {
    pub template_id: String,
    pub template_name: String,
    pub usage_count: u32,
    pub last_used: Option<String>,
    pub success_rate: f32,
}

// ============================================================================
// TEMPLATE MANAGEMENT ENDPOINTS
// ============================================================================

/// Get all notification templates
pub async fn get_templates(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<HashMap<String, String>>,
) -> AppResult<Json<TemplateListResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} getting notification templates", user.user_id);

    let page = query.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let limit = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(20);
    let category = query.get("category");

    // Mock templates - in production, fetch from database
    let mut mock_templates = vec![
        NotificationTemplate {
            id: "tpl_001".to_string(),
            name: "Welcome Message".to_string(),
            title_template: "Welcome to EPSX, {{user_name}}!".to_string(),
            body_template: "Your account is now active. Start exploring our premium analytics features.".to_string(),
            data_template: Some({
                let mut data = HashMap::new();
                data.insert("action".to_string(), "open_dashboard".to_string());
                data.insert("user_id".to_string(), "{{user_id}}".to_string());
                data
            }),
            category: SystemMessageCategory::UserManagement,
            priority: FcmPriority::Normal,
            enabled: true,
            usage_count: 45,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
        NotificationTemplate {
            id: "tpl_002".to_string(),
            name: "Security Alert".to_string(),
            title_template: "🚨 Security Alert".to_string(),
            body_template: "Suspicious {{alert_type}} detected from {{location}}. If this wasn't you, secure your account immediately.".to_string(),
            data_template: Some({
                let mut data = HashMap::new();
                data.insert("alert_severity".to_string(), "high".to_string());
                data.insert("action".to_string(), "security_center".to_string());
                data
            }),
            category: SystemMessageCategory::Security,
            priority: FcmPriority::High,
            enabled: true,
            usage_count: 12,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
        NotificationTemplate {
            id: "tpl_003".to_string(),
            name: "System Maintenance".to_string(),
            title_template: "⚙️ Scheduled Maintenance".to_string(),
            body_template: "EPSX will undergo maintenance on {{maintenance_date}} from {{start_time}} to {{end_time}}. {{impact_description}}".to_string(),
            data_template: None,
            category: SystemMessageCategory::Maintenance,
            priority: FcmPriority::Normal,
            enabled: true,
            usage_count: 3,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    // Filter by category if specified
    if let Some(cat) = category {
        mock_templates.retain(|t| format!("{:?}", t.category).to_lowercase() == cat.to_lowercase());
    }

    Ok(Json(TemplateListResponse {
        templates: mock_templates,
        total: 3,
        page,
        limit,
    }))
}

/// Get a specific template by ID
pub async fn get_template(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(template_id): Path<String>,
) -> AppResult<Json<TemplateResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} getting template: {}", user.user_id, template_id);

    // Mock template retrieval - in production, fetch from database
    let template = NotificationTemplate {
        id: template_id.clone(),
        name: "Mock Template".to_string(),
        title_template: "{{title}}".to_string(),
        body_template: "{{message}}".to_string(),
        data_template: None,
        category: SystemMessageCategory::System,
        priority: FcmPriority::Normal,
        enabled: true,
        usage_count: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(TemplateResponse {
        success: true,
        message: "Template retrieved successfully".to_string(),
        template: Some(template),
    }))
}

/// Create a new notification template
pub async fn create_template(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateTemplateRequest>,
) -> AppResult<Json<TemplateResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} creating template: {}", user.user_id, request.name);

    let template = NotificationTemplate {
        id: format!("tpl_{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
        name: request.name,
        title_template: request.title_template,
        body_template: request.body_template,
        data_template: request.data_template,
        category: request.category,
        priority: request.priority.unwrap_or(FcmPriority::Normal),
        enabled: true,
        usage_count: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    // In production, save to database

    Ok(Json(TemplateResponse {
        success: true,
        message: "Template created successfully".to_string(),
        template: Some(template),
    }))
}

/// Update notification template
pub async fn update_template(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(template_id): Path<String>,
    Json(_request): Json<UpdateTemplateRequest>,
) -> AppResult<Json<TemplateResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} updating template: {}", user.user_id, template_id);

    // In production, update in database
    Ok(Json(TemplateResponse {
        success: true,
        message: "Template updated successfully".to_string(),
        template: None,
    }))
}

/// Delete notification template
pub async fn delete_template(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(template_id): Path<String>,
) -> AppResult<Json<TemplateResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} deleting template: {}", user.user_id, template_id);

    // In production, delete from database
    Ok(Json(TemplateResponse {
        success: true,
        message: "Template deleted successfully".to_string(),
        template: None,
    }))
}

/// Render template with variables for preview
pub async fn render_template(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<RenderTemplateRequest>,
) -> AppResult<Json<RenderedTemplateResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} rendering template: {}", user.user_id, request.template_id);

    // Mock template rendering - in production, load template and render
    let mut title = "Welcome to EPSX, {{user_name}}!".to_string();
    let mut body = "Your account is now active. Start exploring premium features.".to_string();

    // Simple variable substitution
    for (key, value) in &request.variables {
        let placeholder = format!("{{{{{}}}}}", key);
        title = title.replace(&placeholder, value);
        body = body.replace(&placeholder, value);
    }

    let preview_html = format!(
        r#"
        <div style="font-family: Arial, sans-serif; max-width: 400px; margin: 0 auto; border: 1px solid #ddd; border-radius: 8px; overflow: hidden;">
            <div style="background: #2563eb; color: white; padding: 16px;">
                <div style="font-size: 18px; font-weight: bold; margin-bottom: 8px;">{}</div>
            </div>
            <div style="padding: 16px;">
                <div style="color: #374151; line-height: 1.5;">{}</div>
                <div style="margin-top: 16px; font-size: 12px; color: #6b7280;">
                    EPSX Analytics Platform
                </div>
            </div>
        </div>
        "#,
        title, body
    );

    Ok(Json(RenderedTemplateResponse {
        success: true,
        title,
        body,
        data: None,
        preview_html,
    }))
}

/// Get template usage statistics
pub async fn get_template_stats(
    State(_container): State<AppContainer>,
    Extension(user): Extension<AuthenticatedUser>,
) -> AppResult<Json<TemplateStatsResponse>> {
    if !user.has_admin_access() {
        return Err(crate::core::errors::AppError::unauthorized("Admin access required"));
    }

    info!("Admin {} getting template stats", user.user_id);

    // Mock statistics - in production, calculate from database
    let mut templates_by_category = HashMap::new();
    templates_by_category.insert("UserManagement".to_string(), 5);
    templates_by_category.insert("Security".to_string(), 3);
    templates_by_category.insert("System".to_string(), 4);
    templates_by_category.insert("Maintenance".to_string(), 2);

    let usage_stats = vec![
        TemplateUsageStat {
            template_id: "tpl_001".to_string(),
            template_name: "Welcome Message".to_string(),
            usage_count: 45,
            last_used: Some(chrono::Utc::now().to_rfc3339()),
            success_rate: 98.5,
        },
        TemplateUsageStat {
            template_id: "tpl_002".to_string(),
            template_name: "Security Alert".to_string(),
            usage_count: 12,
            last_used: Some(chrono::Utc::now().to_rfc3339()),
            success_rate: 100.0,
        },
    ];

    Ok(Json(TemplateStatsResponse {
        total_templates: 8,
        active_templates: 6,
        most_used: Some("Welcome Message".to_string()),
        templates_by_category,
        usage_stats,
    }))
}