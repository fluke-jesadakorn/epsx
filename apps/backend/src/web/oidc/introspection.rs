use chrono::{DateTime, Utc};// OpenID Connect Token Introspection Endpoint (RFC 7662)
// Implements standard token introspection with granular permission details

use axum::{
    extract::{State, Form},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

use crate::web::auth::AppState;
use crate::auth::jwt::Claims;
use crate::auth::permissions::{
    parse_permission_with_timestamp, 
    get_permission_expiry_time,
    hours_until_expiry
};

/// Standard token introspection request (RFC 7662)
#[derive(Debug, Deserialize)]
pub struct IntrospectionRequest {
    pub token: String,
    pub token_type_hint: Option<String>, // "access_token" or "refresh_token"
}

/// Standard token introspection response (RFC 7662) with granular extensions
#[derive(Debug, Serialize)]
pub struct IntrospectionResponse {
    // Standard RFC 7662 fields
    pub active: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,

    // Granular permission extensions (EPSX-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<PermissionDetail>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_count: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_admin_access: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_tier: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiring_permissions_count: Option<usize>,
}

/// Detailed permission information (granular extension)
#[derive(Debug, Serialize)]
pub struct PermissionDetail {
    pub permission: String,
    pub base_permission: String,
    pub expires_at: Option<i64>,
    pub permission_type: String, // "permanent" or "temporary"
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours_remaining: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

/// Token introspection error response
#[derive(Debug, Serialize)]
pub struct IntrospectionError {
    pub error: String,
    pub error_description: Option<String>,
}

/// POST /oauth/introspect - Token Introspection Endpoint
/// 
/// Standard OAuth 2.0 Token Introspection (RFC 7662) with granular permission details
/// 
/// Standard usage:
/// ```
/// POST /oauth/introspect
/// Content-Type: application/x-www-form-urlencoded
/// 
/// token=eyJhbGciOiJSUzI1NiIs...&token_type_hint=access_token
/// ```
/// 
/// Response includes granular permission details for admin clients
pub async fn introspect_token(
    State(app_state): State<AppState>,
    Form(request): Form<IntrospectionRequest>,
) -> Result<Json<IntrospectionResponse>, (StatusCode, Json<IntrospectionError>)> {
    tracing::debug!(
        token_hint = ?request.token_type_hint,
        "Token introspection request received"
    );

    // Extract JWT claims from token
    let claims = match extract_jwt_claims(&request.token) {
        Ok(claims) => claims,
        Err(e) => {
            tracing::debug!("Invalid token in introspection request: {}", e);
            return Ok(Json(IntrospectionResponse {
                active: false,
                scope: None,
                client_id: None,
                username: None,
                token_type: None,
                exp: None,
                iat: None,
                nbf: None,
                sub: None,
                aud: None,
                iss: None,
                jti: None,
                permissions: None,
                permission_count: None,
                has_admin_access: None,
                package_tier: None,
                expiring_permissions_count: None,
            }));
        }
    };

    // Token revocation check would be implemented here
    // For now, we'll assume the token is not revoked if it passes JWT validation
    tracing::debug!(jti = %claims.jti, "Token introspection - checking revocation status");

    // Check if token has expired
    let now = Utc::now().timestamp() as usize;
    if claims.exp <= now {
        tracing::debug!(
            jti = %claims.jti,
            exp = %claims.exp,
            now = %now,
            "Token has expired"
        );
        return Ok(Json(IntrospectionResponse {
            active: false,
            scope: None,
            client_id: None,
            username: None,
            token_type: None,
            exp: Some(claims.exp as i64),
            iat: Some(claims.iat as i64),
            nbf: Some(claims.nbf as i64),
            sub: Some(claims.sub),
            aud: Some(claims.aud),
            iss: Some(claims.iss),
            jti: Some(claims.jti),
            permissions: None,
            permission_count: None,
            has_admin_access: None,
            package_tier: None,
            expiring_permissions_count: None,
        }));
    }

    // Get granular permission details
    let permission_details = get_granular_permission_details(&app_state, &claims).await;

    // Build standard response with granular extensions
    let response = IntrospectionResponse {
        active: true,
        scope: Some("openid profile email".to_string()),
        client_id: Some("epsx-web".to_string()),
        username: Some(claims.email.clone()),
        token_type: Some("Bearer".to_string()),
        exp: Some(claims.exp as i64),
        iat: Some(claims.iat as i64),
        nbf: Some(claims.nbf as i64),
        sub: Some(claims.sub.clone()),
        aud: Some(claims.aud.clone()),
        iss: Some(claims.iss.clone()),
        jti: Some(claims.jti.clone()),
        
        // Granular extensions
        permissions: permission_details.permissions,
        permission_count: permission_details.count,
        has_admin_access: permission_details.has_admin,
        package_tier: permission_details.package_tier,
        expiring_permissions_count: permission_details.expiring_count,
    };

    tracing::info!(
        jti = %claims.jti,
        sub = %claims.sub,
        active = true,
        permission_count = ?permission_details.count,
        "Token introspection completed"
    );

    Ok(Json(response))
}

/// Granular permission details container
#[derive(Debug)]
struct GranularPermissionDetails {
    permissions: Option<Vec<PermissionDetail>>,
    count: Option<usize>,
    has_admin: Option<bool>,
    package_tier: Option<String>,
    expiring_count: Option<usize>,
}

/// Get granular permission details for the token
async fn get_granular_permission_details(
    app_state: &AppState,
    claims: &Claims,
) -> GranularPermissionDetails {
    // Get user by Firebase UID from JWT sub claim
    let firebase_uid = match crate::domain::user_management::value_objects::FirebaseUid::new(&claims.sub) {
        Ok(uid) => uid,
        Err(e) => {
            tracing::warn!(
                sub = %claims.sub,
                error = %e,
                "Invalid Firebase UID in JWT token for introspection"
            );
            return GranularPermissionDetails {
                permissions: None,
                count: None,
                has_admin: None,
                package_tier: None,
                expiring_count: None,
            };
        }
    };
    
    let user = match app_state.user_repo.find_by_firebase_uid(&firebase_uid).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!(
                sub = %claims.sub,
                "User not found for introspection"
            );
            return GranularPermissionDetails {
                permissions: None,
                count: None,
                has_admin: None,
                package_tier: None,
                expiring_count: None,
            };
        }
        Err(e) => {
            tracing::error!(
                sub = %claims.sub,
                error = %e,
                "Database error during introspection"
            );
            return GranularPermissionDetails {
                permissions: None,
                count: None,
                has_admin: None,
                package_tier: None,
                expiring_count: None,
            };
        }
    };

    // Get user permissions from User aggregate
    let user_permissions = user.active_permissions();

    // Build detailed permission information
    let mut permission_details = Vec::new();
    let mut expiring_count = 0;
    let mut has_admin_access = false;

    for permission in &user_permissions {
        let (base_permission, timestamp) = parse_permission_with_timestamp(permission);
        let _expiry_time = get_permission_expiry_time(permission);
        let hours_remaining = hours_until_expiry(permission);
        
        // Parse permission components
        let parts: Vec<&str> = base_permission.split(':').collect();
        let (platform, resource, action) = if parts.len() >= 3 {
            (Some(parts[0].to_string()), Some(parts[1].to_string()), Some(parts[2].to_string()))
        } else {
            (None, None, None)
        };

        // Check for admin access
        if base_permission.starts_with("admin:") {
            has_admin_access = true;
        }

        // Count expiring permissions (within 24 hours)
        if let Some(hours) = hours_remaining {
            let hours_f64 = hours as f64;
            if hours_f64 <= 24.0 && hours_f64 > 0.0 {
                expiring_count += 1;
            }
        }

        permission_details.push(PermissionDetail {
            permission: permission.clone(),
            base_permission,
            expires_at: timestamp,
            permission_type: if timestamp.is_some() { "temporary" } else { "permanent" }.to_string(),
            hours_remaining: hours_remaining.map(|h| h as f64),
            platform,
            resource,
            action,
        });
    }

    // Derive package tier from permissions
    let package_tier = derive_package_tier_from_permissions(&user_permissions);

    GranularPermissionDetails {
        permissions: if permission_details.is_empty() { None } else { Some(permission_details) },
        count: if user_permissions.is_empty() { None } else { Some(user_permissions.len()) },
        has_admin: Some(has_admin_access),
        package_tier: Some(package_tier),
        expiring_count: if expiring_count == 0 { None } else { Some(expiring_count) },
    }
}

/// Derive package tier from user permissions
fn derive_package_tier_from_permissions(permissions: &[String]) -> String {
    // Use existing logic from auth/permissions.rs
    use crate::auth::permissions::extract_ranking_limit;
    let ranking_limit = extract_ranking_limit(permissions);
    
    match ranking_limit {
        -1 => "ENTERPRISE".to_string(),
        76..=150 => "PLATINUM".to_string(),
        31..=75 => "GOLD".to_string(),
        11..=30 => "SILVER".to_string(),
        4..=10 => "BRONZE".to_string(),
        _ => "FREE".to_string(),
    }
}

/// Extract JWT claims from token string
fn extract_jwt_claims(token: &str) -> Result<Claims, String> {
    use crate::config::env::get_env_var;
    
    let jwt_secret = get_env_var("NEXTAUTH_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
    let mut validation = Validation::new(Algorithm::HS256);
    
    validation.set_audience(&["epsx-api", "epsx-ecosystem"]);
    let issuer_url = get_env_var("OIDC_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    validation.set_issuer(&[&issuer_url]);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| format!("JWT validation failed: {}", e))?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_introspection_request_deserialize() {
        let form_data = "token=abc123&token_type_hint=access_token";
        let request: IntrospectionRequest = serde_urlencoded::from_str(form_data).unwrap();
        
        assert_eq!(request.token, "abc123");
        assert_eq!(request.token_type_hint.unwrap(), "access_token");
    }

    #[test]
    fn test_derive_package_tier() {
        let enterprise_permissions = vec!["epsx:rankings:view:unlimited".to_string()];
        assert_eq!(derive_package_tier_from_permissions(&enterprise_permissions), "ENTERPRISE");
        
        let gold_permissions = vec!["epsx:rankings:view:50".to_string()];
        assert_eq!(derive_package_tier_from_permissions(&gold_permissions), "GOLD");
        
        let free_permissions = vec!["epsx:analytics:view".to_string()];
        assert_eq!(derive_package_tier_from_permissions(&free_permissions), "FREE");
    }

    #[test] 
    fn test_permission_detail_serialization() {
        let detail = PermissionDetail {
            permission: "admin:users:modify:1703980800".to_string(),
            base_permission: "admin:users:modify".to_string(),
            expires_at: Some(1703980800),
            permission_type: "temporary".to_string(),
            hours_remaining: Some(6.5),
            platform: Some("admin".to_string()),
            resource: Some("users".to_string()),
            action: Some("modify".to_string()),
        };
        
        let json = serde_json::to_string(&detail).unwrap();
        assert!(json.contains("admin:users:modify"));
        assert!(json.contains("temporary"));
        assert!(json.contains("6.5"));
    }
}