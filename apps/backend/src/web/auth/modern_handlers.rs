use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn, error};

// use crate::auth::{UserClaimsInput, JWT_SERVICE};

/**
 * Modern Auth.js v5 handler implementations
 * Provides endpoints for Auth.js frontend integration
 */

#[derive(Debug, Deserialize)]
pub struct UserClaimsRequest {
    pub email: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserClaimsResponse {
    pub admin_modules: Vec<String>,
    pub permissions: Vec<String>,
    pub package_tier: String,
    pub role: String,
    pub firebase_uid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertUserRequest {
    pub email: String,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub provider: String,
    pub provider_id: String,
}

#[derive(Debug, Serialize)]
pub struct UpsertUserResponse {
    pub user_id: String,
    pub email: String,
    pub created: bool,
}

/**
 * Get user claims for JWT token generation
 * Called by Auth.js during login process
 */
#[axum::debug_handler]
pub async fn get_user_claims(
    State(pool): State<PgPool>,
    Json(request): Json<UserClaimsRequest>,
) -> Result<Json<UserClaimsResponse>, StatusCode> {
    info!("Getting user claims for: {:?}", request);

    let user_data = if let Some(email) = &request.email {
        get_user_by_email(&pool, email).await
    } else if let Some(user_id) = &request.user_id {
        get_user_by_id(&pool, user_id).await
    } else {
        warn!("No email or user_id provided for user claims request");
        return Err(StatusCode::BAD_REQUEST);
    };

    let user = match user_data {
        Ok(Some(user)) => user,
        Ok(None) => {
            info!("User not found, returning default claims");
            return Ok(Json(UserClaimsResponse {
                admin_modules: vec![],
                permissions: vec!["user:read".to_string()],
                package_tier: "FREE".to_string(),
                role: "user".to_string(),
                firebase_uid: None,
            }));
        }
        Err(err) => {
            error!("Database error getting user claims: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(UserClaimsResponse {
        admin_modules: user.admin_modules,
        permissions: user.permissions,
        package_tier: user.package_tier,
        role: user.role,
        firebase_uid: user.firebase_uid,
    }))
}

/**
 * Create or update user from OAuth provider
 * Called by Auth.js during sign-in process
 */
#[axum::debug_handler]
pub async fn upsert_user(
    State(pool): State<PgPool>,
    Json(request): Json<UpsertUserRequest>,
) -> Result<Json<UpsertUserResponse>, StatusCode> {
    info!("Upserting user: {}", request.email);

    let existing_user = get_user_by_email(&pool, &request.email).await;

    match existing_user {
        Ok(Some(user)) => {
            // User exists, update last login time
            if let Err(err) = update_user_last_login(&pool, &user.id).await {
                error!("Failed to update user last login: {}", err);
            }

            Ok(Json(UpsertUserResponse {
                user_id: user.id,
                email: user.email,
                created: false,
            }))
        }
        Ok(None) => {
            // Create new user
            match create_new_user(&pool, &request).await {
                Ok(user) => Ok(Json(UpsertUserResponse {
                    user_id: user.id,
                    email: user.email,
                    created: true,
                })),
                Err(err) => {
                    error!("Failed to create new user: {}", err);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(err) => {
            error!("Database error during user upsert: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/**
 * Simple database models for user operations
 */
#[derive(Debug, Clone)]
struct DatabaseUser {
    pub id: String,
    pub email: String,
    pub admin_modules: Vec<String>,
    pub permissions: Vec<String>,
    pub package_tier: String,
    pub role: String,
    pub firebase_uid: Option<String>,
}

/**
 * Get user by email from database
 */
async fn get_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<DatabaseUser>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT 
            id::text,
            firebase_uid,
            email,
            'FREE' as package_tier,
            'user' as role
        FROM users 
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        // Generate basic permissions for now
        let permissions = vec!["user:read".to_string()];

        // Get admin modules from user_admin_roles table
        let admin_modules = get_user_admin_modules(pool, &row.firebase_uid).await.unwrap_or_default();

        Ok(Some(DatabaseUser {
            id: row.id.unwrap_or_default(),
            email: row.email,
            admin_modules,
            permissions,
            package_tier: row.package_tier.unwrap_or("FREE".to_string()),
            role: row.role.unwrap_or("user".to_string()),
            firebase_uid: Some(row.firebase_uid),
        }))
    } else {
        Ok(None)
    }
}

/**
 * Get user by ID from database
 */
async fn get_user_by_id(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<DatabaseUser>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT 
            id::text,
            firebase_uid,
            email,
            'FREE' as package_tier,
            'user' as role
        FROM users 
        WHERE id::text = $1 OR firebase_uid = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        // Generate basic permissions for now
        let permissions = vec!["user:read".to_string()];

        // Get admin modules from user_admin_roles table
        let admin_modules = get_user_admin_modules(pool, &row.firebase_uid).await.unwrap_or_default();

        Ok(Some(DatabaseUser {
            id: row.id.unwrap_or_default(),
            email: row.email,
            admin_modules,
            permissions,
            package_tier: row.package_tier.unwrap_or("FREE".to_string()),
            role: row.role.unwrap_or("user".to_string()),
            firebase_uid: Some(row.firebase_uid),
        }))
    } else {
        Ok(None)
    }
}

/**
 * Create new user in database
 */
async fn create_new_user(
    pool: &PgPool,
    request: &UpsertUserRequest,
) -> Result<DatabaseUser, sqlx::Error> {
    let user_id = uuid::Uuid::new_v4();
    let firebase_uid = format!("oauth_{}", &request.provider_id);

    sqlx::query!(
        r#"
        INSERT INTO users (
            firebase_uid, 
            email, 
            created_at,
            updated_at
        ) 
        VALUES ($1, $2, NOW(), NOW())
        "#,
        firebase_uid,
        request.email
    )
    .execute(pool)
    .await?;

    Ok(DatabaseUser {
        id: user_id.to_string(),
        email: request.email.clone(),
        admin_modules: vec![],
        permissions: vec!["user:read".to_string()],
        package_tier: "FREE".to_string(),
        role: "user".to_string(),
        firebase_uid: Some(firebase_uid),
    })
}

/**
 * Update user last login time
 */
async fn update_user_last_login(
    pool: &PgPool,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE users SET updated_at = NOW() WHERE id::text = $1",
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}


/**
 * Get user's admin modules from user_admin_roles table
 */
async fn get_user_admin_modules(
    pool: &PgPool,
    firebase_uid: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT module_code 
        FROM user_admin_roles 
        WHERE firebase_uid = $1 
        AND is_active = true 
        AND (expires_at IS NULL OR expires_at > NOW())
        "#,
        firebase_uid
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| row.module_code).collect())
}