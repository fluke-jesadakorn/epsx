// Wallet User Repository Adapter — directory module
// WalletUserRepositoryAdapter implements all three ports split across submodules
//
// BIG-BANG: migrated to sqlx (real).

pub mod analytics;
pub mod mutations;
pub mod queries;
pub mod sqlx_wallet_user_repository;

use epsx_database_pools::TlsPool;

// Query result struct shared across submodules (sqlx::FromRow)
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct WalletUserQueryResult {
    pub wallet_address: String,
    pub is_active: bool,
    pub wallet_metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// PostgreSQL implementation of wallet user repository ports using sqlx
#[derive(Clone)]
pub struct WalletUserRepositoryAdapter {
    pub(crate) db_pool: &'static TlsPool,
}

impl WalletUserRepositoryAdapter {
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }
}