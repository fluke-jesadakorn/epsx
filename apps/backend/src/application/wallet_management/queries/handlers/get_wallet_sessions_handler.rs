use crate::prelude::*;

use crate::application::shared::{
  Query,
  QueryHandler,
  ApplicationResult,
  ApplicationError,
};
use crate::application::wallet_management::queries::models::{
  GetWalletSessionsQuery,
  GetWalletSessionsResponse,
};
use crate::domain::wallet_management::{ SessionRepositoryPort, WalletAddress };

/// Query handler for retrieving all sessions for a wallet
pub struct GetWalletSessionsQueryHandler {
  session_repository: Arc<dyn SessionRepositoryPort>,
}

impl GetWalletSessionsQueryHandler {
  pub fn new(session_repository: Arc<dyn SessionRepositoryPort>) -> Self {
    Self { session_repository }
  }
}

#[async_trait]
impl QueryHandler<GetWalletSessionsQuery> for GetWalletSessionsQueryHandler {
  async fn handle(
    &self,
    query: GetWalletSessionsQuery
  ) -> ApplicationResult<GetWalletSessionsResponse> {
    // 1. Validate query
    query.validate()?;

    // 2. Parse wallet address
    let wallet_addr = WalletAddress::new(query.wallet_address.clone()).map_err(
      |e| ApplicationError::validation("wallet_address", e.to_string())
    )?;

    // 3. Get all sessions
    let sessions = self.session_repository
      .find_by_wallet_id(&wallet_addr.to_user_id()).await
      .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

    // 4. Separate active and expired sessions with IP address tracking
    let active_sessions: Vec<crate::application::wallet_management::queries::models::get_wallet_sessions::SessionSummary> =
      sessions
        .iter()
        .filter(|s| s.is_valid())
        .map(
          |
            s
          | crate::application::wallet_management::queries::models::get_wallet_sessions::SessionSummary {
            sid: s.id().to_string(),
            created_at: s.created_at(),
            expires_at: s.expires_at(),
            is_valid: s.is_valid(),
            ip_address: s.ip_address().map(|ip| ip.to_string()),
          }
        )
        .collect();

    let expired_sessions: Vec<crate::application::wallet_management::queries::models::get_wallet_sessions::SessionSummary> =
      if query.include_expired {
        sessions
          .iter()
          .filter(|s| !s.is_valid())
          .map(
            |
              s
            | crate::application::wallet_management::queries::models::get_wallet_sessions::SessionSummary {
              sid: s.id().to_string(),
              created_at: s.created_at(),
              expires_at: s.expires_at(),
              is_valid: s.is_valid(),
              ip_address: s.ip_address().map(|ip| ip.to_string()),
            }
          )
          .collect()
      } else {
        Vec::new()
      };

    Ok(GetWalletSessionsResponse {
      wallet_address: query.wallet_address,
      active_sessions,
      expired_sessions,
    })
  }
}
