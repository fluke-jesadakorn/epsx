// ============================================================================
// FCM TOKEN MODELS FOR FIREBASE CLOUD MESSAGING
// ============================================================================
// Models for managing FCM device tokens and push notification delivery

use chrono::{DateTime, Utc};
use uuid::Uuid;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::infra::db::diesel::schema::fcm_tokens;
use crate::infra::db::diesel::types::DevicePlatform;

// ============================================================================
// FCM TOKEN MODELS
// ============================================================================

#[derive(Queryable, Selectable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = fcm_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselFcmToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub platform: DevicePlatform,
    pub device_info: Option<JsonValue>,
    pub user_agent: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = fcm_tokens)]
pub struct NewDieselFcmToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub platform: DevicePlatform,
    pub device_info: Option<JsonValue>,
    pub user_agent: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

// ============================================================================
// FCM TOKEN HELPER FUNCTIONS
// ============================================================================

impl NewDieselFcmToken {
    pub fn new(
        user_id: Uuid,
        token: String,
        platform: DevicePlatform,
        device_info: Option<JsonValue>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            token,
            platform,
            device_info,
            user_agent,
            is_active: Some(true),
            created_at: Some(now),
            updated_at: Some(now),
            last_used_at: Some(now),
        }
    }

    pub fn web_token(user_id: Uuid, token: String, user_agent: Option<String>) -> Self {
        Self::new(user_id, token, DevicePlatform::Web, None, user_agent)
    }

    pub fn android_token(user_id: Uuid, token: String, device_info: Option<JsonValue>) -> Self {
        Self::new(user_id, token, DevicePlatform::Android, device_info, None)
    }

    pub fn ios_token(user_id: Uuid, token: String, device_info: Option<JsonValue>) -> Self {
        Self::new(user_id, token, DevicePlatform::Ios, device_info, None)
    }
}

impl DieselFcmToken {
    /// Check if token is still valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        self.is_active.unwrap_or(false) && 
        self.last_used_at
            .map(|last_used| {
                // Token is valid if used within last 30 days
                let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
                last_used > thirty_days_ago
            })
            .unwrap_or(false)
    }

    /// Update last used timestamp
    pub fn touch(&mut self) {
        self.last_used_at = Some(Utc::now());
        self.updated_at = Some(Utc::now());
    }

    /// Deactivate token
    pub fn deactivate(&mut self) {
        self.is_active = Some(false);
        self.updated_at = Some(Utc::now());
    }
}

// ============================================================================
// DATABASE OPERATIONS
// ============================================================================

/// Register a new FCM token for a user
pub async fn create_fcm_token(
    conn: &mut diesel_async::AsyncPgConnection,
    token_data: NewDieselFcmToken,
) -> Result<DieselFcmToken, diesel::result::Error> {
    diesel::insert_into(fcm_tokens::table)
        .values(&token_data)
        .on_conflict(fcm_tokens::token)
        .do_update()
        .set((
            fcm_tokens::user_id.eq(token_data.user_id),
            fcm_tokens::is_active.eq(true),
            fcm_tokens::updated_at.eq(Utc::now()),
            fcm_tokens::last_used_at.eq(Utc::now()),
        ))
        .returning(DieselFcmToken::as_returning())
        .get_result(conn)
        .await
}

/// Get all active FCM tokens for a user
pub async fn get_user_active_tokens(
    conn: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> Result<Vec<DieselFcmToken>, diesel::result::Error> {
    fcm_tokens::table
        .filter(fcm_tokens::user_id.eq(user_id))
        .filter(fcm_tokens::is_active.eq(true))
        .order(fcm_tokens::last_used_at.desc())
        .select(DieselFcmToken::as_select())
        .load(conn)
        .await
}

/// Get active tokens by platform
pub async fn get_user_tokens_by_platform(
    conn: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
    platform: DevicePlatform,
) -> Result<Vec<DieselFcmToken>, diesel::result::Error> {
    fcm_tokens::table
        .filter(fcm_tokens::user_id.eq(user_id))
        .filter(fcm_tokens::platform.eq(platform))
        .filter(fcm_tokens::is_active.eq(true))
        .order(fcm_tokens::last_used_at.desc())
        .select(DieselFcmToken::as_select())
        .load(conn)
        .await
}

/// Deactivate a specific FCM token
pub async fn deactivate_fcm_token(
    conn: &mut diesel_async::AsyncPgConnection,
    token: &str,
) -> Result<usize, diesel::result::Error> {
    diesel::update(fcm_tokens::table.filter(fcm_tokens::token.eq(token)))
        .set((
            fcm_tokens::is_active.eq(false),
            fcm_tokens::updated_at.eq(Utc::now()),
        ))
        .execute(conn)
        .await
}

/// Deactivate all FCM tokens for a user
pub async fn deactivate_user_tokens(
    conn: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> Result<usize, diesel::result::Error> {
    diesel::update(fcm_tokens::table.filter(fcm_tokens::user_id.eq(user_id)))
        .set((
            fcm_tokens::is_active.eq(false),
            fcm_tokens::updated_at.eq(Utc::now()),
        ))
        .execute(conn)
        .await
}

/// Update token's last used timestamp
pub async fn update_token_last_used(
    conn: &mut diesel_async::AsyncPgConnection,
    token: &str,
) -> Result<usize, diesel::result::Error> {
    let now = Utc::now();
    diesel::update(fcm_tokens::table.filter(fcm_tokens::token.eq(token)))
        .set((
            fcm_tokens::last_used_at.eq(now),
            fcm_tokens::updated_at.eq(now),
        ))
        .execute(conn)
        .await
}

/// Get all active FCM tokens (for broadcast)
pub async fn get_all_active_tokens(
    conn: &mut diesel_async::AsyncPgConnection,
    limit: Option<i64>,
) -> Result<Vec<DieselFcmToken>, diesel::result::Error> {
    let mut query = fcm_tokens::table
        .filter(fcm_tokens::is_active.eq(true))
        .order(fcm_tokens::last_used_at.desc())
        .into_boxed();

    if let Some(limit_val) = limit {
        query = query.limit(limit_val);
    }

    query.select(DieselFcmToken::as_select()).load(conn)
        .await
}

/// Clean up old inactive tokens
pub async fn cleanup_old_tokens(
    conn: &mut diesel_async::AsyncPgConnection,
    days_old: i64,
) -> Result<usize, diesel::result::Error> {
    let cutoff_date = Utc::now() - chrono::Duration::days(days_old);
    
    diesel::delete(
        fcm_tokens::table
            .filter(fcm_tokens::is_active.eq(false))
            .filter(fcm_tokens::updated_at.lt(cutoff_date))
    )
    .execute(conn)
    .await
}

/// Get FCM token statistics
pub async fn get_fcm_stats(
    conn: &mut diesel_async::AsyncPgConnection,
) -> Result<FcmStats, diesel::result::Error> {
    use diesel::dsl::count;

    let total_tokens = fcm_tokens::table
        .select(count(fcm_tokens::id))
        .first::<i64>(conn)
        .await?;

    let active_tokens = fcm_tokens::table
        .filter(fcm_tokens::is_active.eq(true))
        .select(count(fcm_tokens::id))
        .first::<i64>(conn)
        .await?;

    let web_tokens = fcm_tokens::table
        .filter(fcm_tokens::is_active.eq(true))
        .filter(fcm_tokens::platform.eq(DevicePlatform::Web))
        .select(count(fcm_tokens::id))
        .first::<i64>(conn)
        .await?;

    let android_tokens = fcm_tokens::table
        .filter(fcm_tokens::is_active.eq(true))
        .filter(fcm_tokens::platform.eq(DevicePlatform::Android))
        .select(count(fcm_tokens::id))
        .first::<i64>(conn)
        .await?;

    let ios_tokens = fcm_tokens::table
        .filter(fcm_tokens::is_active.eq(true))
        .filter(fcm_tokens::platform.eq(DevicePlatform::Ios))
        .select(count(fcm_tokens::id))
        .first::<i64>(conn)
        .await?;

    Ok(FcmStats {
        total_tokens,
        active_tokens,
        web_tokens,
        android_tokens,
        ios_tokens,
    })
}

// ============================================================================
// FCM STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmStats {
    pub total_tokens: i64,
    pub active_tokens: i64,
    pub web_tokens: i64,
    pub android_tokens: i64,
    pub ios_tokens: i64,
}

impl FcmStats {
    pub fn inactive_tokens(&self) -> i64 {
        self.total_tokens - self.active_tokens
    }

    pub fn platform_distribution(&self) -> Vec<(&str, i64)> {
        vec![
            ("web", self.web_tokens),
            ("android", self.android_tokens),
            ("ios", self.ios_tokens),
        ]
    }
}