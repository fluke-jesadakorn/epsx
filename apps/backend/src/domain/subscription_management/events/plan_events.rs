/// Event emitted when a plan is created
#[derive(Debug, Clone)]
pub struct PlanCreatedEvent {
    pub plan_id: i32,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Event emitted when a plan is updated
#[derive(Debug, Clone)]
pub struct PlanUpdatedEvent {
    pub plan_id: i32,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Event emitted when a plan is deleted
#[derive(Debug, Clone)]
pub struct PlanDeletedEvent {
    pub plan_id: i32,
    pub deleted_at: chrono::DateTime<chrono::Utc>,
}
