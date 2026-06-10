use crate::prelude::*;
use crate::application::shared::{Query, QueryHandler, ApplicationResult, ApplicationError};
use crate::domain::subscription_management::{
    aggregates::Plan,
    repository_ports::{PlanRepositoryPort, PlanSearchCriteria},
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ListPlansQuery {
    pub criteria: PlanSearchCriteria,
}

impl Query for ListPlansQuery {
    type Response = Vec<Plan>;
}

pub struct ListPlansQueryHandler {
    repository: Arc<dyn PlanRepositoryPort>,
}

impl ListPlansQueryHandler {
    pub fn new(repository: Arc<dyn PlanRepositoryPort>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl QueryHandler<ListPlansQuery> for ListPlansQueryHandler {
    async fn handle(&self, query: ListPlansQuery) -> ApplicationResult<Vec<Plan>> {
        self.repository.find_all(query.criteria).await.map_err(ApplicationError::from)
    }
}
