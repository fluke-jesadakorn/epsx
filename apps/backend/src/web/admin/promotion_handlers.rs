use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json as JsonResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::web::auth::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePromotionRequest {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub discount_type: String,
    #[schema(value_type = String)]
    pub discount_value: Decimal,
    #[schema(value_type = Option<String>)]
    pub max_discount_amount: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub min_purchase_amount: Option<Decimal>,
    pub usage_limit: Option<i32>,
    pub is_active: bool,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub applicable_plans: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PromotionResponse {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: String,
    pub max_discount_amount: Option<String>,
    pub min_purchase_amount: Option<String>,
    pub usage_limit: Option<i32>,
    pub current_usage: i32,
    pub is_active: bool,
    pub start_date: String,
    pub end_date: Option<String>,
    pub applicable_plans: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub total_revenue: String,
    pub conversion_rate: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePromotionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    #[schema(value_type = Option<String>)]
    pub discount_value: Option<Decimal>,
    pub usage_limit: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/admin/promotions",
    responses(
        (status = 200, description = "Promotions retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-promotions",
    security(("bearerAuth" = []))
)]
pub async fn list_promotions_handler(
    State(app_state): State<AppState>,
    Query(_query): Query<HashMap<String, String>>,
) -> Result<JsonResponse<serde_json::Value>, StatusCode> {
    let query = sqlx::query!(
        r#"
        SELECT
            pc.id,
            pc.name,
            pc.description,
            pc.campaign_type,
            pc.is_active,
            pc.start_date,
            pc.end_date,
            pc.created_at,
            pc.updated_at,
            COALESCE(COUNT(DISTINCT pp.id), 0)::int as plan_count
        FROM promotional_campaigns pc
        LEFT JOIN plan_promotions pp ON pp.campaign_id = pc.id
        GROUP BY pc.id
        ORDER BY pc.created_at DESC
        "#
    )
    .fetch_all(app_state.db_pool.as_ref())
    .await;

    let campaigns = match query {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch promotional campaigns");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let promotions: Vec<PromotionResponse> = campaigns
        .into_iter()
        .map(|row| {
            let discount_type = row.campaign_type;
            let discount_value = "10";

            PromotionResponse {
                id: row.id,
                name: row.name.clone(),
                code: row.name.to_uppercase().replace(' ', ""),
                description: row.description.clone(),
                discount_type,
                discount_value: discount_value.to_string(),
                max_discount_amount: Some("100".to_string()),
                min_purchase_amount: Some("0".to_string()),
                usage_limit: Some(1000),
                current_usage: 0,
                is_active: row.is_active.unwrap_or(true),
                start_date: row.start_date.to_rfc3339(),
                end_date: row.end_date.map(|d| d.to_rfc3339()),
                applicable_plans: vec![],
                created_at: row.created_at.unwrap_or_else(Utc::now).to_rfc3339(),
                updated_at: row.updated_at.unwrap_or_else(Utc::now).to_rfc3339(),
                total_revenue: "0".to_string(),
                conversion_rate: 0.0,
            }
        })
        .collect();

    Ok(JsonResponse(serde_json::json!({
        "success": true,
        "data": {
            "promotions": promotions,
            "total_count": promotions.len(),
            "has_more": false
        },
        "message": "Promotions retrieved successfully"
    })))
}

#[utoipa::path(
    post,
    path = "/admin/promotions",
    request_body = CreatePromotionRequest,
    responses(
        (status = 200, description = "Promotion created successfully", body = PromotionResponse),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-promotions",
    security(("bearerAuth" = []))
)]
pub async fn create_promotion_handler(
    State(app_state): State<AppState>,
    Json(request): Json<CreatePromotionRequest>,
) -> Result<JsonResponse<PromotionResponse>, StatusCode> {
    let result = sqlx::query!(
        r#"
        INSERT INTO promotional_campaigns (
            name, description, campaign_type, is_active,
            start_date, end_date, priority
        )
        VALUES ($1, $2, $3, $4, $5, $6, 1)
        RETURNING id, created_at, updated_at
        "#,
        request.name,
        request.description,
        request.discount_type,
        request.is_active,
        request.start_date,
        request.end_date,
    )
    .fetch_one(app_state.db_pool.as_ref())
    .await;

    match result {
        Ok(row) => {
            let response = PromotionResponse {
                id: row.id,
                name: request.name,
                code: request.code,
                description: request.description,
                discount_type: request.discount_type,
                discount_value: request.discount_value.to_string(),
                max_discount_amount: request.max_discount_amount.map(|d| d.to_string()),
                min_purchase_amount: request.min_purchase_amount.map(|d| d.to_string()),
                usage_limit: request.usage_limit,
                current_usage: 0,
                is_active: request.is_active,
                start_date: request.start_date.to_rfc3339(),
                end_date: request.end_date.map(|d| d.to_rfc3339()),
                applicable_plans: request.applicable_plans,
                created_at: row.created_at.unwrap_or_else(Utc::now).to_rfc3339(),
                updated_at: row.updated_at.unwrap_or_else(Utc::now).to_rfc3339(),
                total_revenue: "0".to_string(),
                conversion_rate: 0.0,
            };

            Ok(JsonResponse(response))
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to create promotional campaign");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = "/admin/promotions/{id}",
    responses(
        (status = 200, description = "Promotion retrieved successfully", body = PromotionResponse),
        (status = 404, description = "Promotion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-promotions",
    security(("bearerAuth" = []))
)]
pub async fn get_promotion_handler(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<JsonResponse<PromotionResponse>, StatusCode> {
    let result = sqlx::query!(
        r#"
        SELECT
            id, name, description, campaign_type, is_active,
            start_date, end_date, created_at, updated_at
        FROM promotional_campaigns
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(app_state.db_pool.as_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let response = PromotionResponse {
                id: row.id,
                name: row.name.clone(),
                code: row.name.to_uppercase().replace(' ', ""),
                description: row.description,
                discount_type: row.campaign_type,
                discount_value: "10".to_string(),
                max_discount_amount: Some("100".to_string()),
                min_purchase_amount: Some("0".to_string()),
                usage_limit: Some(1000),
                current_usage: 0,
                is_active: row.is_active.unwrap_or(true),
                start_date: row.start_date.to_rfc3339(),
                end_date: row.end_date.map(|d| d.to_rfc3339()),
                applicable_plans: vec![],
                created_at: row.created_at.unwrap_or_else(Utc::now).to_rfc3339(),
                updated_at: row.updated_at.unwrap_or_else(Utc::now).to_rfc3339(),
                total_revenue: "0".to_string(),
                conversion_rate: 0.0,
            };
            Ok(JsonResponse(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            tracing::error!(error = %err, promotion_id = id, "Failed to fetch promotion");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    put,
    path = "/admin/promotions/{id}",
    request_body = UpdatePromotionRequest,
    responses(
        (status = 200, description = "Promotion updated successfully", body = PromotionResponse),
        (status = 404, description = "Promotion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-promotions",
    security(("bearerAuth" = []))
)]
pub async fn update_promotion_handler(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
    Json(request): Json<UpdatePromotionRequest>,
) -> Result<JsonResponse<PromotionResponse>, StatusCode> {
    let result = sqlx::query!(
        r#"
        UPDATE promotional_campaigns
        SET
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            is_active = COALESCE($3, is_active),
            updated_at = NOW()
        WHERE id = $4
        RETURNING id, name, description, campaign_type, is_active,
                  start_date, end_date, created_at, updated_at
        "#,
        request.name,
        request.description,
        request.is_active,
        id
    )
    .fetch_optional(app_state.db_pool.as_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let response = PromotionResponse {
                id: row.id,
                name: row.name.clone(),
                code: row.name.to_uppercase().replace(' ', ""),
                description: row.description,
                discount_type: row.campaign_type,
                discount_value: request.discount_value.map(|d| d.to_string()).unwrap_or_else(|| "10".to_string()),
                max_discount_amount: Some("100".to_string()),
                min_purchase_amount: Some("0".to_string()),
                usage_limit: request.usage_limit.or(Some(1000)),
                current_usage: 0,
                is_active: row.is_active.unwrap_or(true),
                start_date: row.start_date.to_rfc3339(),
                end_date: row.end_date.map(|d| d.to_rfc3339()),
                applicable_plans: vec![],
                created_at: row.created_at.unwrap_or_else(Utc::now).to_rfc3339(),
                updated_at: row.updated_at.unwrap_or_else(Utc::now).to_rfc3339(),
                total_revenue: "0".to_string(),
                conversion_rate: 0.0,
            };
            Ok(JsonResponse(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            tracing::error!(error = %err, promotion_id = id, "Failed to update promotion");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/admin/promotions/{id}",
    responses(
        (status = 200, description = "Promotion deleted successfully"),
        (status = 404, description = "Promotion not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-promotions",
    security(("bearerAuth" = []))
)]
pub async fn delete_promotion_handler(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<JsonResponse<serde_json::Value>, StatusCode> {
    let result = sqlx::query!(
        "DELETE FROM promotional_campaigns WHERE id = $1 RETURNING id",
        id
    )
    .fetch_optional(app_state.db_pool.as_ref())
    .await;

    match result {
        Ok(Some(_)) => Ok(JsonResponse(serde_json::json!({
            "success": true,
            "message": "Promotion deleted successfully"
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            tracing::error!(error = %err, promotion_id = id, "Failed to delete promotion");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
