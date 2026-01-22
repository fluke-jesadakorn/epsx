use crate::prelude::*;
use crate::domain::permission_management::PlanId;
use crate::domain::wallet_management::WalletAddress;

/// Plan Assignment Entity
/// Represents the assignment of a wallet to a permission plan
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlanAssignment {
    id: uuid::Uuid,
    plan_id: PlanId,
    wallet_address: WalletAddress,
    assigned_at: chrono::DateTime<chrono::Utc>,
    assigned_by: Option<WalletAddress>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    is_active: bool,
}

impl PlanAssignment {
    pub fn new(
        plan_id: PlanId,
        wallet_address: WalletAddress,
        assigned_by: Option<WalletAddress>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            plan_id,
            wallet_address,
            assigned_at: chrono::Utc::now(),
            assigned_by,
            expires_at,
            is_active: true,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn revoke(&mut self) {
        self.is_active = false;
    }

    // Getters
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    pub fn plan_id(&self) -> &PlanId {
        &self.plan_id
    }

    pub fn wallet_address(&self) -> &WalletAddress {
        &self.wallet_address
    }

    pub fn assigned_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.assigned_at
    }

    pub fn expires_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.expires_at
    }

    pub fn is_active(&self) -> bool {
        self.is_active && !self.is_expired()
    }
}
