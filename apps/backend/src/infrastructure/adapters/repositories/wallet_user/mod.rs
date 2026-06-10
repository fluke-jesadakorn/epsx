// Wallet User Repository Adapter — directory module
// WalletUserRepositoryAdapter implements all three ports split across submodules

pub mod mutations;
pub mod queries;
pub mod analytics;

use crate::prelude::*;

// Define PostgreSQL LOWER() for type-safe case-insensitive queries
diesel::sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);

// Query result struct shared across submodules
#[derive(diesel::QueryableByName)]
pub(crate) struct WalletUserQueryResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub wallet_address: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub is_active: bool,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub wallet_metadata: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// PostgreSQL implementation of wallet user repository ports using Diesel
#[derive(Clone)]
pub struct WalletUserRepositoryAdapter {
    pub(crate) db_pool: &'static TlsPool,
}

impl WalletUserRepositoryAdapter {
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }
}
