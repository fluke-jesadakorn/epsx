//! Payment Link Management Handlers
//!
//! Admin endpoints for managing dynamic payment links.
//! Enables creation of payment links for plans, plans, products, campaigns, or custom purposes.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json as JsonResponse,
    Json,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::infrastructure::adapters::repositories::{
    PaymentContextRepositoryAdapter, PaymentContextSearchCriteria,
};
use crate::infrastructure::adapters::repositories::payment_context_repository_adapter::{
    NewPaymentContextDb, PaymentContextDb, UpdatePaymentContextDb, is_context_usable,
};
use crate::infrastructure::services::audit_service::{AuditCtx, AuditEntry};
use crate::web::auth::AppState;

// ============================================================================
// REQUEST/RESPONSE DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PaymentContextType {
    Plan,
    Product,
    Campaign,
    Custom,
}

impl std::fmt::Display for PaymentContextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plan => write!(f, "plan"),
            Self::Product => write!(f, "product"),
            Self::Campaign => write!(f, "campaign"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Request to create a new payment link
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePaymentLinkRequest {
    /// Type of payment context
    pub context_type: PaymentContextType,
    /// UUID of linked entity (plan, plan, etc.) - required for plan/plan
    pub context_id: Option<Uuid>,
    /// Custom slug (auto-generated if not provided)
    pub slug: Option<String>,
    /// Display name for the payment link
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Payment amount (as string for precision)
    pub amount: String,
    /// Currency (default: USDT)
    #[serde(default = "default_currency")]
    pub currency: String,
    /// Expiration time (default: 24 hours from now)
    pub expires_at: Option<DateTime<Utc>>,
    /// Maximum uses (null = unlimited multi-use)
    pub max_uses: Option<i32>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

fn default_currency() -> String {
    "USDT".to_string()
}

/// Request to update a payment link
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePaymentLinkRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
    pub max_uses: Option<Option<i32>>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Query parameters for listing payment links
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListPaymentLinksQuery {
    pub context_type: Option<String>,
    pub is_active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Payment link response
#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentLinkResponse {
    pub id: Uuid,
    pub context_type: String,
    pub context_id: Option<Uuid>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub amount: String,
    pub currency: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<i32>,
    pub current_uses: i32,
    pub is_active: bool,
    pub is_usable: bool,
    pub url: String,
    pub link_hash: String,
    pub created_by: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// List response with pagination
#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentLinksListResponse {
    pub payment_links: Vec<PaymentLinkResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Generate a URL-safe slug from name
fn generate_slug(context_type: &PaymentContextType, name: &str) -> String {
    let base = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>();

    let short_id = &Uuid::new_v4().to_string()[..8];
    format!(
        "{}-{}-{}",
        context_type,
        base.chars().take(20).collect::<String>(),
        short_id
    )
    .trim_matches('-')
    .to_string()
}

/// Compute keccak256 hash of slug for smart contract verification
fn compute_link_hash(slug: &str) -> String {
    // Simple hash for now - in production use sha3::Keccak256
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    slug.hash(&mut hasher);
    format!("0x{:064x}", hasher.finish())
}

/// Generate payment URL
fn generate_payment_url(slug: &str) -> String {
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "https://epsx.io".to_string());
    format!("{}/payment?link={}", frontend_url, slug)
}

/// Convert database model to response
fn db_to_response(db: PaymentContextDb) -> PaymentLinkResponse {
    let is_usable = is_context_usable(&db);
    PaymentLinkResponse {
        id: db.id,
        context_type: db.context_type.clone(),
        context_id: db.context_id,
        slug: db.slug.clone(),
        name: db.name,
        description: db.description,
        amount: db.amount.to_string(),
        currency: db.currency,
        expires_at: db.expires_at,
        max_uses: db.max_uses,
        current_uses: db.current_uses,
        is_active: db.is_active,
        is_usable,
        url: generate_payment_url(&db.slug),
        link_hash: compute_link_hash(&db.slug),
        created_by: db.created_by,
        metadata: db.metadata,
        created_at: db.created_at,
        updated_at: db.updated_at,
    }
}

/// Get repository from payments database pool
/// NOTE: payment_contexts table is in PAYMENTS database, not primary
async fn get_repository() -> Result<PaymentContextRepositoryAdapter, StatusCode> {
    use crate::infrastructure::database::get_payments_pool;
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(PaymentContextRepositoryAdapter::new(payments_pool))
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Create a new payment link
#[utoipa::path(
    post,
    path = "/api/admin/payment-links",
    request_body = CreatePaymentLinkRequest,
    responses(
        (status = 201, description = "Payment link created", body = PaymentLinkResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin Payment Links"
)]
pub async fn create_payment_link_handler(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: axum::http::HeaderMap,
    Json(request): Json<CreatePaymentLinkRequest>,
) -> Result<(StatusCode, JsonResponse<PaymentLinkResponse>), StatusCode> {
    let repo = get_repository().await?;

    // Generate slug if not provided
    let slug = request
        .slug
        .unwrap_or_else(|| generate_slug(&request.context_type, &request.name));

    // Default expiration: 24 hours from now
    let expires_at = request
        .expires_at
        .or_else(|| Some(Utc::now() + Duration::hours(24)));

    // Get wallet address from authentication context (placeholder for now)
    let created_by = "0x0000000000000000000000000000000000000000".to_string();

    // Parse amount
    let amount = BigDecimal::from_str(&request.amount).map_err(|_| {
        error!("Invalid amount: {}", request.amount);
        StatusCode::BAD_REQUEST
    })?;

    let new_context = NewPaymentContextDb {
        id: Uuid::new_v4(),
        context_type: request.context_type.to_string(),
        context_id: request.context_id,
        slug,
        name: request.name,
        description: request.description,
        amount,
        currency: request.currency,
        expires_at,
        max_uses: request.max_uses,
        current_uses: 0,
        is_active: true,
        created_by,
        metadata: request.metadata.unwrap_or_else(|| serde_json::json!({})),
    };

    match repo.save(new_context).await {
        Ok(saved) => {
            info!("Created payment link: {} ({})", saved.name, saved.slug);

            // Audit log
            let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
            let entry = AuditEntry::new("payment_link", "create", "payment")
                .id(&saved.id.to_string())
                .meta(serde_json::json!({
                    "name": saved.name,
                    "slug": saved.slug,
                    "context_type": saved.context_type,
                    "amount": saved.amount.to_string(),
                    "currency": saved.currency,
                }));
            app_state.audit.log(ctx, entry);

            Ok((StatusCode::CREATED, JsonResponse(db_to_response(saved))))
        }
        Err(e) => {
            error!("Failed to create payment link: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List payment links with filtering and pagination
#[utoipa::path(
    get,
    path = "/api/admin/payment-links",
    params(
        ("context_type" = Option<String>, Query, description = "Filter by context type"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("limit" = Option<i64>, Query, description = "Number of results"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination")
    ),
    responses(
        (status = 200, description = "List of payment links", body = PaymentLinksListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin Payment Links"
)]
pub async fn list_payment_links_handler(
    State(_app_state): State<AppState>,
    Query(query): Query<ListPaymentLinksQuery>,
) -> Result<JsonResponse<PaymentLinksListResponse>, StatusCode> {
    let repo = get_repository().await?;

    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let criteria = PaymentContextSearchCriteria {
        context_type: query.context_type,
        is_active: query.is_active,
        limit: Some(limit),
        offset: Some(offset),
        ..Default::default()
    };

    let count_criteria = PaymentContextSearchCriteria {
        context_type: criteria.context_type.clone(),
        is_active: criteria.is_active,
        ..Default::default()
    };

    match (repo.find_all(criteria).await, repo.count(count_criteria).await) {
        (Ok(contexts), Ok(total)) => {
            let payment_links: Vec<PaymentLinkResponse> =
                contexts.into_iter().map(db_to_response).collect();

            Ok(JsonResponse(PaymentLinksListResponse {
                payment_links,
                total,
                limit,
                offset,
            }))
        }
        (Err(e), _) | (_, Err(e)) => {
            error!("Failed to list payment links: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a payment link by ID
#[utoipa::path(
    get,
    path = "/api/admin/payment-links/{id}",
    params(
        ("id" = Uuid, Path, description = "Payment link ID")
    ),
    responses(
        (status = 200, description = "Payment link details", body = PaymentLinkResponse),
        (status = 404, description = "Payment link not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin Payment Links"
)]
pub async fn get_payment_link_handler(
    State(_app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<JsonResponse<PaymentLinkResponse>, StatusCode> {
    let repo = get_repository().await?;

    match repo.find_by_id(id).await {
        Ok(Some(context)) => Ok(JsonResponse(db_to_response(context))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get payment link {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a payment link by slug (public endpoint for payment page)
#[utoipa::path(
    get,
    path = "/api/public/payment-links/{slug}",
    params(
        ("slug" = String, Path, description = "Payment link slug")
    ),
    responses(
        (status = 200, description = "Payment link details", body = PaymentLinkResponse),
        (status = 404, description = "Payment link not found"),
        (status = 410, description = "Payment link expired or max uses reached"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Public Payment Links"
)]
pub async fn get_payment_link_by_slug_handler(
    State(_app_state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<JsonResponse<PaymentLinkResponse>, StatusCode> {
    let repo = get_repository().await?;

    match repo.find_by_slug(&slug).await {
        Ok(Some(context)) => {
            // Check if usable
            if !is_context_usable(&context) {
                return Err(StatusCode::GONE); // 410
            }
            Ok(JsonResponse(db_to_response(context)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get payment link by slug {}: {}", slug, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update a payment link
#[utoipa::path(
    put,
    path = "/api/admin/payment-links/{id}",
    params(
        ("id" = Uuid, Path, description = "Payment link ID")
    ),
    request_body = UpdatePaymentLinkRequest,
    responses(
        (status = 200, description = "Payment link updated", body = PaymentLinkResponse),
        (status = 404, description = "Payment link not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin Payment Links"
)]
pub async fn update_payment_link_handler(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdatePaymentLinkRequest>,
) -> Result<JsonResponse<PaymentLinkResponse>, StatusCode> {
    let repo = get_repository().await?;

    // First check if it exists
    match repo.find_by_id(id).await {
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to find payment link {}: {}", id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Ok(Some(_)) => {}
    }

    // Parse amount if provided
    let parsed_amount = match &request.amount {
        Some(amt) => Some(BigDecimal::from_str(amt).map_err(|_| {
            error!("Invalid amount: {}", amt);
            StatusCode::BAD_REQUEST
        })?),
        None => None,
    };

    let changeset = UpdatePaymentContextDb {
        name: request.name,
        description: request.description,
        amount: parsed_amount,
        currency: request.currency,
        expires_at: request.expires_at,
        max_uses: request.max_uses,
        is_active: request.is_active,
        metadata: request.metadata,
        updated_at: Utc::now(),
    };

    match repo.update(id, changeset).await {
        Ok(updated) => {
            info!("Updated payment link: {}", id);

            // Audit log
            let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
            let entry = AuditEntry::new("payment_link", "update", "payment")
                .id(&updated.id.to_string())
                .meta(serde_json::json!({
                    "name": updated.name,
                    "is_active": updated.is_active,
                }));
            app_state.audit.log(ctx, entry);

            Ok(JsonResponse(db_to_response(updated)))
        }
        Err(e) => {
            error!("Failed to update payment link {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete (deactivate) a payment link
#[utoipa::path(
    delete,
    path = "/api/admin/payment-links/{id}",
    params(
        ("id" = Uuid, Path, description = "Payment link ID")
    ),
    responses(
        (status = 204, description = "Payment link deactivated"),
        (status = 404, description = "Payment link not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin Payment Links"
)]
pub async fn delete_payment_link_handler(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = get_repository().await?;

    // First check if it exists
    match repo.find_by_id(id).await {
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to find payment link {}: {}", id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Ok(Some(_)) => {}
    }

    match repo.soft_delete(id).await {
        Ok(()) => {
            info!("Deactivated payment link: {}", id);

            // Audit log
            let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
            let entry = AuditEntry::new("payment_link", "delete", "payment")
                .id(&id.to_string());
            app_state.audit.log(ctx, entry);

            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to delete payment link {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Record a payment usage (called after successful blockchain payment)
#[utoipa::path(
    post,
    path = "/api/admin/payment-links/{id}/record-usage",
    params(
        ("id" = Uuid, Path, description = "Payment link ID")
    ),
    responses(
        (status = 200, description = "Usage recorded", body = PaymentLinkResponse),
        (status = 404, description = "Payment link not found"),
        (status = 410, description = "Payment link no longer usable"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin Payment Links"
)]
pub async fn record_payment_usage_handler(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<JsonResponse<PaymentLinkResponse>, StatusCode> {
    let repo = get_repository().await?;

    // First check if it exists and is usable
    match repo.find_by_id(id).await {
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Ok(Some(context)) => {
            if !is_context_usable(&context) {
                return Err(StatusCode::GONE); // 410
            }
        }
        Err(e) => {
            error!("Failed to find payment link {}: {}", id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    match repo.increment_usage(id).await {
        Ok(updated) => {
            info!(
                "Recorded usage for payment link: {} (now {} uses)",
                id, updated.current_uses
            );

            // Audit log
            let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
            let entry = AuditEntry::new("payment_link", "record_usage", "payment")
                .id(&updated.id.to_string())
                .meta(serde_json::json!({
                    "current_uses": updated.current_uses,
                    "name": updated.name,
                }));
            app_state.audit.log(ctx, entry);

            Ok(JsonResponse(db_to_response(updated)))
        }
        Err(e) => {
            error!("Failed to record usage for payment link {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
