use std::sync::Arc;
use sqlx::PgPool;
use chrono::{Duration, Utc};
use tracing::{info, error, warn};

use crate::application::payment::commands::{
    ActivateSubscriptionCommand, 
    SubscriptionActivationResult
};
use crate::application::marketing::plan_service::PlanService;
use crate::infrastructure::cache::Cache;

/// Handler for subscription activation after successful payment
pub struct ActivateSubscriptionHandler {
    db_pool: Arc<PgPool>,
    plan_service: Arc<PlanService>,
    cache: Arc<dyn Cache>,
}

impl ActivateSubscriptionHandler {
    pub fn new(
        db_pool: Arc<PgPool>,
        cache: Arc<dyn Cache>,
    ) -> Self {
        let plan_service = Arc::new(PlanService::new(
            db_pool.clone(),
            cache.clone()
        ));

        Self {
            db_pool,
            plan_service,
            cache,
        }
    }

    /// Handle subscription activation command
    pub async fn handle(&self, command: ActivateSubscriptionCommand) -> Result<SubscriptionActivationResult, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Processing subscription activation for user: {}, plan: {}, payment: {}",
            command.user_id, command.plan_id, command.payment_id
        );

        // Convert user_id to UUID for database operations
        let user_uuid = uuid::Uuid::parse_str(&command.user_id.to_string())
            .map_err(|e| format!("Invalid user ID format: {}", e))?;

        // Get plan details first
        let plan = match self.plan_service.get_plan(command.plan_id).await? {
            Some(plan) => plan,
            None => {
                let error = format!("Plan {} not found", command.plan_id);
                error!("{}", error);
                return Ok(SubscriptionActivationResult::failure(
                    command.user_id,
                    command.plan_id,
                    command.transaction_hash.to_string(),
                    error,
                ));
            }
        };

        // Calculate subscription expiry (default to 30 days from now)
        let expires_at = Utc::now() + Duration::days(30);

        // Begin database transaction
        let mut tx = self.db_pool.begin().await?;

        // Verify user exists
        let user_check = sqlx::query!(
            "SELECT id FROM users WHERE id = $1",
            user_uuid
        )
        .fetch_optional(&mut *tx)
        .await?;

        if user_check.is_none() {
            let error = format!("User {} not found", command.user_id);
            warn!("{}", error);
            tx.rollback().await?;
            return Ok(SubscriptionActivationResult::failure(
                command.user_id,
                command.plan_id,
                command.transaction_hash.to_string(),
                error,
            ));
        }

        // Create subscription record (if we have a subscriptions table)
        // For now, we'll use the existing user.package_tier approach
        
        // Log the activation for audit purposes
        let log_result = sqlx::query!(
            r#"
            INSERT INTO user_subscription_activations 
            (user_id, plan_id, payment_id, transaction_hash, activated_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT DO NOTHING
            "#,
            user_uuid,
            command.plan_id,
            command.payment_id.to_string(),
            command.transaction_hash.to_string(),
            command.confirmed_at,
            expires_at
        )
        .execute(&mut *tx)
        .await;

        // If the table doesn't exist, that's ok - we'll just warn and continue
        if let Err(e) = log_result {
            warn!("Could not log subscription activation (table may not exist): {}", e);
            // Don't fail the transaction for this
        }

        // Assign permission template based on plan
        self.assign_permission_template(&mut tx, user_uuid, &plan).await?;

        // Commit transaction
        tx.commit().await?;

        // Clear user cache - Note: invalidate_user_cache method not available
        // if let Err(e) = self.cache.invalidate_user_cache(&command.user_id).await {
        //     warn!("Failed to invalidate user cache: {}", e);
        //     // Don't fail for cache errors
        // }

        // Get user's effective tier name for logging
        let display_tier = sqlx::query_scalar!(
            "SELECT get_user_display_tier($1)",
            user_uuid
        )
        .fetch_one(&*self.db_pool)
        .await
        .unwrap_or_else(|_| Some("UNKNOWN".to_string()))
        .unwrap_or_else(|| "UNKNOWN".to_string());
        
        info!(
            "Subscription activated successfully for user: {}, plan: {} ({}), effective tier: {}",
            command.user_id, plan.id, plan.name, display_tier
        );

        Ok(SubscriptionActivationResult::success(
            command.user_id,
            command.plan_id,
            plan.name,
            command.transaction_hash.to_string(),
            Some(expires_at),
        ))
    }

    /// Assign permission template to user based on plan
    async fn assign_permission_template(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_uuid: uuid::Uuid,
        plan: &crate::application::marketing::plan_service::PlanWithPromotions,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get permission template name from plan
        let template_name = derive_template_name_from_plan(plan);
        
        // Calculate expiry time (30 days from now)
        let expires_at = Utc::now() + Duration::days(30);
        
        // Use the database function to assign permission template
        let result = sqlx::query!(
            "SELECT assign_permission_template_to_user($1, $2, $3) as success",
            user_uuid,
            template_name,
            expires_at
        )
        .fetch_one(&mut **tx)
        .await;
        
        match result {
            Ok(_) => {
                info!(
                    "Successfully assigned permission template '{}' to user {} (expires: {})",
                    template_name, user_uuid, expires_at
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to assign permission template: {}", e);
                Err(Box::new(e))
            }
        }
    }
}

/// Derive permission template name from plan details
fn derive_template_name_from_plan(plan: &crate::application::marketing::plan_service::PlanWithPromotions) -> &str {
    // Map plan types to permission template names
    match plan.plan_type.to_lowercase().as_str() {
        "starter" | "basic" | "bronze" => "Bronze Template",
        "silver" => "Silver Template", 
        "professional" | "pro" | "gold" => "Gold Template",
        "platinum" => "Platinum Template",
        "enterprise" | "premium" | "vip" => "Enterprise Template",
        _ => {
            // Fallback: derive from plan name
            let name_lower = plan.name.to_lowercase();
            if name_lower.contains("enterprise") || name_lower.contains("unlimited") {
                "Enterprise Template"
            } else if name_lower.contains("platinum") {
                "Platinum Template"
            } else if name_lower.contains("gold") || name_lower.contains("pro") {
                "Gold Template"
            } else if name_lower.contains("silver") {
                "Silver Template"
            } else if name_lower.contains("bronze") {
                "Bronze Template"
            } else {
                "Bronze Template" // Default fallback
            }
        }
    }
}