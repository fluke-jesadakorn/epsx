use crate::prelude::*;
use crate::application::shared::Command;
use crate::domain::subscription_management::PlanId;

/// Command to delete a plan
#[derive(Debug, Clone)]
pub struct DeletePlanCommand {
    pub id: PlanId,
}

impl Command for DeletePlanCommand {
    type Response = DeletePlanResponse;
}

/// Response for delete plan command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePlanResponse {
    pub plan_id: String,
    pub deleted: bool,
}
