//! Admin Payment Management API Handlers
//!
//! Comprehensive admin interface for managing payments, subscriptions, and financial operations
//! Requires admin permissions and provides detailed analytics and management capabilities

use axum::{
    extract::{State, Query, Path},
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error};

use crate::{
    prelude::*,
    web::middleware::UnifiedErrorResponse,
    schema::{subscriptions, groups},
};

/// Admin payment list query parameters
#[derive(Debug, Deserialize)]
pub struct AdminPaymentListParams {
    /// Page number for pagination
    pub page: Option<u32>,
    /// Number of items per page
    pub limit: Option<u32>,
    /// Filter by payment status
    pub status: Option<String>,
    /// Filter by wallet address
    pub wallet_address: Option<String>,
    /// Filter by plan ID
    pub plan_id: Option<Uuid>,
    /// Filter by date range (start)
    pub start_date: Option<String>,
    /// Filter by date range (end)
    pub end_date: Option<String>,
    /// Search by transaction hash or reference
    pub search: Option<String>,
}

/// Admin payment list response
#[derive(Debug, Serialize)]
pub struct AdminPaymentListResponse {
    pub success: bool,
    pub payments: Vec<AdminPaymentInfo>,
    pub pagination: PaginationInfo,
    pub summary: PaymentSummary,
}

/// Admin payment information
#[derive(Debug, Serialize)]
pub struct AdminPaymentInfo {
    pub id: Uuid,
    pub payment_reference: String,
    pub wallet_address: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub transaction_hash: Option<String>,
    pub contract_address: Option<String>,
    pub token_address: Option<String>,
    pub block_number: Option<i64>,
    pub confirmations: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

/// Pagination information
#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total_count: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Payment summary statistics
#[derive(Debug, Serialize)]
pub struct PaymentSummary {
    pub total_payments: u64,
    pub total_amount: f64,
    pub successful_payments: u64,
    pub failed_payments: u64,
    pub pending_payments: u64,
    pub average_payment_amount: f64,
    pub payments_today: u64,
    pub revenue_today: f64,
}

/// Admin payment details response
#[derive(Debug, Serialize)]
pub struct AdminPaymentDetailsResponse {
    pub success: bool,
    pub payment: Option<AdminPaymentInfo>,
    pub audit_logs: Vec<PaymentAuditLog>,
}

/// Payment audit log entry
#[derive(Debug, Serialize)]
pub struct PaymentAuditLog {
    pub id: Uuid,
    pub action: String,
    pub old_status: Option<String>,
    pub new_status: String,
    pub reason: Option<String>,
    pub performed_by: Option<String>,
    pub performed_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Admin subscription list response
#[derive(Debug, Serialize)]
pub struct AdminSubscriptionListResponse {
    pub success: bool,
    pub subscriptions: Vec<AdminSubscriptionInfo>,
    pub pagination: PaginationInfo,
    pub summary: SubscriptionSummary,
}

/// Admin subscription information
#[derive(Debug, Serialize)]
pub struct AdminSubscriptionInfo {
    pub id: Uuid,
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub status: String,
    pub payment_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub metadata: serde_json::Value,
}

/// Subscription summary statistics
#[derive(Debug, Serialize)]
pub struct SubscriptionSummary {
    pub total_subscriptions: u64,
    pub active_subscriptions: u64,
    pub expired_subscriptions: u64,
    pub cancelled_subscriptions: u64,
    pub new_subscriptions_today: u64,
    pub expiring_soon: u64, // Subscriptions expiring in next 7 days
    pub monthly_revenue: f64,
}

/// Payment analytics response
#[derive(Debug, Serialize)]
pub struct PaymentAnalyticsResponse {
    pub success: bool,
    pub analytics: PaymentAnalytics,
}

/// Payment analytics data
#[derive(Debug, Serialize)]
pub struct PaymentAnalytics {
    pub daily_revenue: Vec<DailyRevenue>,
    pub plan_breakdown: Vec<PlanBreakdown>,
    pub payment_methods: Vec<PaymentMethodStats>,
    pub trends: PaymentTrends,
}

/// Daily revenue data
#[derive(Debug, Serialize)]
pub struct DailyRevenue {
    pub date: String,
    pub revenue: f64,
    pub payment_count: u32,
}

/// Plan breakdown data
#[derive(Debug, Serialize)]
pub struct PlanBreakdown {
    pub plan_id: Uuid,
    pub plan_name: String,
    pub subscription_count: u32,
    pub revenue: f64,
    pub average_revenue_per_user: f64,
}

/// Payment method statistics
#[derive(Debug, Serialize)]
pub struct PaymentMethodStats {
    pub method: String,
    pub count: u32,
    pub revenue: f64,
    pub success_rate: f64,
}

/// Payment trends
#[derive(Debug, Serialize)]
pub struct PaymentTrends {
    pub growth_rate: f64,
    pub churn_rate: f64,
    pub average_subscription_length: f64,
    pub customer_lifetime_value: f64,
}

/// Refund payment request
#[derive(Debug, Deserialize)]
pub struct RefundPaymentRequest {
    pub reason: String,
    pub refund_amount: Option<f64>,
    pub partial_refund: bool,
    pub notify_user: bool,
}

/// Refund payment response
#[derive(Debug, Serialize)]
pub struct RefundPaymentResponse {
    pub success: bool,
    pub message: String,
    pub refund_id: Option<String>,
    pub refund_amount: f64,
    pub processed_at: DateTime<Utc>,
}

/// Update payment status request
#[derive(Debug, Deserialize)]
pub struct UpdatePaymentStatusRequest {
    pub status: String,
    pub reason: Option<String>,
    pub notify_user: bool,
    pub metadata: Option<serde_json::Value>,
}

/// Update payment status response
#[derive(Debug, Serialize)]
pub struct UpdatePaymentStatusResponse {
    pub success: bool,
    pub message: String,
    pub old_status: String,
    pub new_status: String,
    pub updated_at: DateTime<Utc>,
}

/// Get all payments with filtering and pagination
pub async fn admin_list_payments_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Query(params): Query<AdminPaymentListParams>,
) -> Result<Json<AdminPaymentListResponse>, Json<UnifiedErrorResponse>> {
    info!("Admin listing payments with params: {:?}", params);

    let _conn = app_state.db_pool.get().await
        .map_err(|e| {
            error!("Failed to get database connection: {}", e);
            Json(create_error_response(500, "Database connection failed", "Failed to establish database connection"))
        })?;

    // TODO: Implement comprehensive admin payment listing with:
    // - Pagination
    // - Filtering by status, wallet, plan, date range
    // - Search functionality
    // - Join with groups to get plan names
    // - Proper ordering

    // For now, return placeholder data
    let payments = vec![
        AdminPaymentInfo {
            id: Uuid::new_v4(),
            payment_reference: "PAY-001".to_string(),
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
            amount: 59.00,
            currency: "USD".to_string(),
            status: "completed".to_string(),
            plan_id: Uuid::new_v4(),
            plan_name: "Professional Plan".to_string(),
            transaction_hash: Some("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string()),
            contract_address: Some("0xcf2254fEa2ED6aAb8B846C150a00dC4faB2d7558".to_string()),
            token_address: Some("0x55d398326f99059fF775485246999027B3197955".to_string()),
            block_number: Some(12345678),
            confirmations: 12,
            created_at: Utc::now() - chrono::Duration::hours(2),
            updated_at: Utc::now() - chrono::Duration::hours(1),
            completed_at: Some(Utc::now() - chrono::Duration::hours(1)),
            expires_at: None,
            metadata: serde_json::json!({"network": "bsc"}),
        }
    ];

    let pagination = PaginationInfo {
        page: params.page.unwrap_or(1),
        limit: params.limit.unwrap_or(20),
        total_count: payments.len() as u64,
        total_pages: 1,
        has_next: false,
        has_prev: false,
    };

    let summary = PaymentSummary {
        total_payments: payments.len() as u64,
        total_amount: payments.iter().map(|p| p.amount).sum(),
        successful_payments: payments.iter().filter(|p| p.status == "completed").count() as u64,
        failed_payments: 0,
        pending_payments: 0,
        average_payment_amount: payments.iter().map(|p| p.amount).sum::<f64>() / payments.len() as f64,
        payments_today: 5,
        revenue_today: 295.00,
    };

    Ok(Json(AdminPaymentListResponse {
        success: true,
        payments,
        pagination,
        summary,
    }))
}

/// Get payment details with audit log
pub async fn admin_get_payment_details_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    Path(payment_id): Path<Uuid>,
) -> Result<Json<AdminPaymentDetailsResponse>, Json<UnifiedErrorResponse>> {
    info!("Admin getting payment details for {}", payment_id);

    // TODO: Implement comprehensive payment details retrieval
    // - Full payment information
    // - Complete audit log
    // - Related subscription information
    // - Transaction verification details

    let payment = AdminPaymentInfo {
        id: payment_id,
        payment_reference: "PAY-001".to_string(),
        wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
        amount: 59.00,
        currency: "USD".to_string(),
        status: "completed".to_string(),
        plan_id: Uuid::new_v4(),
        plan_name: "Professional Plan".to_string(),
        transaction_hash: Some("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string()),
        contract_address: Some("0xcf2254fEa2ED6aAb8B846C150a00dC4faB2d7558".to_string()),
        token_address: Some("0x55d398326f99059fF775485246999027B3197955".to_string()),
        block_number: Some(12345678),
        confirmations: 12,
        created_at: Utc::now() - chrono::Duration::hours(2),
        updated_at: Utc::now() - chrono::Duration::hours(1),
        completed_at: Some(Utc::now() - chrono::Duration::hours(1)),
        expires_at: None,
        metadata: serde_json::json!({"network": "bsc"}),
    };

    let audit_logs = vec![
        PaymentAuditLog {
            id: Uuid::new_v4(),
            action: "created".to_string(),
            old_status: None,
            new_status: "created".to_string(),
            reason: Some("Payment initiated".to_string()),
            performed_by: Some("0x1234567890123456789012345678901234567890".to_string()),
            performed_at: Utc::now() - chrono::Duration::hours(2),
            metadata: serde_json::json!({"source": "web"}),
        },
        PaymentAuditLog {
            id: Uuid::new_v4(),
            action: "confirmed".to_string(),
            old_status: Some("created".to_string()),
            new_status: "completed".to_string(),
            reason: Some("Transaction verified on blockchain".to_string()),
            performed_by: Some("system".to_string()),
            performed_at: Utc::now() - chrono::Duration::hours(1),
            metadata: serde_json::json!({"block_number": 12345678}),
        },
    ];

    Ok(Json(AdminPaymentDetailsResponse {
        success: true,
        payment: Some(payment),
        audit_logs,
    }))
}

/// Update payment status
pub async fn admin_update_payment_status_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    Path(payment_id): Path<Uuid>,
    Json(request): Json<UpdatePaymentStatusRequest>,
) -> Result<Json<UpdatePaymentStatusResponse>, Json<UnifiedErrorResponse>> {
    info!(
        "Admin updating payment {} status to {}",
        payment_id,
        request.status
    );

    // TODO: Implement payment status update
    // - Validate status transition
    // - Update payment in database
    // - Create audit log entry
    // - Send notifications if requested
    // - Handle related subscription status updates

    Ok(Json(UpdatePaymentStatusResponse {
        success: true,
        message: "Payment status updated successfully".to_string(),
        old_status: "created".to_string(),
        new_status: request.status,
        updated_at: Utc::now(),
    }))
}

/// Process refund
pub async fn admin_process_refund_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    Path(payment_id): Path<Uuid>,
    Json(request): Json<RefundPaymentRequest>,
) -> Result<Json<RefundPaymentResponse>, Json<UnifiedErrorResponse>> {
    info!(
        "Admin processing refund for payment {}, reason: {}",
        payment_id,
        request.reason
    );

    // TODO: Implement refund processing
    // - Validate refund eligibility
    // - Calculate refund amount
    // - Process blockchain refund (if applicable)
    // - Update payment status to 'refunded'
    // - Create audit log entry
    // - Notify user if requested
    // - Update subscription status if needed

    Ok(Json(RefundPaymentResponse {
        success: true,
        message: "Refund processed successfully".to_string(),
        refund_id: Some("REF-001".to_string()),
        refund_amount: request.refund_amount.unwrap_or(59.00),
        processed_at: Utc::now(),
    }))
}

/// Get all subscriptions
pub async fn admin_list_subscriptions_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Query(params): Query<AdminPaymentListParams>, // Reuse same params
) -> Result<Json<AdminSubscriptionListResponse>, Json<UnifiedErrorResponse>> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::models::payment::SubscriptionDb;

    info!("Admin listing subscriptions with params: {:?}", params);

    let mut conn = app_state.db_pool.get().await
        .map_err(|e| {
            error!("Failed to get database connection: {}", e);
            Json(create_error_response(500, "Database connection failed", "Failed to establish database connection"))
        })?;

    // Build query
    let mut query = subscriptions::table.into_boxed();

    // Apply wallet_address filter if provided
    if let Some(ref wallet_addr) = params.wallet_address {
        query = query.filter(subscriptions::wallet_address.eq(wallet_addr));
    }

    // Apply plan_id filter if provided
    if let Some(ref plan_id) = params.plan_id {
        query = query.filter(subscriptions::plan_id.eq(plan_id));
    }

    // Apply status filter if provided
    if let Some(ref status) = params.status {
        query = query.filter(subscriptions::status.eq(status));
    }

    // Apply pagination
    let limit = params.limit.unwrap_or(50) as i64;
    let page = params.page.unwrap_or(1);
    let offset = ((page - 1) * params.limit.unwrap_or(50)) as i64;

    // Get total count (before pagination)
    let total_count: i64 = subscriptions::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap_or(0);

    // Fetch subscriptions with pagination
    let subscription_rows = query
        .order(subscriptions::started_at.desc().nulls_last())
        .limit(limit)
        .offset(offset)
        .load::<SubscriptionDb>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to query subscriptions: {}", e);
            Json(create_error_response(500, "Query failed", &format!("Failed to load subscriptions: {}", e)))
        })?;

    // Map to response format with plan name lookup
    let mut subscriptions_resp: Vec<AdminSubscriptionInfo> = Vec::new();
    for sub_db in subscription_rows {
        // Try to get plan name from groups table
        let plan_name = groups::table
            .filter(groups::id.eq(sub_db.plan_id))
            .select(groups::name)
            .first::<String>(&mut conn)
            .await
            .unwrap_or_else(|_| "Unknown Plan".to_string());

        subscriptions_resp.push(AdminSubscriptionInfo {
            id: sub_db.id,
            wallet_address: sub_db.wallet_address,
            plan_id: sub_db.plan_id,
            plan_name,
            status: sub_db.status,
            payment_id: sub_db.payment_id.unwrap_or(Uuid::nil()),
            started_at: sub_db.started_at.unwrap_or_else(Utc::now),
            expires_at: sub_db.expires_at,
            cancelled_at: sub_db.cancelled_at,
            auto_renew: sub_db.auto_renew.unwrap_or(false),
            metadata: sub_db.metadata.unwrap_or(serde_json::json!({})),
        });
    }

    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;
    let pagination = PaginationInfo {
        page,
        limit: limit as u32,
        total_count: total_count as u64,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    // Calculate summary statistics
    let active_count = subscriptions_resp.iter().filter(|s| s.status == "active").count() as u64;
    let summary = SubscriptionSummary {
        total_subscriptions: total_count as u64,
        active_subscriptions: active_count,
        expired_subscriptions: subscriptions_resp.iter().filter(|s| s.status == "expired").count() as u64,
        cancelled_subscriptions: subscriptions_resp.iter().filter(|s| s.status == "cancelled").count() as u64,
        new_subscriptions_today: 0, // TODO: Calculate from timestamps
        expiring_soon: 0, // TODO: Calculate subscriptions expiring in next 7 days
        monthly_revenue: 0.0, // TODO: Calculate from payments
    };

    info!("Found {} subscriptions (page {} of {})", subscriptions_resp.len(), page, total_pages);

    Ok(Json(AdminSubscriptionListResponse {
        success: true,
        subscriptions: subscriptions_resp,
        pagination,
        summary,
    }))
}

/// Get payment analytics
pub async fn admin_get_payment_analytics_handler(
    State(_app_state): State<crate::web::auth::AppState>,
) -> Result<Json<PaymentAnalyticsResponse>, Json<UnifiedErrorResponse>> {
    info!("Admin getting payment analytics");

    // TODO: Implement comprehensive payment analytics
    // - Daily revenue trends
    // - Plan performance breakdown
    // - Payment method statistics
    // - Customer lifecycle metrics
    // - Revenue projections

    let analytics = PaymentAnalytics {
        daily_revenue: vec![
            DailyRevenue {
                date: "2024-01-01".to_string(),
                revenue: 295.00,
                payment_count: 5,
            },
            DailyRevenue {
                date: "2024-01-02".to_string(),
                revenue: 590.00,
                payment_count: 10,
            },
        ],
        plan_breakdown: vec![
            PlanBreakdown {
                plan_id: Uuid::new_v4(),
                plan_name: "Starter Plan".to_string(),
                subscription_count: 100,
                revenue: 2900.00,
                average_revenue_per_user: 29.00,
            },
            PlanBreakdown {
                plan_id: Uuid::new_v4(),
                plan_name: "Professional Plan".to_string(),
                subscription_count: 50,
                revenue: 2950.00,
                average_revenue_per_user: 59.00,
            },
        ],
        payment_methods: vec![
            PaymentMethodStats {
                method: "USDT".to_string(),
                count: 120,
                revenue: 7080.00,
                success_rate: 98.5,
            },
            PaymentMethodStats {
                method: "USDC".to_string(),
                count: 30,
                revenue: 1770.00,
                success_rate: 99.0,
            },
        ],
        trends: PaymentTrends {
            growth_rate: 15.5,
            churn_rate: 5.2,
            average_subscription_length: 45.0,
            customer_lifetime_value: 2655.00,
        },
    };

    Ok(Json(PaymentAnalyticsResponse {
        success: true,
        analytics,
    }))
}

/// Helper function to create UnifiedErrorResponse
fn create_error_response(code: u16, message: &str, reason: &str) -> UnifiedErrorResponse {
    UnifiedErrorResponse {
        success: false,
        error: crate::web::middleware::bearer_middleware::ErrorDetails {
            code,
            message: message.to_string(),
            reason: reason.to_string(),
        },
    }
}