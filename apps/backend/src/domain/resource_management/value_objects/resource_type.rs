// Resource type value object
// Defines different types of resources that can be tracked and billed

use serde::{Deserialize, Serialize};
use std::fmt;

/// Types of resources that can be consumed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    // Internal web app usage (non-billable)
    WebPageLoad { page: String },
    WebUserAction { action: String },
    WebAnalyticsView { dataset: String },
    WebDashboardInteraction { component: String },
    WebSessionActivity { duration_seconds: u32 },
    
    // External API usage (billable)
    ApiCall { 
        endpoint: String, 
        method: String,
        response_size_bytes: u64,
    },
    ApiDataTransfer { 
        bytes: u64,
        direction: TransferDirection,
    },
    ApiWebhookDelivery { 
        destination: String,
        payload_size_bytes: u64,
    },
    ApiBulkOperation {
        operation_type: String,
        records_processed: u32,
    },
    
    // Admin operations (audit-level tracking)
    AdminUserManagement { operation: String },
    AdminPlanManagement { operation: String },
    AdminSystemOperation { operation: String },
    AdminDataExport { 
        export_type: String,
        records_count: u32,
    },
    
    // Shared infrastructure resources
    DatabaseQuery { 
        query_type: QueryType,
        execution_time_ms: u32,
    },
    CacheOperation { 
        operation: CacheOperationType,
        data_size_bytes: u64,
    },
    FileStorage { 
        operation: StorageOperation,
        file_size_bytes: u64,
    },
    ComputeTime { 
        duration_ms: u32,
        cpu_cores: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransferDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Analytics,
    Migration,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheOperationType {
    Hit,
    Miss,
    Set,
    Delete,
    Invalidate,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageOperation {
    Read,
    Write,
    Delete,
    List,
}

impl ResourceType {
    /// Check if this resource type is billable
    pub fn is_billable(&self) -> bool {
        match self {
            // Internal web app usage is not billable
            ResourceType::WebPageLoad { .. } |
            ResourceType::WebUserAction { .. } |
            ResourceType::WebAnalyticsView { .. } |
            ResourceType::WebDashboardInteraction { .. } |
            ResourceType::WebSessionActivity { .. } => false,
            
            // External API usage is billable
            ResourceType::ApiCall { .. } |
            ResourceType::ApiDataTransfer { .. } |
            ResourceType::ApiWebhookDelivery { .. } |
            ResourceType::ApiBulkOperation { .. } => true,
            
            // Admin operations are not billable but require audit
            ResourceType::AdminUserManagement { .. } |
            ResourceType::AdminPlanManagement { .. } |
            ResourceType::AdminSystemOperation { .. } |
            ResourceType::AdminDataExport { .. } => false,
            
            // Shared infrastructure costs are distributed
            ResourceType::DatabaseQuery { .. } |
            ResourceType::CacheOperation { .. } |
            ResourceType::FileStorage { .. } |
            ResourceType::ComputeTime { .. } => false,
        }
    }
    
    /// Get the category of this resource type
    pub fn category(&self) -> ResourceCategory {
        match self {
            ResourceType::WebPageLoad { .. } |
            ResourceType::WebUserAction { .. } |
            ResourceType::WebAnalyticsView { .. } |
            ResourceType::WebDashboardInteraction { .. } |
            ResourceType::WebSessionActivity { .. } => ResourceCategory::WebApp,
            
            ResourceType::ApiCall { .. } |
            ResourceType::ApiDataTransfer { .. } |
            ResourceType::ApiWebhookDelivery { .. } |
            ResourceType::ApiBulkOperation { .. } => ResourceCategory::ExternalApi,
            
            ResourceType::AdminUserManagement { .. } |
            ResourceType::AdminPlanManagement { .. } |
            ResourceType::AdminSystemOperation { .. } |
            ResourceType::AdminDataExport { .. } => ResourceCategory::Admin,
            
            ResourceType::DatabaseQuery { .. } |
            ResourceType::CacheOperation { .. } |
            ResourceType::FileStorage { .. } |
            ResourceType::ComputeTime { .. } => ResourceCategory::Infrastructure,
        }
    }
    
    /// Get the base cost factor for this resource type
    pub fn base_cost_factor(&self) -> f64 {
        match self {
            // Web app usage - no direct cost but infrastructure allocation
            ResourceType::WebPageLoad { .. } => 0.0001,
            ResourceType::WebUserAction { .. } => 0.0001,
            ResourceType::WebAnalyticsView { .. } => 0.0005,
            ResourceType::WebDashboardInteraction { .. } => 0.0002,
            ResourceType::WebSessionActivity { .. } => 0.0001,
            
            // API usage - billable rates
            ResourceType::ApiCall { .. } => 0.01,
            ResourceType::ApiDataTransfer { bytes, .. } => (*bytes as f64) * 0.0001,
            ResourceType::ApiWebhookDelivery { .. } => 0.005,
            ResourceType::ApiBulkOperation { records_processed, .. } => (*records_processed as f64) * 0.001,
            
            // Admin operations - operational cost tracking
            ResourceType::AdminUserManagement { .. } => 0.0,
            ResourceType::AdminPlanManagement { .. } => 0.0,
            ResourceType::AdminSystemOperation { .. } => 0.0,
            ResourceType::AdminDataExport { .. } => 0.0,
            
            // Infrastructure costs - distributed
            ResourceType::DatabaseQuery { execution_time_ms, .. } => (*execution_time_ms as f64) * 0.00001,
            ResourceType::CacheOperation { data_size_bytes, .. } => (*data_size_bytes as f64) * 0.000001,
            ResourceType::FileStorage { file_size_bytes, .. } => (*file_size_bytes as f64) * 0.0000001,
            ResourceType::ComputeTime { duration_ms, cpu_cores, .. } => {
                (*duration_ms as f64) * (*cpu_cores as f64) * 0.0001
            },
        }
    }
    
    /// Get the priority level for resource tracking
    pub fn priority(&self) -> TrackingPriority {
        match self {
            // High priority - billable or security-sensitive
            ResourceType::ApiCall { .. } |
            ResourceType::ApiDataTransfer { .. } |
            ResourceType::ApiWebhookDelivery { .. } |
            ResourceType::ApiBulkOperation { .. } |
            ResourceType::AdminUserManagement { .. } |
            ResourceType::AdminPlanManagement { .. } |
            ResourceType::AdminSystemOperation { .. } |
            ResourceType::AdminDataExport { .. } => TrackingPriority::High,
            
            // Medium priority - user experience tracking
            ResourceType::WebAnalyticsView { .. } |
            ResourceType::WebDashboardInteraction { .. } => TrackingPriority::Medium,
            
            // Low priority - basic usage tracking
            ResourceType::WebPageLoad { .. } |
            ResourceType::WebUserAction { .. } |
            ResourceType::WebSessionActivity { .. } |
            ResourceType::DatabaseQuery { .. } |
            ResourceType::CacheOperation { .. } |
            ResourceType::FileStorage { .. } |
            ResourceType::ComputeTime { .. } => TrackingPriority::Low,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceCategory {
    WebApp,
    ExternalApi,
    Admin,
    Infrastructure,
    UserManagement,
    SystemQueries,
    BulkOperations,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackingPriority {
    High,    // Real-time tracking required
    Medium,  // Near real-time tracking
    Low,     // Batch tracking acceptable
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::WebPageLoad { page } => write!(f, "WebPageLoad({})", page),
            ResourceType::WebUserAction { action } => write!(f, "WebUserAction({})", action),
            ResourceType::WebAnalyticsView { dataset } => write!(f, "WebAnalyticsView({})", dataset),
            ResourceType::WebDashboardInteraction { component } => write!(f, "WebDashboardInteraction({})", component),
            ResourceType::WebSessionActivity { duration_seconds } => write!(f, "WebSessionActivity({}s)", duration_seconds),
            
            ResourceType::ApiCall { endpoint, method, .. } => write!(f, "ApiCall({} {})", method, endpoint),
            ResourceType::ApiDataTransfer { bytes, direction } => write!(f, "ApiDataTransfer({:?} {}B)", direction, bytes),
            ResourceType::ApiWebhookDelivery { destination, .. } => write!(f, "ApiWebhookDelivery({})", destination),
            ResourceType::ApiBulkOperation { operation_type, records_processed } => {
                write!(f, "ApiBulkOperation({} {})", operation_type, records_processed)
            },
            
            ResourceType::AdminUserManagement { operation } => write!(f, "AdminUserManagement({})", operation),
            ResourceType::AdminPlanManagement { operation } => write!(f, "AdminPlanManagement({})", operation),
            ResourceType::AdminSystemOperation { operation } => write!(f, "AdminSystemOperation({})", operation),
            ResourceType::AdminDataExport { export_type, .. } => write!(f, "AdminDataExport({})", export_type),
            
            ResourceType::DatabaseQuery { query_type, .. } => write!(f, "DatabaseQuery({:?})", query_type),
            ResourceType::CacheOperation { operation, .. } => write!(f, "CacheOperation({:?})", operation),
            ResourceType::FileStorage { operation, .. } => write!(f, "FileStorage({:?})", operation),
            ResourceType::ComputeTime { duration_ms, cpu_cores } => write!(f, "ComputeTime({}ms, {}cores)", duration_ms, cpu_cores),
        }
    }
}

impl fmt::Display for ResourceCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceCategory::WebApp => write!(f, "WebApp"),
            ResourceCategory::ExternalApi => write!(f, "ExternalAPI"),
            ResourceCategory::Admin => write!(f, "Admin"),
            ResourceCategory::Infrastructure => write!(f, "Infrastructure"),
            ResourceCategory::UserManagement => write!(f, "UserManagement"),
            ResourceCategory::SystemQueries => write!(f, "SystemQueries"),
            ResourceCategory::BulkOperations => write!(f, "BulkOperations"),
        }
    }
}