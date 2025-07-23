// Audit web module for audit log API endpoints

pub mod handlers;

use axum::{
    routing::get,
    Router,
};

use crate::web::auth::AppState;
use handlers::{search_audit_logs, get_audit_statistics, export_audit_logs, get_audit_log};

/// Create audit log routes
pub fn create_audit_router() -> Router<AppState> {
    Router::new()
        // Search and list audit logs with filters
        .route("/logs", get(search_audit_logs))
        
        // Get specific audit log entry
        .route("/logs/:log_id", get(get_audit_log))
        
        // Get audit statistics for compliance reporting
        .route("/statistics", get(get_audit_statistics))
        
        // Export audit logs for compliance (JSON/CSV/XML)
        .route("/export", get(export_audit_logs))
}

/// Audit API endpoints overview:
/// 
/// GET /audit/logs
/// - Search audit logs with filters (actor, action, resource, time range, etc.)
/// - Supports pagination with limit/offset
/// - Query parameters:
///   - actor_id: Filter by user who performed action
///   - action: Filter by action type (login, user_created, etc.)
///   - resource_type: Filter by resource type (user, role, policy, etc.)  
///   - resource_id: Filter by specific resource ID
///   - result: Filter by result (success, failure, denied, error)
///   - from_time: Start of time range (ISO 8601)
///   - to_time: End of time range (ISO 8601)
///   - client_ip: Filter by client IP address
///   - session_id: Filter by session ID
///   - limit: Maximum results per page (default 100)
///   - offset: Pagination offset (default 0)
/// 
/// GET /audit/logs/{log_id}
/// - Get specific audit log entry by ID
/// - Returns detailed entry with all metadata
/// 
/// GET /audit/statistics
/// - Get audit statistics for compliance reporting
/// - Query parameters:
///   - from_time: Start of time range (required)
///   - to_time: End of time range (required)
/// - Returns:
///   - Total entries, success/failure counts
///   - Unique actor count
///   - Top actions and top actors
/// 
/// GET /audit/export
/// - Export audit logs for compliance
/// - Query parameters:
///   - format: Export format (json, csv, xml)
///   - All search filters from /logs endpoint
/// - Returns file download with appropriate content-type headers
/// 
/// Authentication & Authorization:
/// - All endpoints require authentication (admin role recommended)
/// - Audit log access should be strictly controlled
/// - Consider implementing additional authorization checks for sensitive operations
/// 
/// Compliance Features:
/// - Complete audit trail for all IAM and security operations
/// - Tamper-evident logging (timestamps, actor tracking)
/// - Export capabilities for compliance reporting
/// - Retention policy support via cleanup operations
/// - Search and filtering for audit investigations

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn should_create_audit_router() {
        let router = create_audit_router();
        // Basic test to ensure router creation doesn't panic
        // More comprehensive tests would require test harness setup
    }
}