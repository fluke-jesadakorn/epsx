use crate::prelude::*;

use crate::application::shared::{
  Query,
  QueryHandler,
  ApplicationResult,
  ApplicationError,
};
use crate::application::wallet_management::queries::models::{
  GetSessionQuery,
  GetSessionResponse,
};
use crate::domain::wallet_management::{ SessionRepositoryPort, SessionId };

/// Query handler for retrieving session information
pub struct GetSessionQueryHandler {
  session_repository: Arc<dyn SessionRepositoryPort>,
}

impl GetSessionQueryHandler {
  pub fn new(session_repository: Arc<dyn SessionRepositoryPort>) -> Self {
    Self { session_repository }
  }
}

#[async_trait]
impl QueryHandler<GetSessionQuery> for GetSessionQueryHandler {
  async fn handle(
    &self,
    query: GetSessionQuery
  ) -> ApplicationResult<GetSessionResponse> {
    // 1. Validate query
    query.validate()?;

    // 2. Parse session ID
    let sid = SessionId::from(query.sid.clone());

    // 3. Find session
    let session = self.session_repository
      .find_by_id(&sid).await
      .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
      .ok_or_else(||
        ApplicationError::not_found("Session", query.sid.clone())
      )?;

    // 4. Build response
    Ok(GetSessionResponse {
      sid: session.id().to_string(),
      wallet_address: session.user_id().to_string(),
      is_valid: session.is_valid(),
      created_at: session.created_at(),
      expires_at: session.expires_at(),
    })
  }
}
