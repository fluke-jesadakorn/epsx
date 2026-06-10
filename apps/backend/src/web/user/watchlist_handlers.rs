use axum::{
    extract::{Path, State},
    Extension, Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::web::{
    auth::AppState,
    middleware::OpenIDUserContext,
    responses::UnifiedApiResponse,
};
use crate::schemas::primary::user_watchlist;

// ============================================================================
// TYPES
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct WatchlistResponse {
    pub symbols: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddWatchlistRequest {
    pub symbol: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = user_watchlist)]
struct NewWatchlistEntry {
    wallet_address: String,
    symbol: String,
}

// ============================================================================
// HELPERS
// ============================================================================

async fn fetch_watchlist(
    pool: &crate::prelude::TlsPool,
    wallet: &str,
) -> Result<Vec<String>, String> {
    let mut conn = pool.get().await.map_err(|e| e.to_string())?;

    user_watchlist::table
        .filter(user_watchlist::wallet_address.eq(wallet))
        .order(user_watchlist::added_at.asc())
        .select(user_watchlist::symbol)
        .load::<String>(&mut conn)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Get user's watchlist symbols
pub async fn get_watchlist(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
) -> Result<Json<UnifiedApiResponse<WatchlistResponse>>, Json<UnifiedApiResponse<()>>> {
    match fetch_watchlist(&app_state.db_pool, &ctx.wallet_address).await {
        Ok(symbols) => Ok(Json(UnifiedApiResponse::success(WatchlistResponse { symbols }))),
        Err(e) => {
            error!("Failed to fetch watchlist: {}", e);
            Ok(Json(UnifiedApiResponse::success(WatchlistResponse { symbols: vec![] })))
        }
    }
}

/// Add symbol to watchlist
pub async fn add_to_watchlist(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Json(body): Json<AddWatchlistRequest>,
) -> Result<Json<UnifiedApiResponse<WatchlistResponse>>, Json<UnifiedApiResponse<()>>> {
    let symbol = body.symbol.to_uppercase().trim().to_string();
    if symbol.is_empty() || symbol.len() > 20 {
        return Err(Json(UnifiedApiResponse::error(400, "Invalid symbol", "Symbol must be 1-20 characters")));
    }

    let mut conn = app_state.db_pool.get().await.map_err(|e| {
        error!("DB connection error: {}", e);
        Json(UnifiedApiResponse::error(500, "Database error", "Failed to connect"))
    })?;

    let entry = NewWatchlistEntry {
        wallet_address: ctx.wallet_address.clone(),
        symbol: symbol.clone(),
    };

    // INSERT ON CONFLICT DO NOTHING (idempotent)
    diesel::insert_into(user_watchlist::table)
        .values(&entry)
        .on_conflict((user_watchlist::wallet_address, user_watchlist::symbol))
        .do_nothing()
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to add to watchlist: {}", e);
            Json(UnifiedApiResponse::error(500, "Database error", &e.to_string()))
        })?;

    info!("Added {} to watchlist for {}", symbol, ctx.wallet_address);

    match fetch_watchlist(&app_state.db_pool, &ctx.wallet_address).await {
        Ok(symbols) => Ok(Json(UnifiedApiResponse::success(WatchlistResponse { symbols }))),
        Err(e) => {
            error!("Failed to fetch watchlist after add: {}", e);
            Ok(Json(UnifiedApiResponse::success(WatchlistResponse { symbols: vec![symbol] })))
        }
    }
}

/// Remove symbol from watchlist
pub async fn remove_from_watchlist(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(symbol): Path<String>,
) -> Result<Json<UnifiedApiResponse<WatchlistResponse>>, Json<UnifiedApiResponse<()>>> {
    let symbol = symbol.to_uppercase();

    let mut conn = app_state.db_pool.get().await.map_err(|e| {
        error!("DB connection error: {}", e);
        Json(UnifiedApiResponse::error(500, "Database error", "Failed to connect"))
    })?;

    diesel::delete(
        user_watchlist::table
            .filter(user_watchlist::wallet_address.eq(&ctx.wallet_address))
            .filter(user_watchlist::symbol.eq(&symbol)),
    )
    .execute(&mut conn)
    .await
    .map_err(|e| {
        error!("Failed to remove from watchlist: {}", e);
        Json(UnifiedApiResponse::error(500, "Database error", &e.to_string()))
    })?;

    info!("Removed {} from watchlist for {}", symbol, ctx.wallet_address);

    match fetch_watchlist(&app_state.db_pool, &ctx.wallet_address).await {
        Ok(symbols) => Ok(Json(UnifiedApiResponse::success(WatchlistResponse { symbols }))),
        Err(e) => {
            error!("Failed to fetch watchlist after remove: {}", e);
            Ok(Json(UnifiedApiResponse::success(WatchlistResponse { symbols: vec![] })))
        }
    }
}
