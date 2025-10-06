use crate::prelude::*;

use crate::application::shared::{Query, QueryHandler, ApplicationResult, ApplicationError};
use crate::application::wallet_management::queries::models::{
    ListWalletSessionsQuery, ListWalletSessionsResponse, WalletSessionInfo
};
use crate::domain::wallet_management::SessionRepositoryPort;

/// Query handler for listing wallet sessions with details
pub struct ListWalletSessionsQueryHandler {
    session_repository: Arc<dyn SessionRepositoryPort>,
}

impl ListWalletSessionsQueryHandler {
    pub fn new(session_repository: Arc<dyn SessionRepositoryPort>) -> Self {
        Self { session_repository }
    }
}

#[async_trait]
impl QueryHandler<ListWalletSessionsQuery> for ListWalletSessionsQueryHandler {
    async fn handle(&self, query: ListWalletSessionsQuery) -> ApplicationResult<ListWalletSessionsResponse> {
        // 1. Validate query
        query.validate()?;

        // 2. Get sessions
        let all_sessions = self.session_repository
            .find_by_wallet_id(&query.wallet_address)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 3. Filter sessions based on include_expired
        let sessions: Vec<_> = if query.include_expired {
            all_sessions
        } else {
            all_sessions.into_iter().filter(|s| s.is_valid()).collect()
        };

        // 4. Map to response DTOs
        let session_infos: Vec<WalletSessionInfo> = sessions
            .iter()
            .map(|session| WalletSessionInfo {
                session_id: session.id().to_string(),
                created_at: session.created_at().to_rfc3339(),
                expires_at: session.expires_at().to_rfc3339(),
                is_active: session.is_valid(),
                device_info: session.user_agent().map(String::from),
            })
            .collect();

        Ok(ListWalletSessionsResponse {
            sessions: session_infos.clone(),
            total_count: session_infos.len(),
        })
    }
}
