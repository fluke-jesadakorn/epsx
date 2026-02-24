use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult};
use crate::application::market_analytics::queries::{
    GetAdminModulesQuery, GetAdminModulesResponse,
};

/// Query handler for getting admin module statistics
pub struct GetAdminModulesQueryHandler {}

impl Default for GetAdminModulesQueryHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GetAdminModulesQueryHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl QueryHandler<GetAdminModulesQuery> for GetAdminModulesQueryHandler {
    async fn handle(
        &self,
        _query: GetAdminModulesQuery,
    ) -> ApplicationResult<GetAdminModulesResponse> {
        Ok(GetAdminModulesResponse {
            success: true,
            modules: Vec::new(),
            total_active: 0,
            total_inactive: 0,
        })
    }
}
