// Audit API handlers for audit log management and compliance

use axum::{
    extract::{State, Path, Query},
    response::Json,
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::web::auth::AppState;
use crate::dom::entities::audit::{AuditQuery, AuditAction, ResourceType, AuditResult};
use crate::dom::values::UserId;
use crate::app::ports::repositories::ExportFormat;

/// Request to search audit logs
#[derive(Debug, Deserialize)]
pub struct SearchAuditLogsReq {
    pub actor_id: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub result: Option<String>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub client_ip: Option<String>,
    pub session_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Response for audit log search
#[derive(Debug, Serialize)]
pub struct SearchAuditLogsRes {
    pub entries: Vec<AuditLogEntryDto>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
}

/// Audit log entry DTO for API responses
#[derive(Debug, Serialize)]
pub struct AuditLogEntryDto {
    pub id: String,
    pub actor_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub result: String,
    pub timestamp: DateTime<Utc>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub metadata: AuditMetadataDto,
}

/// Audit metadata DTO
#[derive(Debug, Serialize)]
pub struct AuditMetadataDto {
    pub previous_values: Option<HashMap<String, String>>,
    pub new_values: Option<HashMap<String, String>>,
    pub error_message: Option<String>,
    pub additional_data: HashMap<String, String>,
    pub affected_count: Option<u32>,
    pub duration_ms: Option<u64>,
}

/// Request to get audit statistics
#[derive(Debug, Deserialize)]
pub struct GetAuditStatsReq {
    pub from_time: DateTime<Utc>,
    pub to_time: DateTime<Utc>,
}

/// Response for audit statistics
#[derive(Debug, Serialize)]
pub struct GetAuditStatsRes {
    pub total_entries: u64,
    pub failed_operations: u64,
    pub successful_operations: u64,
    pub unique_actors: u32,
    pub top_actions: Vec<ActionCount>,
    pub top_actors: Vec<ActorCount>,
    pub from_time: DateTime<Utc>,
    pub to_time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ActionCount {
    pub action: String,
    pub count: u32,
}

#[derive(Debug, Serialize)]
pub struct ActorCount {
    pub actor_id: String,
    pub count: u32,
}

/// Request to create audit log entry
#[derive(Debug, Deserialize)]
pub struct CreateAuditLogReq {
    pub user_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub details: HashMap<String, serde_json::Value>,
    pub ip_address: String,
    pub user_agent: String,
    pub event_category: String,
    pub severity: String,
    pub success: bool,
}

/// Request to export audit logs
#[derive(Debug, Deserialize)]
pub struct ExportAuditLogsReq {
    pub format: String, // "json", "csv", "xml"
    pub actor_id: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub result: Option<String>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}

/// Error response for audit operations
#[derive(Debug, Serialize)]
pub struct AuditErrorRes {
    pub error: String,
    pub code: String,
}

/// Response for audit log creation
#[derive(Debug, Serialize)]
pub struct CreateAuditLogRes {
    pub id: String,
    pub message: String,
}

/// Create a new audit log entry
pub async fn create_audit_log(
    State(state): State<AppState>,
    Json(req): Json<CreateAuditLogReq>,
) -> Result<Json<CreateAuditLogRes>, (StatusCode, Json<AuditErrorRes>)> {
    use crate::dom::entities::audit::{AuditLogEntry, AuditMetadata, AuditAction, ResourceType, AuditResult};
    
    // Parse user ID (use system UUID for events without user_id like registration)
    let actor_id = req.user_id
        .map(UserId::new)
        .unwrap_or_else(|| UserId::new("00000000-0000-0000-0000-000000000000".to_string()));
    
    // Map action string to AuditAction enum (simplified mapping)
    let action = match req.action.as_str() {
        "LOGIN_SUCCESS" => AuditAction::Login,
        "LOGIN_FAILED" => AuditAction::LoginFailed,
        "REGISTER_SUCCESS" => AuditAction::UserCreated,
        "REGISTER_FAILED" => AuditAction::LoginFailed, // Using closest match
        "LOGOUT" => AuditAction::Logout,
        "PASSWORD_RESET_REQUEST" => AuditAction::PasswordReset,
        "PASSWORD_RESET_SUCCESS" => AuditAction::PasswordReset,
        _ => AuditAction::Login, // Default fallback
    };
    
    // Map resource type string to ResourceType enum
    let resource_type = match req.resource_type.as_str() {
        "auth" => ResourceType::Session,
        "user" => ResourceType::User,
        "role" => ResourceType::Role,
        "policy" => ResourceType::Policy,
        "group" => ResourceType::Group,
        _ => ResourceType::Session, // Default fallback
    };
    
    // Result based on success flag
    let result = if req.success { AuditResult::Success } else { AuditResult::Failure };
    
    // Convert details to string map for audit metadata
    let mut additional_data = HashMap::new();
    for (key, value) in req.details {
        additional_data.insert(key, value.to_string());
    }
    
    // Create audit metadata
    let metadata = AuditMetadata {
        previous_values: None,
        new_values: None,
        error_message: if !req.success { 
            additional_data.get("error").cloned()
        } else { 
            None 
        },
        additional_data,
        affected_count: None,
        duration_ms: None,
    };
    
    // Create audit log entry with correct signature
    let audit_entry = AuditLogEntry::new(
        actor_id,
        action,
        resource_type,
        "unknown".to_string(), // resource_id - could be extracted from details
        result,
    )
    .with_metadata(metadata)
    .with_client_info(Some(req.ip_address), Some(req.user_agent));
    
    // Store in repository using the correct method name
    match state.audit_repo.store(&audit_entry).await {
        Ok(_) => {
            Ok(Json(CreateAuditLogRes {
                id: audit_entry.id().value().to_string(),
                message: "Audit log entry created successfully".to_string(),
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuditErrorRes {
                error: format!("Failed to create audit log: {}", e),
                code: "CREATE_ERROR".to_string(),
            })
        ))
    }
}

/// Search audit logs with filters and pagination
pub async fn search_audit_logs(
    State(state): State<AppState>,
    Query(req): Query<SearchAuditLogsReq>,
) -> Result<Json<SearchAuditLogsRes>, (StatusCode, Json<AuditErrorRes>)> {
    // Build audit query
    let mut query = AuditQuery::new();
    
    if let Some(actor_id) = req.actor_id {
        query = query.by_actor(UserId::new(actor_id));
    }
    
    if let Some(action_str) = req.action {
        if let Some(action) = parse_audit_action(&action_str) {
            query = query.by_action(action);
        }
    }
    
    if let Some(resource_type_str) = req.resource_type {
        if let Some(resource_type) = parse_resource_type(&resource_type_str) {
            query = query.by_resource_type(resource_type);
        }
    }
    
    if let Some(resource_id) = req.resource_id {
        query = query.by_resource_id(resource_id);
    }
    
    if let Some(result_str) = req.result {
        if let Some(result) = parse_audit_result(&result_str) {
            query = query.by_result(result);
        }
    }
    
    if let (Some(from), Some(to)) = (req.from_time, req.to_time) {
        query = query.in_time_range(from, to);
    }
    
    let limit = req.limit.unwrap_or(100);
    let offset = req.offset.unwrap_or(0);
    query = query.with_pagination(limit, offset);
    
    // Execute search
    match state.audit_repo.search(&query).await {
        Ok(entries) => {
            let total_count = state.audit_repo.count(&query).await
                .map_err(|e| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AuditErrorRes {
                        error: format!("Failed to count audit entries: {}", e),
                        code: "COUNT_ERROR".to_string(),
                    })
                ))?;
            
            let entries_dto: Vec<AuditLogEntryDto> = entries.into_iter()
                .map(|entry| AuditLogEntryDto {
                    id: entry.id().value().to_string(),
                    actor_id: entry.actor_id().to_string(),
                    action: entry.action().to_string(),
                    resource_type: entry.resource_type().to_string(),
                    resource_id: entry.resource_id().to_string(),
                    result: entry.result().to_string(),
                    timestamp: *entry.timestamp(),
                    client_ip: entry.client_ip().map(|s| s.to_string()),
                    user_agent: entry.user_agent().map(|s| s.to_string()),
                    session_id: entry.session_id().map(|s| s.to_string()),
                    metadata: AuditMetadataDto {
                        previous_values: entry.metadata().previous_values.clone(),
                        new_values: entry.metadata().new_values.clone(),
                        error_message: entry.metadata().error_message.clone(),
                        additional_data: entry.metadata().additional_data.clone(),
                        affected_count: entry.metadata().affected_count,
                        duration_ms: entry.metadata().duration_ms,
                    },
                })
                .collect();
            
            Ok(Json(SearchAuditLogsRes {
                entries: entries_dto,
                total_count,
                page: offset / limit,
                limit,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuditErrorRes {
                error: format!("Failed to search audit logs: {}", e),
                code: "SEARCH_ERROR".to_string(),
            })
        ))
    }
}

/// Get audit statistics for a time range
pub async fn get_audit_statistics(
    State(state): State<AppState>,
    Query(req): Query<GetAuditStatsReq>,
) -> Result<Json<GetAuditStatsRes>, (StatusCode, Json<AuditErrorRes>)> {
    match state.audit_repo.statistics(req.from_time, req.to_time).await {
        Ok(stats) => {
            let top_actions: Vec<ActionCount> = stats.top_actions.into_iter()
                .map(|(action, count)| ActionCount {
                    action: action.to_string(),
                    count,
                })
                .collect();
            
            let top_actors: Vec<ActorCount> = stats.top_actors.into_iter()
                .map(|(actor, count)| ActorCount {
                    actor_id: actor.to_string(),
                    count,
                })
                .collect();
            
            Ok(Json(GetAuditStatsRes {
                total_entries: stats.total_entries,
                failed_operations: stats.failed_operations,
                successful_operations: stats.successful_operations,
                unique_actors: stats.unique_actors,
                top_actions,
                top_actors,
                from_time: stats.from_time,
                to_time: stats.to_time,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuditErrorRes {
                error: format!("Failed to get audit statistics: {}", e),
                code: "STATS_ERROR".to_string(),
            })
        ))
    }
}

/// Export audit logs in various formats
pub async fn export_audit_logs(
    State(state): State<AppState>,
    Query(req): Query<ExportAuditLogsReq>,
) -> Result<axum::response::Response, (StatusCode, Json<AuditErrorRes>)> {
    // Parse export format
    let format = match req.format.as_str() {
        "json" => ExportFormat::Json,
        "csv" => ExportFormat::Csv,
        "xml" => ExportFormat::Xml,
        _ => return Err((
            StatusCode::BAD_REQUEST,
            Json(AuditErrorRes {
                error: "Invalid export format. Supported: json, csv, xml".to_string(),
                code: "INVALID_FORMAT".to_string(),
            })
        ))
    };
    
    // Build query
    let mut query = AuditQuery::new();
    
    if let Some(actor_id) = req.actor_id {
        query = query.by_actor(UserId::new(actor_id));
    }
    
    if let Some(action_str) = req.action {
        if let Some(action) = parse_audit_action(&action_str) {
            query = query.by_action(action);
        }
    }
    
    if let Some(resource_type_str) = req.resource_type {
        if let Some(resource_type) = parse_resource_type(&resource_type_str) {
            query = query.by_resource_type(resource_type);
        }
    }
    
    if let Some(resource_id) = req.resource_id {
        query = query.by_resource_id(resource_id);
    }
    
    if let Some(result_str) = req.result {
        if let Some(result) = parse_audit_result(&result_str) {
            query = query.by_result(result);
        }
    }
    
    if let (Some(from), Some(to)) = (req.from_time, req.to_time) {
        query = query.in_time_range(from, to);
    }
    
    if let Some(limit) = req.limit {
        query = query.with_pagination(limit, 0);
    }
    
    // Export data
    match state.audit_repo.export(&query, format.clone()).await {
        Ok(data) => {
            let content_type = match format {
                ExportFormat::Json => "application/json",
                ExportFormat::Csv => "text/csv",
                ExportFormat::Xml => "application/xml",
            };
            
            let filename = format!("audit_logs_{}.{}", 
                chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                match format {
                    ExportFormat::Json => "json",
                    ExportFormat::Csv => "csv", 
                    ExportFormat::Xml => "xml",
                });
            
            Ok(axum::response::Response::builder()
                .status(StatusCode::OK)
                .header("content-type", content_type)
                .header("content-disposition", format!("attachment; filename=\"{}\"", filename))
                .body(axum::body::Body::from(data))
                .unwrap())
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuditErrorRes {
                error: format!("Failed to export audit logs: {}", e),
                code: "EXPORT_ERROR".to_string(),
            })
        ))
    }
}

/// Get a specific audit log entry by ID
pub async fn get_audit_log(
    State(state): State<AppState>,
    Path(log_id): Path<String>,
) -> Result<Json<AuditLogEntryDto>, (StatusCode, Json<AuditErrorRes>)> {
    let audit_log_id = crate::dom::entities::audit::AuditLogId::new(log_id);
    
    match state.audit_repo.get(&audit_log_id).await {
        Ok(Some(entry)) => {
            Ok(Json(AuditLogEntryDto {
                id: entry.id().value().to_string(),
                actor_id: entry.actor_id().to_string(),
                action: entry.action().to_string(),
                resource_type: entry.resource_type().to_string(),
                resource_id: entry.resource_id().to_string(),
                result: entry.result().to_string(),
                timestamp: *entry.timestamp(),
                client_ip: entry.client_ip().map(|s| s.to_string()),
                user_agent: entry.user_agent().map(|s| s.to_string()),
                session_id: entry.session_id().map(|s| s.to_string()),
                metadata: AuditMetadataDto {
                    previous_values: entry.metadata().previous_values.clone(),
                    new_values: entry.metadata().new_values.clone(),
                    error_message: entry.metadata().error_message.clone(),
                    additional_data: entry.metadata().additional_data.clone(),
                    affected_count: entry.metadata().affected_count,
                    duration_ms: entry.metadata().duration_ms,
                },
            }))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(AuditErrorRes {
                error: "Audit log entry not found".to_string(),
                code: "NOT_FOUND".to_string(),
            })
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuditErrorRes {
                error: format!("Failed to get audit log: {}", e),
                code: "GET_ERROR".to_string(),
            })
        ))
    }
}

// Helper functions to parse enum values from strings

fn parse_audit_action(action_str: &str) -> Option<AuditAction> {
    match action_str {
        "login" => Some(AuditAction::Login),
        "login_failed" => Some(AuditAction::LoginFailed),
        "logout" => Some(AuditAction::Logout),
        "password_reset" => Some(AuditAction::PasswordReset),
        "session_expired" => Some(AuditAction::SessionExpired),
        "user_created" => Some(AuditAction::UserCreated),
        "user_updated" => Some(AuditAction::UserUpdated),
        "user_deleted" => Some(AuditAction::UserDeleted),
        "user_role_changed" => Some(AuditAction::UserRoleChanged),
        "user_level_changed" => Some(AuditAction::UserLevelChanged),
        "bulk_user_update" => Some(AuditAction::BulkUserUpdate),
        "role_created" => Some(AuditAction::RoleCreated),
        "role_updated" => Some(AuditAction::RoleUpdated),
        "role_deleted" => Some(AuditAction::RoleDeleted),
        "role_assigned" => Some(AuditAction::RoleAssigned),
        "role_unassigned" => Some(AuditAction::RoleUnassigned),
        "policy_created" => Some(AuditAction::PolicyCreated),
        "policy_updated" => Some(AuditAction::PolicyUpdated),
        "policy_deleted" => Some(AuditAction::PolicyDeleted),
        "policy_attached" => Some(AuditAction::PolicyAttached),
        "policy_detached" => Some(AuditAction::PolicyDetached),
        "group_created" => Some(AuditAction::GroupCreated),
        "group_updated" => Some(AuditAction::GroupUpdated),
        "group_deleted" => Some(AuditAction::GroupDeleted),
        "group_member_added" => Some(AuditAction::GroupMemberAdded),
        "group_member_removed" => Some(AuditAction::GroupMemberRemoved),
        "permission_granted" => Some(AuditAction::PermissionGranted),
        "permission_denied" => Some(AuditAction::PermissionDenied),
        "permission_evaluated" => Some(AuditAction::PermissionEvaluated),
        "permission_override_set" => Some(AuditAction::PermissionOverrideSet),
        "permission_override_removed" => Some(AuditAction::PermissionOverrideRemoved),
        "configuration_changed" => Some(AuditAction::ConfigurationChanged),
        "security_policy_updated" => Some(AuditAction::SecurityPolicyUpdated),
        "audit_log_accessed" => Some(AuditAction::AuditLogAccessed),
        "data_exported" => Some(AuditAction::DataExported),
        "backup_created" => Some(AuditAction::BackupCreated),
        "backup_restored" => Some(AuditAction::BackupRestored),
        _ => None,
    }
}

fn parse_resource_type(resource_type_str: &str) -> Option<ResourceType> {
    match resource_type_str {
        "user" => Some(ResourceType::User),
        "role" => Some(ResourceType::Role),
        "policy" => Some(ResourceType::Policy),
        "group" => Some(ResourceType::Group),
        "session" => Some(ResourceType::Session),
        "permission" => Some(ResourceType::Permission),
        "configuration" => Some(ResourceType::Configuration),
        "audit_log" => Some(ResourceType::AuditLog),
        "backup" => Some(ResourceType::Backup),
        "export" => Some(ResourceType::Export),
        _ => None,
    }
}

fn parse_audit_result(result_str: &str) -> Option<AuditResult> {
    match result_str {
        "success" => Some(AuditResult::Success),
        "failure" => Some(AuditResult::Failure),
        "partial_success" => Some(AuditResult::PartialSuccess),
        "denied" => Some(AuditResult::Denied),
        "error" => Some(AuditResult::Error),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn should_parse_audit_action() {
        assert_eq!(parse_audit_action("login"), Some(AuditAction::Login));
        assert_eq!(parse_audit_action("user_created"), Some(AuditAction::UserCreated));
        assert_eq!(parse_audit_action("invalid"), None);
    }
    
    #[test]
    fn should_parse_resource_type() {
        assert_eq!(parse_resource_type("user"), Some(ResourceType::User));
        assert_eq!(parse_resource_type("role"), Some(ResourceType::Role));
        assert_eq!(parse_resource_type("invalid"), None);
    }
    
    #[test]
    fn should_parse_audit_result() {
        assert_eq!(parse_audit_result("success"), Some(AuditResult::Success));
        assert_eq!(parse_audit_result("failure"), Some(AuditResult::Failure));
        assert_eq!(parse_audit_result("invalid"), None);
    }
}