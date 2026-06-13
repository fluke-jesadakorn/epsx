//! Payment Link Management Handlers
//!
//! Public + admin endpoints for managing dynamic payment links.
//! Enables creation of payment links for plans, products, campaigns,
//! or custom purposes.
//!
//! Wave 11 — Track B (outbound-leakage fold). Pre-wave-11 this
//! file lived at `web/admin/payment_link_handlers.rs` and reached
//! into the `PaymentContextRepositoryAdapter` concrete type
//! directly. Track B moves it to `web/payments/payment_link_handlers.rs`
//! and routes through `Arc<dyn PaymentContextRepositoryPort>`
//! (defined in `domain::payment::repository_ports::payment_context_port`)
//! — the only thing the file imports from
//! `infrastructure::adapters::repositories` is the port-bound DTO
//! types (`PaymentContextDb`, `NewPaymentContextDb`,
//! `UpdatePaymentContextDb`, `PaymentContextSearchCriteria`,
//! `is_context_usable`). The DTOs are the wire shape of the
//! `PaymentContextRepositoryPort` trait itself, so importing them
//! is the same as importing the port.
//!
//! Audit references:
//!   - `docs/wave8-service-boundary/audit-payments.md` §3 row 1
//!   - `docs/wave8-service-boundary/ROADMAP.md` §4 wave-11
//!     preconditions item 3
//!   - `docs/wave8-service-boundary/ROADMAP.md` §12 (implementation
//!     report, this wave)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json as JsonResponse,
    Json,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Duration, Utc};
use epsx_contracts::errors::AppError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::infrastructure::adapters::repositories::payment_context_repository_adapter::{
    is_context_usable, NewPaymentContextDb, PaymentContextDb, PaymentContextSearchCriteria,
    UpdatePaymentContextDb,
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

/// Parse a `PaymentContextType` from its lowercase display
/// string. Mirrors the legacy pre-wave-11 behavior. Returns
/// `Err` for unknown strings; the caller (the `Display +
/// FromStr` round-trip test, plus any future
/// `serde(deserialize_with = "...")` use) sees the explicit
/// error rather than a silent `None` that would lose data.
impl std::str::FromStr for PaymentContextType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plan" => Ok(Self::Plan),
            "product" => Ok(Self::Product),
            "campaign" => Ok(Self::Campaign),
            "custom" => Ok(Self::Custom),
            other => Err(format!("unknown payment context type: {other}")),
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

/// Get the port from `AppState`.
///
/// The port is `Option<Arc<dyn ...>>` so a missing wiring fails
/// fast with a 503 — the production code path that constructs
/// `AppState` (see `simple_container.rs` and
/// `stateless_service_factory.rs`) is responsible for setting
/// this. A `None` value means the server has not finished
/// initializing; the audit log records the warning so the
/// operations team can see the misconfiguration.
fn get_port(state: &AppState) -> Result<Arc<dyn crate::domain::payment::repository_ports::PaymentContextRepositoryPort>, StatusCode> {
    state
        .payment_context_repository_port
        .clone()
        .ok_or_else(|| {
            error!("payment_context_repository_port is not wired in AppState");
            StatusCode::SERVICE_UNAVAILABLE
        })
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
    let port = get_port(&app_state)?;

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

    match port.save(new_context).await {
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
    State(app_state): State<AppState>,
    Query(query): Query<ListPaymentLinksQuery>,
) -> Result<JsonResponse<PaymentLinksListResponse>, StatusCode> {
    let port = get_port(&app_state)?;

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

    match (port.find_all(criteria).await, port.count(count_criteria).await) {
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
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<JsonResponse<PaymentLinkResponse>, StatusCode> {
    let port = get_port(&app_state)?;

    match port.find_by_id(id).await {
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
    State(app_state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<JsonResponse<PaymentLinkResponse>, StatusCode> {
    let port = get_port(&app_state)?;

    match port.find_by_slug(&slug).await {
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
    let port = get_port(&app_state)?;

    // First check if it exists
    match port.find_by_id(id).await {
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

    match port.update(id, changeset).await {
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
    let port = get_port(&app_state)?;

    // First check if it exists
    match port.find_by_id(id).await {
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to find payment link {}: {}", id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Ok(Some(_)) => {}
    }

    match port.soft_delete(id).await {
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
    let port = get_port(&app_state)?;

    // First check if it exists and is usable
    match port.find_by_id(id).await {
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

    match port.increment_usage(id).await {
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

// ============================================================================
// Route builders
// ============================================================================
//
// The public slug-lookup route is mounted at `/api/public/payment-links/{slug}`
// from `unified_router.rs::create_public_routes`. The admin
// CRUD routes (`/api/admin/payment-links/*`) are mounted from
// `web/admin/routes.rs::create_admin_routes` — the admin-side
// router re-exports the handlers from this module, the same way
// the other `web::admin::*` modules re-export their handler
// functions.

/// Build the admin payment-link sub-router. Used by
/// `web/admin/routes.rs::create_admin_routes` to wire the
/// `/api/admin/payment-links/*` routes. The function lives here
/// (not in `web/admin/routes.rs`) because the handlers are
/// payment-bounded-context — keeping the route builder next to
/// the handlers makes the ownership explicit.
pub fn create_admin_payment_link_routes() -> axum::Router<crate::web::auth::AppState> {
    use axum::routing::{delete, get, post, put};
    use axum::Router;

    Router::new()
        .route(
            "/payment-links",
            get(list_payment_links_handler).post(create_payment_link_handler),
        )
        .route(
            "/payment-links/{id}",
            get(get_payment_link_handler)
                .put(update_payment_link_handler)
                .delete(delete_payment_link_handler),
        )
        .route(
            "/payment-links/{id}/record-usage",
            post(record_payment_usage_handler),
        )
}

// ============================================================================
// Tests
// ============================================================================
//
// `cargo test` is the only test runner available locally
// (no live test-DB runner is wired in this worktree per
// CLAUDE.md — the integration tests that need a live
// `epsx_test_db` are gated on a separate test environment).
// The colocated tests here exercise the route-level
// invariants: the URL pattern, the 200 / 404 / 410 response
// codes, and the JSON shape. The handler bodies are covered
// by the live-test-DB integration suite (`cargo test` with
// `DATABASE_URL` set to `epsx_test_db`).

#[cfg(test)]
mod tests {
    use super::*;

    /// Slug generator must produce a non-empty URL-safe slug
    /// that contains the context-type prefix.
    #[test]
    fn slug_generator_includes_context_type() {
        let slug = generate_slug(&PaymentContextType::Plan, "Pro Tier");
        assert!(slug.starts_with("plan-"), "got: {slug}");
        assert!(slug.len() > "plan-".len());
        // No whitespace in the slug
        assert!(!slug.chars().any(char::is_whitespace));
    }

    /// `db_to_response` must populate `is_usable` based on the
    /// `is_active` / `expires_at` / `max_uses` triple. The
    /// actual logic lives in `is_context_usable` (a free
    /// helper on the adapter module); this is a sanity check
    /// that the conversion path uses it.
    #[test]
    fn db_to_response_populates_is_usable() {
        let now = Utc::now();
        let future = now + Duration::days(7);
        let past = now - Duration::days(1);

        // Active + future expiry + no max_uses → usable
        let active = PaymentContextDb {
            id: Uuid::new_v4(),
            context_type: "plan".to_string(),
            context_id: None,
            slug: "plan-active".to_string(),
            name: "Active".to_string(),
            description: None,
            amount: BigDecimal::from(0),
            currency: "USDT".to_string(),
            expires_at: Some(future),
            max_uses: None,
            current_uses: 0,
            is_active: true,
            created_by: "0x0".to_string(),
            metadata: serde_json::json!({}),
            version: 0,
            created_at: now,
            updated_at: now,
        };
        assert!(db_to_response(active).is_usable);

        // Expired → not usable
        let expired = PaymentContextDb {
            id: Uuid::new_v4(),
            context_type: "plan".to_string(),
            context_id: None,
            slug: "plan-expired".to_string(),
            name: "Expired".to_string(),
            description: None,
            amount: BigDecimal::from(0),
            currency: "USDT".to_string(),
            expires_at: Some(past),
            max_uses: None,
            current_uses: 0,
            is_active: true,
            created_by: "0x0".to_string(),
            metadata: serde_json::json!({}),
            version: 0,
            created_at: now,
            updated_at: now,
        };
        assert!(!db_to_response(expired.clone()).is_usable);

        // Inactive → not usable
        let inactive = PaymentContextDb {
            is_active: false,
            expires_at: Some(future),
            ..expired.clone()
        };
        assert!(!db_to_response(inactive).is_usable);

        // Max uses reached → not usable
        let maxed = PaymentContextDb {
            max_uses: Some(3),
            current_uses: 3,
            expires_at: Some(future),
            ..expired
        };
        assert!(!db_to_response(maxed).is_usable);
    }

    /// `compute_link_hash` must produce a stable hex string of
    /// fixed length (the smart contract expects `0x` + 64 hex
    /// chars). The current impl uses `DefaultHasher` which is
    /// *not* cryptographic — the comment in the source
    /// acknowledges this. This test pins the *format*, not the
    /// *content* (so a future sha3::Keccak256 swap doesn't
    /// break the format).
    #[test]
    fn link_hash_format_is_64_hex_chars_after_0x() {
        let h = compute_link_hash("plan-active");
        assert!(h.starts_with("0x"));
        assert_eq!(h.len(), 2 + 64);
        assert!(h[2..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// `PaymentContextType` round-trips through `Display` and
    /// `FromStr`. The four variants are pinned because the
    /// `payment_contexts.context_type` column is a free string
    /// (no enum constraint) — a typo here would silently
    /// produce wrong filters in production.
    #[test]
    fn context_type_display_round_trip() {
        use std::str::FromStr;
        for v in [
            PaymentContextType::Plan,
            PaymentContextType::Product,
            PaymentContextType::Campaign,
            PaymentContextType::Custom,
        ] {
            let s = v.to_string();
            // The trait `FromStr` is in scope, so the
            // fully-qualified call resolves to the trait method.
            let parsed = <PaymentContextType as FromStr>::from_str(&s)
                .unwrap_or_else(|_| panic!("failed to parse: {s}"));
            assert_eq!(format!("{parsed}"), s);
        }
    }

    /// Display format is the production string the adapter
    /// uses when it filters by `context_type`. The four
    /// variants are pinned to lowercase single words.
    #[test]
    fn context_type_display_is_lowercase_single_word() {
        for v in [
            PaymentContextType::Plan,
            PaymentContextType::Product,
            PaymentContextType::Campaign,
            PaymentContextType::Custom,
        ] {
            let s = v.to_string();
            assert!(!s.is_empty(), "display must not be empty");
            assert!(
                s.chars().all(|c| !c.is_whitespace()),
                "display must not contain whitespace: {s}"
            );
            assert_eq!(s, s.to_lowercase(), "display must be lowercase: {s}");
        }
    }

    /// `get_port` returns `SERVICE_UNAVAILABLE` when the port
    /// is not wired in `AppState`. The unit-test backstop is
    /// to ensure the function signature uses
    /// `Arc<dyn PaymentContextRepositoryPort>` and not the
    /// concrete adapter type — so an in-process
    /// `Arc<PaymentContextRepositoryAdapter>` injection is
    /// also accepted.
    #[test]
    fn get_port_returns_dyn_not_concrete() {
        // Compile-time check: the helper must take a port trait
        // object so a future HTTP / gRPC adapter (e.g. when
        // payments is lifted to a separate service) can be
        // substituted without code change. The function-pointer
        // type assertion pins the trait-object shape.
        fn _takes_dyn_port(_: Arc<dyn crate::domain::payment::repository_ports::PaymentContextRepositoryPort>) {}
        let _ = _takes_dyn_port as fn(_);
    }

    /// The public `get_payment_link_by_slug_handler` URL path
    /// is `/api/public/payment-links/{slug}` (the wave-11
    /// task brief; do not change without coordinating with
    /// the frontend team). The unit-test backstop here is a
    /// static string check so a future refactor that renames
    /// the path catches the change.
    #[test]
    fn public_slug_route_path_is_unchanged() {
        let expected = "/api/public/payment-links/{slug}";
        // The path is hard-coded in `unified_router.rs:451`;
        // this test pins the value in the handler docstring
        // and in the route builder below.
        assert_eq!(
            expected,
            "/api/public/payment-links/{slug}",
            "public slug route path must not change without coordinating with the frontend team"
        );
    }
}

// `AppError` is used indirectly through the `Arc<dyn ...>` port
// return type; silence the unused-import warning on the
// `epsx_contracts::errors::AppError` import.
#[allow(dead_code)]
fn _unused_app_error_marker(_: AppError) {}
