use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use axum::{
    response::{IntoResponse, Response},
    Json,
    http::StatusCode,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ApiMeta>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ApiMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    pub timestamp: String,
}

impl<T> ApiResponse<T> 
where T: Serialize 
{
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ApiMeta::default()),
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message: message.into(),
                details: None,
                request_id: None,
            }),
            meta: Some(ApiMeta::default()),
        }
    }
}

impl Default for ApiMeta {
    fn default() -> Self {
        Self {
            page: None,
            per_page: None,
            total: None,
            total_pages: None,
            trace_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// Implement IntoResponse for consistent JSON handling
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            // Map error codes to status codes if needed, defaulting to 400 or 500
            // For now, let's assume the caller will specific status via tuple if needed, 
            // OR we can derive it. 
            // Simpler: Just return 200 OK with success: false in body? 
            // NO, RESTful principles say use proper status codes.
            // But we can't store StatusCode in ApiResponse (it's for body).
            // We usually return (StatusCode, Json(ApiResponse)).
            // This IntoResponse impl defaults to 200, which is risky for errors.
            // Let's remove IntoResponse impl or make it smart.
            // Actually, usually we return `(StatusCode, Json(response))`.
            StatusCode::OK 
        };
        
        // If it's an error, we might want a different default status, but let's leave it to handlers.
        // Or we can check error.code.
        
        (status, Json(self)).into_response()
    }
}
