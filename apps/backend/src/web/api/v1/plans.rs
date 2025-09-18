use axum::{
    extract::{Query, Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    application::marketing::{PlanService, PlanFilters, PromotionService, AffiliateService, PlanWithPromotions},
    infrastructure::{
        adapters::repositories::diesel_types::DbPool,
        cache::{Cache, CacheFactory},
    },
};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlansResponse {
    pub success: bool,
    pub data: Vec<PlanWithPromotions>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanResponse {
    pub success: bool,
    pub data: Option<PlanWithPromotions>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct PlansQueryParams {
    pub plan_type: Option<String>,
    pub is_active: Option<bool>,
    pub is_highlighted: Option<bool>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub include_affiliate_data: Option<bool>,
    pub affiliate_code: Option<String>,
}

pub struct PlansHandlers {
    plan_service: Arc<PlanService>,
    promotion_service: Arc<PromotionService>,
    affiliate_service: Arc<AffiliateService>,
}

impl PlansHandlers {
    pub fn new(
        db_pool: Arc<PgPool>,
        cache: Arc<dyn Cache>,
    ) -> Self {
        Self {
            plan_service: Arc::new(PlanService::new(db_pool.clone(), cache.clone())),
            promotion_service: Arc::new(PromotionService::new(db_pool.clone(), cache.clone())),
            affiliate_service: Arc::new(AffiliateService::new(db_pool, cache)),
        }
    }

    /// GET /api/v1/plans
    pub async fn get_plans(
        State(handlers): State<Arc<Self>>,
        Query(params): Query<PlansQueryParams>,
    ) -> Result<Json<PlansResponse>, (StatusCode, Json<ErrorResponse>)> {
        let filters = Some(PlanFilters {
            plan_type: params.plan_type,
            is_active: params.is_active,
            is_highlighted: params.is_highlighted,
            min_price: params.min_price,
            max_price: params.max_price,
        });

        match handlers.plan_service.list_plans(filters).await {
            Ok(plans) => {
                // Add affiliate tracking if affiliate code provided
                if let Some(affiliate_code) = params.affiliate_code {
                    if let Ok(Some(_affiliate)) = handlers.affiliate_service.get_affiliate_by_code(&affiliate_code).await {
                        // Track the referral click
                        let _ = handlers.affiliate_service.track_referral_click(
                            &affiliate_code,
                            "0.0.0.0", // TODO: Extract real IP from request
                            None,
                            Some("api".to_string()),
                            Some("direct".to_string()),
                            Some("plan_listing".to_string())
                        ).await;
                    }
                }

                Ok(Json(PlansResponse {
                    success: true,
                    data: plans,
                    message: None,
                }))
            }
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    message: format!("Failed to fetch plans: {}", e),
                }),
            )),
        }
    }

    /// GET /api/v1/plans/{plan_type}
    pub async fn get_plans_by_type(
        State(handlers): State<Arc<Self>>,
        Path(plan_type): Path<String>,
        Query(params): Query<PlansQueryParams>,
    ) -> Result<Json<PlansResponse>, (StatusCode, Json<ErrorResponse>)> {
        // Validate plan_type
        if !["personal", "api"].contains(&plan_type.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    message: "Invalid plan type. Must be 'personal' or 'api'".to_string(),
                }),
            ));
        }

        match handlers.plan_service.get_plans_by_type(&plan_type).await {
            Ok(plans) => {
                // Track affiliate referral if provided
                if let Some(affiliate_code) = params.affiliate_code {
                    if let Ok(Some(_affiliate)) = handlers.affiliate_service.get_affiliate_by_code(&affiliate_code).await {
                        let _ = handlers.affiliate_service.track_referral_click(
                            &affiliate_code,
                            "0.0.0.0",
                            None,
                            Some("api".to_string()),
                            Some("direct".to_string()),
                            Some(format!("{}_plans", plan_type))
                        ).await;
                    }
                }

                Ok(Json(PlansResponse {
                    success: true,
                    data: plans,
                    message: Some(format!("{} plans retrieved successfully", plan_type)),
                }))
            }
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    message: format!("Failed to fetch {} plans: {}", plan_type, e),
                }),
            )),
        }
    }

    /// GET /api/v1/plans/details/{plan_id}
    pub async fn get_plan_details(
        State(handlers): State<Arc<Self>>,
        Path(plan_id): Path<i32>,
        Query(params): Query<PlansQueryParams>,
    ) -> Result<Json<PlanResponse>, (StatusCode, Json<ErrorResponse>)> {
        match handlers.plan_service.get_plan(plan_id).await {
            Ok(Some(plan)) => {
                // Track affiliate referral if provided
                if let Some(affiliate_code) = params.affiliate_code {
                    if let Ok(Some(_affiliate)) = handlers.affiliate_service.get_affiliate_by_code(&affiliate_code).await {
                        let _ = handlers.affiliate_service.track_referral_click(
                            &affiliate_code,
                            "0.0.0.0",
                            None,
                            Some("api".to_string()),
                            Some("direct".to_string()),
                            Some(format!("plan_{}", plan_id))
                        ).await;
                    }
                }

                Ok(Json(PlanResponse {
                    success: true,
                    data: Some(plan),
                    message: Some("Plan details retrieved successfully".to_string()),
                }))
            }
            Ok(None) => Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    success: false,
                    message: "Plan not found".to_string(),
                }),
            )),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    message: format!("Failed to fetch plan: {}", e),
                }),
            )),
        }
    }

    /// GET /api/v1/plans/calculate-price/{plan_id}
    pub async fn calculate_effective_price(
        State(handlers): State<Arc<Self>>,
        Path(plan_id): Path<i32>,
        Query(params): Query<PlansQueryParams>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
        match handlers.plan_service.calculate_effective_price(plan_id, None).await {
            Ok(effective_price) => {
                let mut response = serde_json::json!({
                    "success": true,
                    "data": {
                        "plan_id": plan_id,
                        "effective_price": effective_price,
                        "currency": "USDT"
                    }
                });

                // Add affiliate commission info if requested
                if let Some(affiliate_code) = params.affiliate_code {
                    if let Ok(Some(affiliate)) = handlers.affiliate_service.get_affiliate_by_code(&affiliate_code).await {
                        if let Ok(commission_amount) = handlers.affiliate_service.calculate_commission(
                            affiliate.id,
                            plan_id,
                            effective_price,
                            "preview".to_string()
                        ).await {
                            response["data"]["affiliate"] = serde_json::json!({
                                "code": affiliate_code,
                                "commission_rate": commission_amount.commission_rate,
                                "commission_amount": commission_amount.commission_amount
                            });
                        }
                    }
                }

                Ok(Json(response))
            }
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    message: format!("Failed to calculate price: {}", e),
                }),
            )),
        }
    }

    /// POST /api/v1/plans/validate-discount
    pub async fn validate_discount_code(
        State(handlers): State<Arc<Self>>,
        Json(payload): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
        let code = payload.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    message: "Missing 'code' field".to_string(),
                })
            ))?;

        let plan_id = payload.get("plan_id")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .ok_or_else(|| (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    message: "Missing 'plan_id' field".to_string(),
                })
            ))?;

        let user_id = payload.get("user_id")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);

        match handlers.promotion_service.validate_discount_code(code, plan_id, user_id).await {
            Ok(validation) => {
                Ok(Json(serde_json::json!({
                    "success": true,
                    "data": validation
                })))
            }
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    message: format!("Failed to validate discount code: {}", e),
                }),
            )),
        }
    }
}

pub fn create_plans_router(db_pool: Arc<DbPool>) -> Router {
    let cache = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            CacheFactory::with_fallback_arc().await
        })
    });

    let handlers = Arc::new(PlansHandlers::new(db_pool, cache));

    Router::new()
        .route("/", get(PlansHandlers::get_plans))
        .route("/:plan_type", get(PlansHandlers::get_plans_by_type))
        .route("/details/:plan_id", get(PlansHandlers::get_plan_details))
        .route("/calculate-price/:plan_id", get(PlansHandlers::calculate_effective_price))
        .route("/validate-discount", post(PlansHandlers::validate_discount_code))
        .with_state(handlers)
}