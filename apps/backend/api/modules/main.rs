use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let path = req.uri().path();
    let method = req.method().as_str();
    
    // Route to specific modules endpoints
    let response_body = match (method, path) {
        ("GET", "/api/modules/stock-ranking") => handle_stock_ranking_module(req).await?,
        ("GET", "/api/modules/portfolio-analysis") => handle_portfolio_analysis_module(req).await?,
        ("GET", "/api/modules/trading-signals") => handle_trading_signals_module(req).await?,
        ("GET", "/api/modules/market-data") => handle_market_data_module(req).await?,
        ("GET", "/api/modules/user/permissions") => handle_user_module_permissions(req).await?,
        ("POST", "/api/modules/user/assign") => handle_assign_user_module(req).await?,
        ("DELETE", "/api/modules/user/revoke") => handle_revoke_user_module(req).await?,
        _ => json!({
            "error": "Not Found",
            "message": format!("Modules endpoint not found: {} {}", method, path)
        })
    };

    let status = if response_body.get("error").is_some() { 
        StatusCode::NOT_FOUND 
    } else { 
        StatusCode::OK 
    };

    let response = Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .body(response_body.to_string().into())?;

    Ok(response)
}

// Modules endpoint handlers (placeholder implementations)
async fn handle_stock_ranking_module(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "module": "stock-ranking",
        "data": [],
        "rankings": [],
        "total_count": 0,
        "message": "Stock ranking module - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_portfolio_analysis_module(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "module": "portfolio-analysis",
        "analysis": {},
        "recommendations": [],
        "message": "Portfolio analysis module - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_trading_signals_module(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "module": "trading-signals",
        "signals": [],
        "indicators": [],
        "message": "Trading signals module - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_market_data_module(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "module": "market-data",
        "data": {},
        "last_updated": chrono::Utc::now().to_rfc3339(),
        "message": "Market data module - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_user_module_permissions(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "permissions": [],
        "modules": [],
        "user_id": "unknown",
        "message": "User module permissions - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_assign_user_module(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "message": "Assign user module endpoint - implementation needed",
        "status": "placeholder",
        "assignment_id": "placeholder-assignment-id"
    }))
}

async fn handle_revoke_user_module(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "message": "Revoke user module endpoint - implementation needed",
        "status": "placeholder"
    }))
}