use std::collections::{HashMap, HashSet};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::schemas::primary::user_watchlist;
use crate::web::{auth::AppState, middleware::OpenIDUserContext, responses::UnifiedApiResponse};

const MAX_GROUPS: usize = 200;
const MAX_WATCHLIST_SYMBOLS: usize = 1_000;

#[derive(Debug, Serialize, ToSchema)]
pub struct WatchlistResponse {
    pub symbols: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AddWatchlistRequest {
    pub symbol: String,
    /// Missing or empty keeps the symbol in the virtual Ungrouped section.
    /// Supplying IDs adds memberships without removing existing memberships.
    pub group_ids: Option<Vec<Uuid>>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct WatchlistGroupResponse {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub symbols: Vec<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct WatchlistLayoutResponse {
    pub groups: Vec<WatchlistGroupResponse>,
    pub ungrouped: Vec<String>,
    /// Count of distinct user_watchlist rows, never the membership count.
    pub watched: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WatchlistGroupNameRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WatchlistGroupLayoutInput {
    pub id: Uuid,
    pub symbols: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WatchlistLayoutRequest {
    pub groups: Vec<WatchlistGroupLayoutInput>,
    pub ungrouped: Vec<String>,
}

type ApiError = (StatusCode, Json<UnifiedApiResponse<()>>);
type ApiResult<T> = Result<Json<UnifiedApiResponse<T>>, ApiError>;

fn api_error(status: StatusCode, message: &str, reason: &str) -> ApiError {
    (
        status,
        Json(UnifiedApiResponse::error(status.as_u16(), message, reason)),
    )
}

#[derive(Debug)]
enum MutationError {
    Database(diesel::result::Error),
    Invalid(&'static str),
    NotFound,
}

impl From<diesel::result::Error> for MutationError {
    fn from(value: diesel::result::Error) -> Self {
        Self::Database(value)
    }
}

fn mutation_api_error(error_value: MutationError) -> ApiError {
    match error_value {
        MutationError::Invalid(reason) => {
            api_error(StatusCode::BAD_REQUEST, "Invalid watchlist layout", reason)
        }
        MutationError::NotFound => api_error(
            StatusCode::NOT_FOUND,
            "Group not found",
            "The group does not belong to this user",
        ),
        MutationError::Database(error_value) => {
            if matches!(
                error_value,
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _
                )
            ) {
                return api_error(
                    StatusCode::CONFLICT,
                    "Group name already exists",
                    "Group names must be unique, ignoring letter case",
                );
            }
            error!("Watchlist organizer database error: {error_value}");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
                "The watchlist change could not be saved",
            )
        }
    }
}

#[derive(QueryableByName)]
struct SymbolRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    symbol: String,
}

#[derive(QueryableByName)]
struct GroupRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

#[derive(QueryableByName)]
struct MembershipRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    group_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    symbol: String,
}

async fn fetch_watchlist(
    pool: &crate::prelude::TlsPool,
    wallet: &str,
) -> Result<Vec<String>, String> {
    let mut conn = pool
        .acquire().await
        .await
        .map_err(|error_value| error_value.to_string())?;
    user_watchlist::table
        .filter(user_watchlist::wallet_address.eq(wallet))
        .order((user_watchlist::added_at.asc(), user_watchlist::id.asc()))
        .select(user_watchlist::symbol)
        .load::<String>(&mut conn)
        .await
        .map_err(|error_value| error_value.to_string())
}

async fn fetch_layout_conn(
    conn: &mut AsyncPgConnection,
    wallet: &str,
) -> Result<WatchlistLayoutResponse, diesel::result::Error> {
    let group_rows = diesel::sql_query(
        "SELECT id, name
         FROM user_watchlist_groups
         WHERE wallet_address = $1
         ORDER BY position ASC, id ASC",
    )
    .bind::<diesel::sql_types::Text, _>(wallet)
    .load::<GroupRow>(conn)
    .await?;
    let membership_rows = diesel::sql_query(
        "SELECT membership.group_id, membership.symbol
         FROM user_watchlist_group_memberships membership
         JOIN user_watchlist_groups watchlist_group ON watchlist_group.id = membership.group_id
         WHERE watchlist_group.wallet_address = $1
         ORDER BY watchlist_group.position ASC, watchlist_group.id ASC,
                  membership.position ASC, membership.id ASC",
    )
    .bind::<diesel::sql_types::Text, _>(wallet)
    .load::<MembershipRow>(conn)
    .await?;
    let watched_rows = diesel::sql_query(
        "SELECT symbol
         FROM user_watchlist
         WHERE wallet_address = $1
         ORDER BY ungrouped_position ASC, added_at ASC, id ASC",
    )
    .bind::<diesel::sql_types::Text, _>(wallet)
    .load::<SymbolRow>(conn)
    .await?;

    let mut memberships: HashMap<Uuid, Vec<String>> = HashMap::new();
    let mut grouped_symbols = HashSet::new();
    for row in membership_rows {
        grouped_symbols.insert(row.symbol.clone());
        memberships
            .entry(row.group_id)
            .or_default()
            .push(row.symbol);
    }
    let groups = group_rows
        .into_iter()
        .enumerate()
        .map(|(position, row)| WatchlistGroupResponse {
            id: row.id,
            name: row.name,
            position: i32::try_from(position).unwrap_or(i32::MAX),
            symbols: memberships.remove(&row.id).unwrap_or_default(),
        })
        .collect();
    let watched = watched_rows.len();
    let ungrouped = watched_rows
        .into_iter()
        .filter_map(|row| (!grouped_symbols.contains(&row.symbol)).then_some(row.symbol))
        .collect();
    Ok(WatchlistLayoutResponse {
        groups,
        ungrouped,
        watched,
    })
}

fn normalize_watchlist_symbol(value: &str) -> Option<String> {
    let symbol = value.trim().to_ascii_uppercase();
    let mut characters = symbol.chars();
    let first = characters.next()?;
    if symbol.len() > 20
        || !first.is_ascii_alphanumeric()
        || !characters
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
    {
        return None;
    }
    Some(symbol)
}

fn normalize_group_name(value: &str) -> Option<String> {
    let name = value.trim();
    if !(1..=50).contains(&name.chars().count()) || name.chars().any(char::is_control) {
        return None;
    }
    Some(name.to_string())
}

fn normalize_symbol_list(values: &[String]) -> Result<Vec<String>, MutationError> {
    if values.len() > MAX_WATCHLIST_SYMBOLS {
        return Err(MutationError::Invalid("A group contains too many symbols"));
    }
    let mut seen = HashSet::new();
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        let symbol = normalize_watchlist_symbol(value).ok_or(MutationError::Invalid(
            "The layout contains an invalid symbol",
        ))?;
        if !seen.insert(symbol.clone()) {
            return Err(MutationError::Invalid(
                "A symbol appears more than once in the same group",
            ));
        }
        normalized.push(symbol);
    }
    Ok(normalized)
}

pub async fn get_watchlist(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
) -> Result<Json<UnifiedApiResponse<WatchlistResponse>>, Json<UnifiedApiResponse<()>>> {
    match fetch_watchlist(&app_state.db_pool, &ctx.wallet_address).await {
        Ok(symbols) => Ok(Json(UnifiedApiResponse::success(WatchlistResponse {
            symbols,
        }))),
        Err(error_value) => {
            error!("Failed to fetch watchlist: {error_value}");
            Err(Json(UnifiedApiResponse::error(
                500,
                "Database error",
                "Failed to fetch watchlist",
            )))
        }
    }
}

pub async fn get_watchlist_layout(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
) -> ApiResult<WatchlistLayoutResponse> {
    let mut conn = app_state.db_pool.acquire().await.map_err(|error_value| {
        error!("Watchlist layout connection error: {error_value}");
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error",
            "Failed to load watchlist layout",
        )
    })?;
    let layout = fetch_layout_conn(&mut conn, &ctx.wallet_address)
        .await
        .map_err(MutationError::Database)
        .map_err(mutation_api_error)?;
    Ok(Json(UnifiedApiResponse::success(layout)))
}

pub async fn add_to_watchlist(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Json(body): Json<AddWatchlistRequest>,
) -> Result<Json<UnifiedApiResponse<WatchlistResponse>>, Json<UnifiedApiResponse<()>>> {
    let Some(symbol) = normalize_watchlist_symbol(&body.symbol) else {
        return Err(Json(UnifiedApiResponse::error(
            400,
            "Invalid symbol",
            "Symbol must be 1-20 letters, numbers, dots, or hyphens",
        )));
    };
    let group_ids = body.group_ids.unwrap_or_default();
    if group_ids.len() > MAX_GROUPS
        || group_ids.iter().collect::<HashSet<_>>().len() != group_ids.len()
    {
        return Err(Json(UnifiedApiResponse::error(
            400,
            "Invalid groups",
            "Group IDs must be unique and owner-scoped",
        )));
    }

    let mut conn = app_state.db_pool.acquire().await.map_err(|error_value| {
        error!("DB connection error: {error_value}");
        Json(UnifiedApiResponse::error(
            500,
            "Database error",
            "Failed to connect",
        ))
    })?;
    let wallet = ctx.wallet_address.clone();
    let symbol_for_transaction = symbol.clone();
    let result = conn
        .transaction::<_, MutationError, _>(|conn| {
            Box::pin(async move {
                diesel::sql_query("SELECT pg_advisory_xact_lock(hashtext($1))")
                    .bind::<diesel::sql_types::Text, _>(&wallet)
                    .execute(conn)
                    .await?;
                let owner_groups = diesel::sql_query(
                    "SELECT id, name FROM user_watchlist_groups WHERE wallet_address = $1 FOR UPDATE",
                )
                .bind::<diesel::sql_types::Text, _>(&wallet)
                .load::<GroupRow>(conn)
                .await?;
                let owner_group_ids: HashSet<_> =
                    owner_groups.into_iter().map(|row| row.id).collect();
                if group_ids.iter().any(|id| !owner_group_ids.contains(id)) {
                    return Err(MutationError::Invalid(
                        "A group does not belong to this user",
                    ));
                }
                diesel::sql_query(
                    "INSERT INTO user_watchlist (wallet_address, symbol, ungrouped_position)
                     SELECT $1, $2, COALESCE(MAX(ungrouped_position), -1) + 1
                     FROM user_watchlist WHERE wallet_address = $1
                     ON CONFLICT (wallet_address, symbol) DO NOTHING",
                )
                .bind::<diesel::sql_types::Text, _>(&wallet)
                .bind::<diesel::sql_types::Text, _>(&symbol_for_transaction)
                .execute(conn)
                .await?;
                for group_id in group_ids {
                    diesel::sql_query(
                        "INSERT INTO user_watchlist_group_memberships
                            (group_id, wallet_address, symbol, position)
                         SELECT watchlist_group.id, $2, $3, COALESCE(MAX(membership.position), -1) + 1
                         FROM user_watchlist_groups watchlist_group
                         LEFT JOIN user_watchlist_group_memberships membership
                           ON membership.group_id = watchlist_group.id
                         WHERE watchlist_group.id = $1 AND watchlist_group.wallet_address = $2
                         GROUP BY watchlist_group.id
                         ON CONFLICT (group_id, symbol) DO NOTHING",
                    )
                    .bind::<diesel::sql_types::Uuid, _>(group_id)
                    .bind::<diesel::sql_types::Text, _>(&wallet)
                    .bind::<diesel::sql_types::Text, _>(&symbol_for_transaction)
                    .execute(conn)
                    .await?;
                }
                Ok(())
            })
        })
        .await;
    if let Err(error_value) = result {
        let (_, response) = mutation_api_error(error_value);
        return Err(response);
    }

    info!("Added {symbol} to watchlist for {}", ctx.wallet_address);
    match fetch_watchlist(&app_state.db_pool, &ctx.wallet_address).await {
        Ok(symbols) => Ok(Json(UnifiedApiResponse::success(WatchlistResponse {
            symbols,
        }))),
        Err(error_value) => {
            error!("Failed to fetch watchlist after add: {error_value}");
            Err(Json(UnifiedApiResponse::error(
                500,
                "Database error",
                "Watchlist update could not be verified",
            )))
        }
    }
}

pub async fn remove_from_watchlist(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(symbol): Path<String>,
) -> Result<Json<UnifiedApiResponse<WatchlistResponse>>, Json<UnifiedApiResponse<()>>> {
    let Some(symbol) = normalize_watchlist_symbol(&symbol) else {
        return Err(Json(UnifiedApiResponse::error(
            400,
            "Invalid symbol",
            "Symbol must be 1-20 letters, numbers, dots, or hyphens",
        )));
    };
    let mut conn = app_state.db_pool.acquire().await.map_err(|error_value| {
        error!("DB connection error: {error_value}");
        Json(UnifiedApiResponse::error(
            500,
            "Database error",
            "Failed to connect",
        ))
    })?;
    diesel::delete(
        user_watchlist::table
            .filter(user_watchlist::wallet_address.eq(&ctx.wallet_address))
            .filter(user_watchlist::symbol.eq(&symbol)),
    )
    .execute(&mut *conn)
    .await
    .map_err(|error_value| {
        error!("Failed to remove from watchlist: {error_value}");
        Json(UnifiedApiResponse::error(
            500,
            "Database error",
            "The symbol could not be removed",
        ))
    })?;
    info!("Removed {symbol} from watchlist for {}", ctx.wallet_address);
    match fetch_watchlist(&app_state.db_pool, &ctx.wallet_address).await {
        Ok(symbols) => Ok(Json(UnifiedApiResponse::success(WatchlistResponse {
            symbols,
        }))),
        Err(error_value) => {
            error!("Failed to fetch watchlist after remove: {error_value}");
            Err(Json(UnifiedApiResponse::error(
                500,
                "Database error",
                "Watchlist update could not be verified",
            )))
        }
    }
}

pub async fn create_watchlist_group(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Json(body): Json<WatchlistGroupNameRequest>,
) -> ApiResult<WatchlistLayoutResponse> {
    let name = normalize_group_name(&body.name).ok_or_else(|| {
        api_error(
            StatusCode::BAD_REQUEST,
            "Invalid group name",
            "Group names must contain 1-50 characters",
        )
    })?;
    let mut conn = app_state.db_pool.acquire().await.map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error",
            "Failed to save group",
        )
    })?;
    let wallet = ctx.wallet_address.clone();
    let layout = conn
        .transaction::<_, MutationError, _>(|conn| {
            Box::pin(async move {
                diesel::sql_query("SELECT pg_advisory_xact_lock(hashtext($1))")
                    .bind::<diesel::sql_types::Text, _>(&wallet)
                    .execute(conn)
                    .await?;
                diesel::sql_query(
                    "INSERT INTO user_watchlist_groups (wallet_address, name, position)
                     SELECT $1, $2, COALESCE(MAX(position), -1) + 1
                     FROM user_watchlist_groups WHERE wallet_address = $1",
                )
                .bind::<diesel::sql_types::Text, _>(&wallet)
                .bind::<diesel::sql_types::Text, _>(&name)
                .execute(conn)
                .await?;
                fetch_layout_conn(conn, &wallet)
                    .await
                    .map_err(MutationError::Database)
            })
        })
        .await
        .map_err(mutation_api_error)?;
    Ok(Json(UnifiedApiResponse::success(layout)))
}

pub async fn update_watchlist_group(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(group_id): Path<Uuid>,
    Json(body): Json<WatchlistGroupNameRequest>,
) -> ApiResult<WatchlistLayoutResponse> {
    let name = normalize_group_name(&body.name).ok_or_else(|| {
        api_error(
            StatusCode::BAD_REQUEST,
            "Invalid group name",
            "Group names must contain 1-50 characters",
        )
    })?;
    let mut conn = app_state.db_pool.acquire().await.map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error",
            "Failed to rename group",
        )
    })?;
    let wallet = ctx.wallet_address.clone();
    let layout = conn
        .transaction::<_, MutationError, _>(|conn| {
            Box::pin(async move {
                let updated = diesel::sql_query(
                    "UPDATE user_watchlist_groups SET name = $3, updated_at = NOW()
                     WHERE id = $1 AND wallet_address = $2",
                )
                .bind::<diesel::sql_types::Uuid, _>(group_id)
                .bind::<diesel::sql_types::Text, _>(&wallet)
                .bind::<diesel::sql_types::Text, _>(&name)
                .execute(conn)
                .await?;
                if updated == 0 {
                    return Err(MutationError::NotFound);
                }
                fetch_layout_conn(conn, &wallet)
                    .await
                    .map_err(MutationError::Database)
            })
        })
        .await
        .map_err(mutation_api_error)?;
    Ok(Json(UnifiedApiResponse::success(layout)))
}

pub async fn delete_watchlist_group(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(group_id): Path<Uuid>,
) -> ApiResult<WatchlistLayoutResponse> {
    let mut conn = app_state.db_pool.acquire().await.map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error",
            "Failed to delete group",
        )
    })?;
    let wallet = ctx.wallet_address.clone();
    let layout = conn
        .transaction::<_, MutationError, _>(|conn| {
            Box::pin(async move {
                diesel::sql_query("SELECT pg_advisory_xact_lock(hashtext($1))")
                    .bind::<diesel::sql_types::Text, _>(&wallet)
                    .execute(conn)
                    .await?;
                let groups = diesel::sql_query(
                    "SELECT id, name FROM user_watchlist_groups
                     WHERE id = $1 AND wallet_address = $2 FOR UPDATE",
                )
                .bind::<diesel::sql_types::Uuid, _>(group_id)
                .bind::<diesel::sql_types::Text, _>(&wallet)
                .load::<GroupRow>(conn)
                .await?;
                if groups.is_empty() {
                    return Err(MutationError::NotFound);
                }
                diesel::sql_query(
                    "WITH orphaned AS (
                         SELECT membership.symbol,
                                ROW_NUMBER() OVER (ORDER BY membership.position, membership.id) AS offset
                         FROM user_watchlist_group_memberships membership
                         WHERE membership.group_id = $1 AND membership.wallet_address = $2
                           AND NOT EXISTS (
                               SELECT 1 FROM user_watchlist_group_memberships other
                               WHERE other.wallet_address = $2
                                 AND other.symbol = membership.symbol
                                 AND other.group_id <> $1
                           )
                     ), base AS (
                         SELECT COALESCE(MAX(watchlist.ungrouped_position), -1) AS position
                         FROM user_watchlist watchlist
                         WHERE watchlist.wallet_address = $2
                           AND NOT EXISTS (
                               SELECT 1 FROM user_watchlist_group_memberships existing
                               WHERE existing.wallet_address = $2
                                 AND existing.symbol = watchlist.symbol
                           )
                     )
                     UPDATE user_watchlist watchlist
                     SET ungrouped_position = base.position + orphaned.offset
                     FROM orphaned, base
                     WHERE watchlist.wallet_address = $2 AND watchlist.symbol = orphaned.symbol",
                )
                .bind::<diesel::sql_types::Uuid, _>(group_id)
                .bind::<diesel::sql_types::Text, _>(&wallet)
                .execute(conn)
                .await?;
                diesel::sql_query(
                    "DELETE FROM user_watchlist_groups WHERE id = $1 AND wallet_address = $2",
                )
                .bind::<diesel::sql_types::Uuid, _>(group_id)
                .bind::<diesel::sql_types::Text, _>(&wallet)
                .execute(conn)
                .await?;
                diesel::sql_query(
                    "WITH ordered AS (
                         SELECT id, ROW_NUMBER() OVER (ORDER BY position, id) - 1 AS position
                         FROM user_watchlist_groups WHERE wallet_address = $1
                     )
                     UPDATE user_watchlist_groups watchlist_group
                     SET position = ordered.position, updated_at = NOW()
                     FROM ordered WHERE watchlist_group.id = ordered.id",
                )
                .bind::<diesel::sql_types::Text, _>(&wallet)
                .execute(conn)
                .await?;
                fetch_layout_conn(conn, &wallet)
                    .await
                    .map_err(MutationError::Database)
            })
        })
        .await
        .map_err(mutation_api_error)?;
    Ok(Json(UnifiedApiResponse::success(layout)))
}

pub async fn update_watchlist_layout(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Json(body): Json<WatchlistLayoutRequest>,
) -> ApiResult<WatchlistLayoutResponse> {
    if body.groups.len() > MAX_GROUPS {
        return Err(mutation_api_error(MutationError::Invalid(
            "The layout contains too many groups",
        )));
    }
    let mut seen_group_ids = HashSet::new();
    let mut prepared_groups = Vec::with_capacity(body.groups.len());
    let mut grouped_union = HashSet::new();
    for group in body.groups {
        if !seen_group_ids.insert(group.id) {
            return Err(mutation_api_error(MutationError::Invalid(
                "A group appears more than once",
            )));
        }
        let symbols = normalize_symbol_list(&group.symbols).map_err(mutation_api_error)?;
        grouped_union.extend(symbols.iter().cloned());
        prepared_groups.push((group.id, symbols));
    }
    let ungrouped = normalize_symbol_list(&body.ungrouped).map_err(mutation_api_error)?;
    if ungrouped
        .iter()
        .any(|symbol| grouped_union.contains(symbol))
    {
        return Err(mutation_api_error(MutationError::Invalid(
            "Grouped symbols cannot also appear in Ungrouped",
        )));
    }

    let mut conn = app_state.db_pool.acquire().await.map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error",
            "Failed to save watchlist layout",
        )
    })?;
    let wallet = ctx.wallet_address.clone();
    let layout = conn
        .transaction::<_, MutationError, _>(|conn| {
            Box::pin(async move {
                diesel::sql_query("SELECT pg_advisory_xact_lock(hashtext($1))")
                    .bind::<diesel::sql_types::Text, _>(&wallet)
                    .execute(conn)
                    .await?;
                let owner_groups = diesel::sql_query(
                    "SELECT id, name FROM user_watchlist_groups
                     WHERE wallet_address = $1 ORDER BY position, id FOR UPDATE",
                )
                .bind::<diesel::sql_types::Text, _>(&wallet)
                .load::<GroupRow>(conn)
                .await?;
                let owner_group_ids: HashSet<_> = owner_groups.iter().map(|row| row.id).collect();
                let requested_group_ids: HashSet<_> =
                    prepared_groups.iter().map(|(id, _)| *id).collect();
                if owner_group_ids != requested_group_ids {
                    return Err(MutationError::Invalid(
                        "The layout must contain every owner-scoped group exactly once",
                    ));
                }
                let watched_rows = diesel::sql_query(
                    "SELECT symbol FROM user_watchlist WHERE wallet_address = $1 FOR UPDATE",
                )
                .bind::<diesel::sql_types::Text, _>(&wallet)
                .load::<SymbolRow>(conn)
                .await?;
                let watched_symbols: HashSet<_> =
                    watched_rows.into_iter().map(|row| row.symbol).collect();
                let requested_symbols: HashSet<_> = grouped_union
                    .iter()
                    .cloned()
                    .chain(ungrouped.iter().cloned())
                    .collect();
                if watched_symbols != requested_symbols {
                    return Err(MutationError::Invalid(
                        "The layout cannot add or indirectly unwatch symbols",
                    ));
                }

                diesel::sql_query(
                    "DELETE FROM user_watchlist_group_memberships membership
                     USING user_watchlist_groups watchlist_group
                     WHERE membership.group_id = watchlist_group.id
                       AND watchlist_group.wallet_address = $1",
                )
                .bind::<diesel::sql_types::Text, _>(&wallet)
                .execute(conn)
                .await?;
                for (group_position, (group_id, symbols)) in prepared_groups.iter().enumerate() {
                    diesel::sql_query(
                        "UPDATE user_watchlist_groups SET position = $3, updated_at = NOW()
                         WHERE id = $1 AND wallet_address = $2",
                    )
                    .bind::<diesel::sql_types::Uuid, _>(*group_id)
                    .bind::<diesel::sql_types::Text, _>(&wallet)
                    .bind::<diesel::sql_types::Integer, _>(
                        i32::try_from(group_position).unwrap_or(i32::MAX),
                    )
                    .execute(conn)
                    .await?;
                    for (symbol_position, symbol) in symbols.iter().enumerate() {
                        diesel::sql_query(
                            "INSERT INTO user_watchlist_group_memberships
                                (group_id, wallet_address, symbol, position)
                             VALUES ($1, $2, $3, $4)",
                        )
                        .bind::<diesel::sql_types::Uuid, _>(*group_id)
                        .bind::<diesel::sql_types::Text, _>(&wallet)
                        .bind::<diesel::sql_types::Text, _>(symbol)
                        .bind::<diesel::sql_types::Integer, _>(
                            i32::try_from(symbol_position).unwrap_or(i32::MAX),
                        )
                        .execute(conn)
                        .await?;
                    }
                }
                for (position, symbol) in ungrouped.iter().enumerate() {
                    diesel::sql_query(
                        "UPDATE user_watchlist SET ungrouped_position = $3
                         WHERE wallet_address = $1 AND symbol = $2",
                    )
                    .bind::<diesel::sql_types::Text, _>(&wallet)
                    .bind::<diesel::sql_types::Text, _>(symbol)
                    .bind::<diesel::sql_types::Integer, _>(
                        i32::try_from(position).unwrap_or(i32::MAX),
                    )
                    .execute(conn)
                    .await?;
                }
                fetch_layout_conn(conn, &wallet)
                    .await
                    .map_err(MutationError::Database)
            })
        })
        .await
        .map_err(mutation_api_error)?;
    Ok(Json(UnifiedApiResponse::success(layout)))
}

#[cfg(test)]
mod tests {
    use super::{normalize_group_name, normalize_symbol_list, normalize_watchlist_symbol};

    #[test]
    fn watchlist_symbols_are_canonical_and_path_safe() {
        for (raw, expected) in [
            (" aapl ", Some("AAPL")),
            ("BRK.B", Some("BRK.B")),
            ("btc-usd", Some("BTC-USD")),
            ("../AAPL", None),
            ("AAPL/US", None),
            ("", None),
            ("ABCDEFGHIJKLMNOPQRSTU", None),
        ] {
            assert_eq!(normalize_watchlist_symbol(raw).as_deref(), expected);
        }
    }

    #[test]
    fn group_names_are_trimmed_bounded_and_case_collision_ready() {
        assert_eq!(
            normalize_group_name("  Long term  ").as_deref(),
            Some("Long term")
        );
        assert_eq!(normalize_group_name(" "), None);
        assert_eq!(normalize_group_name(&"x".repeat(51)), None);
        assert_eq!(normalize_group_name("bad\nname"), None);
        assert_eq!(
            normalize_group_name("Growth").unwrap().to_lowercase(),
            normalize_group_name(" growth ").unwrap().to_lowercase()
        );
    }

    #[test]
    fn layout_rejects_duplicates_after_symbol_normalization() {
        assert!(normalize_symbol_list(&["AAPL".into(), " aapl ".into()]).is_err());
        assert_eq!(
            normalize_symbol_list(&["brk.b".into(), "BTC-USD".into()]).unwrap(),
            ["BRK.B", "BTC-USD"]
        );
    }

    #[test]
    fn migration_backfills_every_legacy_row_in_added_order_without_deleting_data() {
        let migration =
            include_str!("../../../migrations/core/20260821120000_add_watchlist_groups/up.sql");
        for required in [
            "ADD COLUMN IF NOT EXISTS ungrouped_position",
            "PARTITION BY wallet_address",
            "ORDER BY added_at ASC, id ASC",
            "watchlist.ungrouped_position IS NULL",
            "ALTER COLUMN ungrouped_position SET NOT NULL",
        ] {
            assert!(
                migration.contains(required),
                "missing migration guard: {required}"
            );
        }
        assert!(!migration
            .to_ascii_uppercase()
            .contains("DELETE FROM USER_WATCHLIST"));
        assert!(!migration
            .to_ascii_uppercase()
            .contains("DROP TABLE USER_WATCHLIST"));
    }
}
