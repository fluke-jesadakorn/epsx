use crate::prelude::*;
use crate::application::shared::{Query, QueryHandler, ApplicationResult, ApplicationError};
use crate::domain::subscription_management::{
    aggregates::Plan,
    value_objects::PlanId,
    repository_ports::PlanRepositoryPort,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GetPlanQuery {
    pub id: PlanId,
}

impl Query for GetPlanQuery {
    type Response = Option<Plan>;
}

pub struct GetPlanQueryHandler {
    repository: Arc<dyn PlanRepositoryPort>,
}

impl GetPlanQueryHandler {
    pub fn new(repository: Arc<dyn PlanRepositoryPort>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl QueryHandler<GetPlanQuery> for GetPlanQueryHandler {
    async fn handle(&self, query: GetPlanQuery) -> ApplicationResult<Option<Plan>> {
        self.repository.find_by_id(&query.id).await.map_err(ApplicationError::from)
    }
}
