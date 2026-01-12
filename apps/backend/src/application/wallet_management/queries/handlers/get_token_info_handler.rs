use crate::prelude::*;

use crate::application::shared::{
  Query,
  QueryHandler,
  ApplicationResult,
  ApplicationError,
};
use crate::application::wallet_management::queries::models::{
  GetTokenInfoQuery,
  GetTokenInfoResponse,
  TokenInfo,
};
use crate::domain::wallet_management::{
  SessionRepositoryPort,
  WalletUserRepositoryPort,
};

/// Query handler for retrieving token information
pub struct GetTokenInfoQueryHandler {
  session_repository: Arc<dyn SessionRepositoryPort>,
  wallet_repository: Arc<dyn WalletUserRepositoryPort>,
}

impl GetTokenInfoQueryHandler {
  pub fn new(
    session_repository: Arc<dyn SessionRepositoryPort>,
    wallet_repository: Arc<dyn WalletUserRepositoryPort>
  ) -> Self {
    Self {
      session_repository,
      wallet_repository,
    }
  }
}

#[async_trait]
impl QueryHandler<GetTokenInfoQuery> for GetTokenInfoQueryHandler {
  async fn handle(
    &self,
    query: GetTokenInfoQuery
  ) -> ApplicationResult<GetTokenInfoResponse> {
    // 1. Validate query
    query.validate()?;

    // 2. Find session by access token
    let session = self.session_repository
      .find_by_access_token(&query.token).await
      .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
      .ok_or_else(||
        ApplicationError::not_found(
          "Token",
          "Invalid or expired token".to_string()
        )
      )?;

    // 3. Check validity if requested
    let is_valid = if query.validate_expiry {
      session.is_valid()
    } else {
      true // Don't validate expiry if not requested
    };

    // 4. Get wallet permissions
    let wallet = self.wallet_repository
      .find_by_wallet(session.user_id()).await
      .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
      .ok_or_else(||
        ApplicationError::not_found("Wallet", session.user_id().to_string())
      )?;

    let permissions: Vec<String> = wallet
      .permissions()
      .iter()
      .filter(|p| p.is_active())
      .map(|p| p.as_str().to_string())
      .collect();

    // 5. Build token info response
    let token_info = TokenInfo {
      wallet_address: session.user_id().to_string(),
      sid: Some(session.id().to_string()),
      expires_at: session.expires_at().to_rfc3339(),
      is_valid,
      permissions,
    };

    Ok(GetTokenInfoResponse { token_info })
  }
}
