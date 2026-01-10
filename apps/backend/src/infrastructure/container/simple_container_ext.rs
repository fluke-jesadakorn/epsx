
    pub fn get_plan_repository_port(&self) -> Option<Arc<dyn crate::domain::subscription_management::repository_ports::PlanRepositoryPort>> {
         self.plan_repository.as_ref().map(|repo| Arc::clone(repo) as Arc<dyn crate::domain::subscription_management::repository_ports::PlanRepositoryPort>)
    }
