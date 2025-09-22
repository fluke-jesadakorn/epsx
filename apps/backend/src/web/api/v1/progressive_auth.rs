/**
 * Progressive Authentication API Endpoints
 * 
 * Supports three-tier authentication:
 * - PUBLIC: No auth needed (analytics, home, pricing)
 * - CONNECTED: Wallet address only (preferences, watchlists)
 * - AUTHENTICATED: Full signature required (payments, sensitive operations)
 */

use axum::{
    extract::{Query, Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::infrastructure::container::AppContainer;

// ===== DTOs =====

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletPreferencesDto {
    pub wallet_address: String,
    pub preferences: Value,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletWatchlistDto {
    pub wallet_address: String,
    pub symbol: String,
    pub added_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub wallet_address: String,
    pub preferences: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddToWatchlistRequest {
    pub wallet_address: String,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryParams {
    pub wallet_address: Option<String>,
}

// ===== CONNECTED LEVEL HANDLERS (Wallet Address Only) =====

/// Get wallet preferences - requires wallet address only
pub async fn get_wallet_preferences(
    Query(params): Query<QueryParams>,
    State(_container): State<Arc<AppContainer>>,
) -> Result<Json<Value>, StatusCode> {
    let wallet_address = params.wallet_address
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Validate wallet address format
    if !is_valid_wallet_address(&wallet_address) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // For now, return mock preferences data
    // In production, this would query the database
    let preferences = WalletPreferencesDto {
        wallet_address: wallet_address.clone(),
        preferences: json!({
            "theme": "dark",
            "default_country": "US",
            "default_sector": "Technology",
            "notifications_enabled": true,
            "display_currency": "USD"
        }),
        updated_at: chrono::Utc::now(),
    };

    Ok(Json(json!({
        "success": true,
        "data": preferences
    })))
}

/// Update wallet preferences - requires wallet address only
pub async fn update_wallet_preferences(
    State(_container): State<Arc<AppContainer>>,
    Json(request): Json<UpdatePreferencesRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Validate wallet address format
    if !is_valid_wallet_address(&request.wallet_address) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // For now, return success
    // In production, this would update the database
    let updated_preferences = WalletPreferencesDto {
        wallet_address: request.wallet_address,
        preferences: request.preferences,
        updated_at: chrono::Utc::now(),
    };

    Ok(Json(json!({
        "success": true,
        "message": "Preferences updated successfully",
        "data": updated_preferences
    })))
}

/// Get wallet watchlist - requires wallet address only
pub async fn get_wallet_watchlist(
    Query(params): Query<QueryParams>,
    State(_container): State<Arc<AppContainer>>,
) -> Result<Json<Value>, StatusCode> {
    let wallet_address = params.wallet_address
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Validate wallet address format
    if !is_valid_wallet_address(&wallet_address) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // For now, return mock watchlist data
    // In production, this would query the database
    let watchlist = vec![
        WalletWatchlistDto {
            wallet_address: wallet_address.clone(),
            symbol: "AAPL".to_string(),
            added_at: chrono::Utc::now() - chrono::Duration::days(5),
        },
        WalletWatchlistDto {
            wallet_address: wallet_address.clone(),
            symbol: "GOOGL".to_string(),
            added_at: chrono::Utc::now() - chrono::Duration::days(2),
        },
        WalletWatchlistDto {
            wallet_address: wallet_address.clone(),
            symbol: "MSFT".to_string(),
            added_at: chrono::Utc::now() - chrono::Duration::hours(6),
        },
    ];

    Ok(Json(json!({
        "success": true,
        "data": watchlist
    })))
}

/// Add symbol to wallet watchlist - requires wallet address only
pub async fn add_to_wallet_watchlist(
    State(_container): State<Arc<AppContainer>>,
    Json(request): Json<AddToWatchlistRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Validate wallet address format
    if !is_valid_wallet_address(&request.wallet_address) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate symbol format (basic validation)
    if request.symbol.is_empty() || request.symbol.len() > 10 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // For now, return success
    // In production, this would insert into the database
    let watchlist_item = WalletWatchlistDto {
        wallet_address: request.wallet_address,
        symbol: request.symbol,
        added_at: chrono::Utc::now(),
    };

    Ok(Json(json!({
        "success": true,
        "message": "Symbol added to watchlist",
        "data": watchlist_item
    })))
}

/// Remove symbol from wallet watchlist - requires wallet address only
pub async fn remove_from_wallet_watchlist(
    Path((wallet_address, symbol)): Path<(String, String)>,
    State(_container): State<Arc<AppContainer>>,
) -> Result<Json<Value>, StatusCode> {
    // Validate wallet address format
    if !is_valid_wallet_address(&wallet_address) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // For now, return success
    // In production, this would delete from the database
    Ok(Json(json!({
        "success": true,
        "message": format!("Symbol {} removed from watchlist", symbol)
    })))
}

/// Get personalized analytics based on wallet address - requires wallet address only
pub async fn get_personalized_analytics(
    Query(params): Query<QueryParams>,
    State(_container): State<Arc<AppContainer>>,
) -> Result<Json<Value>, StatusCode> {
    let wallet_address = params.wallet_address
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Validate wallet address format
    if !is_valid_wallet_address(&wallet_address) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // For now, return mock personalized analytics
    // In production, this would combine public analytics with user preferences
    Ok(Json(json!({
        "success": true,
        "data": {
            "wallet_address": wallet_address,
            "personalized_rankings": [
                {
                    "symbol": "AAPL",
                    "rank": 1,
                    "eps_score": 95.6,
                    "in_watchlist": true
                },
                {
                    "symbol": "GOOGL", 
                    "rank": 2,
                    "eps_score": 92.8,
                    "in_watchlist": true
                }
            ],
            "preferred_sectors": ["Technology", "Healthcare"],
            "preferred_countries": ["US", "CA"],
            "last_updated": chrono::Utc::now()
        }
    })))
}

// ===== UTILITY FUNCTIONS =====

/// Basic wallet address validation (Ethereum format)
fn is_valid_wallet_address(address: &str) -> bool {
    // Basic Ethereum address validation: 0x followed by 40 hex characters
    address.len() == 42 && 
    address.starts_with("0x") && 
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

// ===== ROUTER CREATION =====

/// Create progressive authentication routes
pub fn create_progressive_auth_routes(container: Arc<AppContainer>) -> Router {
    Router::new()
        // CONNECTED level endpoints (wallet address only, no signature)
        .route("/connected/preferences", get(get_wallet_preferences))
        .route("/connected/preferences", put(update_wallet_preferences))
        .route("/connected/watchlist", get(get_wallet_watchlist))
        .route("/connected/watchlist", post(add_to_wallet_watchlist))
        .route("/connected/watchlist/:wallet_address/:symbol", delete(remove_from_wallet_watchlist))
        .route("/connected/analytics", get(get_personalized_analytics))
        
        // Health endpoint for progressive auth system
        .route("/health", get(|| async {
            Json(json!({
                "status": "healthy",
                "service": "progressive-auth",
                "timestamp": chrono::Utc::now(),
                "levels": ["public", "connected", "authenticated"]
            }))
        }))
        .with_state(container)
}