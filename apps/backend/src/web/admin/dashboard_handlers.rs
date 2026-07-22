use axum::{
    extract::{RawQuery, State},
    Json,
};
use chrono::{DateTime, Utc};
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use epsx_contracts::errors::{AppError, ErrorKind};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::web::admin::responses::{AdminApiResponse, AdminMetadata};
use crate::web::auth::AppState;

const DASHBOARD_USER_STATUS_SQL: &str = r#"
SELECT
    statement_timestamp() AS observed_at,
    COUNT(*)::bigint AS total_users,
    COUNT(*) FILTER (WHERE is_active = TRUE)::bigint AS active_users
FROM wallet_users
"#;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct AdminDashboardUserStatusResponse {
    pub observed_at: DateTime<Utc>,
    pub total_users: i64,
    pub active_users: i64,
}

#[derive(Debug, QueryableByName)]
struct DashboardUserStatusRow {
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    observed_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total_users: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    active_users: i64,
}

impl TryFrom<DashboardUserStatusRow> for AdminDashboardUserStatusResponse {
    type Error = AppError;

    fn try_from(row: DashboardUserStatusRow) -> Result<Self, Self::Error> {
        if row.total_users < 0 || row.active_users < 0 || row.active_users > row.total_users {
            return Err(AppError::new(
                ErrorKind::InternalServerError,
                "Dashboard user status count invariants failed",
            ));
        }

        Ok(Self {
            observed_at: row.observed_at,
            total_users: row.total_users,
            active_users: row.active_users,
        })
    }
}

pub async fn admin_dashboard_user_status_handler(
    State(app_state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<AdminApiResponse<AdminDashboardUserStatusResponse>>, AppError> {
    reject_query(raw_query.as_deref())?;

    let mut connection = app_state.db_pool.get().await.map_err(|pool_error| {
        error!(error = ?pool_error, "Failed to acquire dashboard user status database connection");
        AppError::database_error("Dashboard user status is temporarily unavailable")
    })?;
    let row = diesel::sql_query(DASHBOARD_USER_STATUS_SQL)
        .get_result::<DashboardUserStatusRow>(&mut connection)
        .await
        .map_err(|query_error| {
            error!(error = ?query_error, "Failed to query dashboard user status");
            AppError::database_error("Dashboard user status is temporarily unavailable")
        })?;
    let response = AdminDashboardUserStatusResponse::try_from(row)?;

    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "Dashboard user status retrieved successfully",
        AdminMetadata::crud_operation("get_dashboard_user_status", None),
    )))
}

fn reject_query(raw_query: Option<&str>) -> Result<(), AppError> {
    if raw_query.is_some() {
        return Err(AppError::bad_request(
            "Dashboard user status does not accept query parameters",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observed_at() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-07-23T10:15:30Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn row(total_users: i64, active_users: i64) -> DashboardUserStatusRow {
        DashboardUserStatusRow {
            observed_at: observed_at(),
            total_users,
            active_users,
        }
    }

    #[test]
    fn dashboard_user_status_query_is_one_statement_against_wallet_users() {
        let query = DASHBOARD_USER_STATUS_SQL.trim();

        assert!(query.starts_with("SELECT"));
        assert!(query.contains("statement_timestamp() AS observed_at"));
        assert!(query.contains("COUNT(*)::bigint AS total_users"));
        assert!(query.contains("COUNT(*) FILTER (WHERE is_active = TRUE)::bigint AS active_users"));
        assert!(query.ends_with("FROM wallet_users"));
        assert!(!query.trim_end_matches(';').contains(';'));
        assert!(!query.contains('$'));
    }

    #[test]
    fn dashboard_user_status_rejects_every_query_string() {
        assert!(reject_query(None).is_ok());
        for raw_query in [Some(""), Some("page=1"), Some("status=active")] {
            let error = reject_query(raw_query).unwrap_err();
            assert_eq!(error.kind, ErrorKind::ValidationError);
        }
    }

    #[test]
    fn dashboard_user_status_row_mapping_preserves_i64_counts() {
        let total_users = i64::from(i32::MAX) + 50;
        let response = AdminDashboardUserStatusResponse::try_from(row(total_users, 42)).unwrap();

        assert_eq!(response.observed_at, observed_at());
        assert_eq!(response.total_users, total_users);
        assert_eq!(response.active_users, 42);
    }

    #[test]
    fn dashboard_user_status_row_mapping_fails_closed_on_invalid_counts() {
        for invalid_row in [row(-1, 0), row(1, -1), row(1, 2)] {
            let error = AdminDashboardUserStatusResponse::try_from(invalid_row).unwrap_err();
            assert_eq!(error.kind, ErrorKind::InternalServerError);
        }
    }

    #[test]
    fn dashboard_user_status_uses_typed_admin_envelope() {
        let response = AdminDashboardUserStatusResponse::try_from(row(10, 7)).unwrap();
        let envelope = AdminApiResponse::success_with_meta(
            response,
            "Dashboard user status retrieved successfully",
            AdminMetadata::crud_operation("get_dashboard_user_status", None),
        );
        let json = serde_json::to_value(envelope).unwrap();
        let data = json["data"].as_object().unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(data.len(), 3);
        assert_eq!(data["total_users"], 10);
        assert_eq!(data["active_users"], 7);
        assert_eq!(data["observed_at"], "2026-07-23T10:15:30Z");
        assert_eq!(json["admin_meta"]["operation"], "get_dashboard_user_status");
    }
}
