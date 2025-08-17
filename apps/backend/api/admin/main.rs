use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let path = req.uri().path();
    let method = req.method().as_str();
    
    // Route to specific admin endpoints
    let response_body = match (method, path) {
        ("GET", "/api/admin/users") => handle_get_users(req).await?,
        ("POST", "/api/admin/users") => handle_create_user(req).await?,
        ("PUT", "/api/admin/users") => handle_update_user(req).await?,
        ("DELETE", "/api/admin/users") => handle_delete_user(req).await?,
        ("POST", "/api/admin/users/bulk/assign-modules") => handle_bulk_assign_modules(req).await?,
        ("GET", "/api/admin/modules/user") => handle_get_user_modules(req).await?,
        ("POST", "/api/admin/admin-modules/assign") => handle_assign_admin_modules(req).await?,
        ("GET", "/api/admin/admin-modules") => handle_get_admin_modules(req).await?,
        ("POST", "/api/admin/stock-ranking/assignments") => handle_stock_ranking_assignments(req).await?,
        ("POST", "/api/admin/stock-ranking/assignments/extend") => handle_extend_assignment(req).await?,
        ("POST", "/api/admin/stock-ranking/assignments/revoke") => handle_revoke_assignment(req).await?,
        _ => json!({
            "error": "Not Found",
            "message": format!("Admin endpoint not found: {} {}", method, path)
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

// Admin endpoint handlers (placeholder implementations)
async fn handle_get_users(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "users": [],
        "total_count": 0,
        "page": 1,
        "per_page": 20,
        "message": "Get users endpoint - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_create_user(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "message": "Create user endpoint - implementation needed",
        "status": "placeholder",
        "user_id": "placeholder-user-id"
    }))
}

async fn handle_update_user(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "message": "Update user endpoint - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_delete_user(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "message": "Delete user endpoint - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_bulk_assign_modules(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "message": "Bulk assign modules endpoint - implementation needed",
        "status": "placeholder",
        "assigned_count": 0
    }))
}

async fn handle_get_user_modules(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "modules": [],
        "user_id": "unknown",
        "message": "Get user modules endpoint - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_assign_admin_modules(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "message": "Assign admin modules endpoint - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_get_admin_modules(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "modules": [],
        "message": "Get admin modules endpoint - implementation needed", 
        "status": "placeholder"
    }))
}

async fn handle_stock_ranking_assignments(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "assignments": [],
        "message": "Stock ranking assignments endpoint - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_extend_assignment(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "message": "Extend assignment endpoint - implementation needed",
        "status": "placeholder"
    }))
}

async fn handle_revoke_assignment(_req: Request) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "message": "Revoke assignment endpoint - implementation needed",
        "status": "placeholder"
    }))
}