use crate::prelude::*;
use crate::application::shared::Command;

/// Command to delete a plan
#[derive(Debug, Clone)]
pub struct DeletePlanCommand {
    pub plan_id: i32,
}

impl Command for DeletePlanCommand {
    type Response = DeletePlanResponse;
}

/// Response for delete plan command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePlanResponse {
    pub plan_id: i32,
    pub deleted: bool,
}
