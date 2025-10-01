// Unified Web3 Permission Management Service
// Uses the new unified permission groups system with wallet-first authentication
// Replaces the old user_permissions system with wallet_group_memberships

use chrono::{ DateTime, Utc };
use serde::{ Deserialize, Serialize };
use sqlx::{ PgPool, Row };
use std::collections::HashMap;
use tracing::{ debug, info, warn };
use uuid::Uuid;

use super::web3_shared_types::{ PermissionInfo, Web3PermissionResult };

/// Unified Web3 permission management service using new group-based system
#[derive(Clone)]
pub struct UnifiedWeb3PermissionService {
  db_pool: PgPool,
  #[allow(dead_code)] // TODO: Implement cache duration configuration
  cache_duration_minutes: i64,
}

impl UnifiedWeb3PermissionService {
  /// Create new unified permission service
  pub fn new(db_pool: PgPool) -> Self {
    Self {
      db_pool,
      cache_duration_minutes: 30, // 30 minute cache
    }
  }

  /// Get all effective permissions for a wallet address using new database function
  pub async fn get_wallet_permissions(
    &self,
    wallet_address: &str
  ) -> Web3PermissionResult<Vec<PermissionInfo>> {
    info!("🔍 Fetching effective permissions for wallet: {}", wallet_address);

    let lowercase_wallet = wallet_address.to_lowercase();

    // Use the new get_wallet_effective_permissions() database function
    let permissions_json: serde_json::Value = sqlx
      ::query_scalar("SELECT get_wallet_effective_permissions($1)")
      .bind(&lowercase_wallet)
      .fetch_one(&self.db_pool).await?;

    let permission_strings: Vec<String> = if
      let serde_json::Value::Array(permissions_array) = permissions_json
    {
      permissions_array
        .into_iter()
        .filter_map(|p| p.as_str().map(|s| s.to_string()))
        .collect()
    } else {
      vec![]
    };

    // Get detailed permission info from group memberships
    let permission_details = self.get_permission_details(
      &lowercase_wallet
    ).await?;

    // Convert to PermissionInfo structs
    let permissions: Vec<PermissionInfo> = permission_strings
      .into_iter()
      .map(|permission| {
        // Find matching detail or create default
        let detail = permission_details
          .iter()
          .find(|d| d.permission == permission)
          .cloned()
          .unwrap_or_else(|| PermissionInfo {
            permission: permission.clone(),
            permission_type: "group".to_string(),
            is_active: true,
            expires_at: None,
            granted_at: Utc::now(),
            last_verified_at: None,
            verification_data: None,
          });
        detail
      })
      .collect();

    info!(
      "✅ Found {} effective permissions for wallet: {}",
      permissions.len(),
      wallet_address
    );
    Ok(permissions)
  }

  /// Check if wallet has specific permission using new database function
  pub async fn has_permission(
    &self,
    wallet_address: &str,
    permission: &str
  ) -> Web3PermissionResult<bool> {
    debug!(
      "🔎 Checking permission '{}' for wallet: {}",
      permission,
      wallet_address
    );

    let lowercase_wallet = wallet_address.to_lowercase();

    // Use the new wallet_has_permission() database function
    let has_permission: Option<bool> = sqlx
      ::query_scalar("SELECT wallet_has_permission($1, $2)")
      .bind(&lowercase_wallet)
      .bind(permission)
      .fetch_one(&self.db_pool).await?;

    let has_permission = has_permission.unwrap_or(false);

    if has_permission {
      debug!(
        "✅ Permission '{}' granted for wallet: {}",
        permission,
        wallet_address
      );
    } else {
      debug!(
        "❌ Permission '{}' denied for wallet: {}",
        permission,
        wallet_address
      );
    }

    Ok(has_permission)
  }

  /// Assign wallet to permission group (replaces manual permission grants)
  pub async fn assign_to_group(
    &self,
    wallet_address: &str,
    group_id: Uuid,
    assignment_source: &str,
    assignment_reason: Option<&str>,
    expires_at: Option<DateTime<Utc>>,
    assigned_by: Option<&str>,
    payment_reference: Option<&str>,
    subscription_id: Option<&str>
  ) -> Web3PermissionResult<Uuid> {
    info!("✅ Assigning wallet '{}' to group: {}", wallet_address, group_id);

    let lowercase_wallet = wallet_address.to_lowercase();

    // Use the new assign_wallet_to_group() database function
    let membership_id: Option<Uuid> = sqlx
      ::query_scalar(
        r#"
            SELECT assign_wallet_to_group(
                $1::varchar(42), 
                $2::uuid, 
                $3::varchar(42), 
                $4::varchar(50), 
                $5::text, 
                $6::timestamptz,
                $7::varchar(255),
                $8::varchar(255)
            )
            "#
      )
      .bind(&lowercase_wallet)
      .bind(group_id)
      .bind(assigned_by)
      .bind(assignment_source)
      .bind(assignment_reason)
      .bind(expires_at)
      .bind(payment_reference)
      .bind(subscription_id)
      .fetch_one(&self.db_pool).await?;

    let membership_id = membership_id.unwrap_or_else(Uuid::new_v4);

    info!(
      "✅ Successfully assigned wallet '{}' to group {} with membership ID: {}",
      wallet_address,
      group_id,
      membership_id
    );
    Ok(membership_id)
  }

  /// Remove wallet from permission group
  pub async fn remove_from_group(
    &self,
    wallet_address: &str,
    group_id: Uuid,
    _removed_by: Option<&str>
  ) -> Web3PermissionResult<()> {
    info!("❌ Removing wallet '{}' from group: {}", wallet_address, group_id);

    let lowercase_wallet = wallet_address.to_lowercase();

    let rows_affected = sqlx
      ::query(
        r#"
            UPDATE wallet_group_memberships 
            SET 
                is_active = false,
                updated_at = NOW()
            WHERE wallet_address = $1 
            AND group_id = $2 
            AND is_active = true
            "#
      )
      .bind(lowercase_wallet)
      .bind(group_id)
      .execute(&self.db_pool).await?
      .rows_affected();

    if rows_affected > 0 {
      info!(
        "✅ Successfully removed wallet '{}' from group: {}",
        wallet_address,
        group_id
      );
    } else {
      warn!("⚠️ No active group membership found to remove for wallet: {}", wallet_address);
    }

    Ok(())
  }

  /// Verify multiple permissions for wallet (batch operation)
  pub async fn verify_permissions_batch(
    &self,
    wallet_address: &str,
    permissions: &[String]
  ) -> Web3PermissionResult<HashMap<String, bool>> {
    debug!(
      "🔎 Batch verifying {} permissions for wallet: {}",
      permissions.len(),
      wallet_address
    );

    let mut results = HashMap::new();

    // Get all effective permissions once and check against them
    let effective_permissions =
      self.get_wallet_permissions(wallet_address).await?;
    let permission_set: std::collections::HashSet<String> = effective_permissions
      .into_iter()
      .map(|p| p.permission)
      .collect();

    // Check each requested permission
    for permission in permissions {
      let has_permission =
        permission_set.contains(permission) ||
        self.check_wildcard_permission(&permission_set, permission);
      results.insert(permission.clone(), has_permission);
    }

    debug!("✅ Batch verification complete for wallet: {}", wallet_address);
    Ok(results)
  }

  /// Get permission statistics for wallet using new system
  pub async fn get_permission_stats(
    &self,
    wallet_address: &str
  ) -> Web3PermissionResult<PermissionStats> {
    let lowercase_wallet = wallet_address.to_lowercase();

    // Get membership statistics
    let _membership_stats = sqlx
      ::query(
        r#"
            SELECT 
                COUNT(*) as total_memberships,
                COUNT(*) FILTER (WHERE expires_at IS NULL) as permanent_memberships,
                COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at > NOW()) as temporary_memberships,
                COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at <= NOW()) as expired_memberships
            FROM wallet_group_memberships
            WHERE wallet_address = $1 AND is_active = true
            "#
      )
      .bind(lowercase_wallet)
      .fetch_one(&self.db_pool).await?;

    // Get effective permission count
    let effective_permissions =
      self.get_wallet_permissions(wallet_address).await?;
    let total_permissions = effective_permissions.len() as u32;

    let permanent_permissions = effective_permissions
      .iter()
      .filter(|p| p.expires_at.is_none())
      .count() as u32;

    let temporary_permissions = effective_permissions
      .iter()
      .filter(|p| p.expires_at.is_some() && p.expires_at.unwrap() > Utc::now())
      .count() as u32;

    let expired_permissions = effective_permissions
      .iter()
      .filter(|p| p.expires_at.is_some() && p.expires_at.unwrap() <= Utc::now())
      .count() as u32;

    Ok(PermissionStats {
      total_permissions,
      permanent_permissions,
      temporary_permissions,
      expired_permissions,
    })
  }

  /// Get wallet's permission group memberships
  pub async fn get_group_memberships(
    &self,
    wallet_address: &str
  ) -> Web3PermissionResult<Vec<GroupMembership>> {
    let lowercase_wallet = wallet_address.to_lowercase();

    let memberships = sqlx
      ::query_as::<_, GroupMembershipRow>(
        r#"
            SELECT 
                wgm.id,
                wgm.group_id,
                pg.name as group_name,
                pg.group_type,
                wgm.assigned_at,
                wgm.expires_at,
                wgm.is_active,
                wgm.assignment_source,
                wgm.assignment_reason,
                wgm.assigned_by,
                wgm.payment_reference,
                wgm.subscription_id,
                wgm.auto_renew,
                wgm.next_billing_date
            FROM wallet_group_memberships wgm
            JOIN permission_groups pg ON wgm.group_id = pg.id
            WHERE wgm.wallet_address = $1
            ORDER BY wgm.assigned_at DESC
            "#
      )
      .bind(lowercase_wallet)
      .fetch_all(&self.db_pool).await?;

    let group_memberships = memberships
      .into_iter()
      .map(|row| GroupMembership {
        id: row.id,
        group_id: row.group_id,
        group_name: row.group_name,
        group_type: row.group_type,
        assigned_at: row.assigned_at,
        expires_at: row.expires_at,
        is_active: row.is_active,
        assignment_source: row.assignment_source,
        assignment_reason: row.assignment_reason,
        assigned_by: row.assigned_by,
        payment_reference: row.payment_reference,
        subscription_id: row.subscription_id,
        auto_renew: row.auto_renew.unwrap_or(false),
        next_billing_date: row.next_billing_date,
      })
      .collect();

    Ok(group_memberships)
  }

  /// Helper: Get detailed permission info from group memberships
  async fn get_permission_details(
    &self,
    wallet_address: &str
  ) -> Web3PermissionResult<Vec<PermissionInfo>> {
    let memberships = sqlx
      ::query(
        r#"
            SELECT 
                wgm.assigned_at as granted_at,
                wgm.expires_at,
                pg.group_type as permission_type,
                pg.permissions
            FROM wallet_group_memberships wgm
            JOIN permission_groups pg ON wgm.group_id = pg.id
            WHERE wgm.wallet_address = $1 
            AND wgm.is_active = true
            AND pg.is_active = true
            AND (wgm.expires_at IS NULL OR wgm.expires_at > NOW())
            "#
      )
      .bind(wallet_address)
      .fetch_all(&self.db_pool).await?;

    let mut permission_details = Vec::new();

    for membership in memberships {
      let permissions: serde_json::Value = membership.try_get("permissions")?;
      if let serde_json::Value::Array(permissions_array) = permissions {
        for permission_value in permissions_array {
          if let Some(permission_str) = permission_value.as_str() {
            permission_details.push(PermissionInfo {
              permission: permission_str.to_string(),
              permission_type: membership.try_get("permission_type")?,
              is_active: true,
              expires_at: membership.try_get("expires_at")?,
              granted_at: membership.try_get("granted_at")?,
              last_verified_at: Some(Utc::now()),
              verification_data: None,
            });
          }
        }
      }
    }

    Ok(permission_details)
  }

  /// Helper: Check wildcard permission patterns
  fn check_wildcard_permission(
    &self,
    permission_set: &std::collections::HashSet<String>,
    requested_permission: &str
  ) -> bool {
    // Check for admin wildcard (admin:*:*)
    if permission_set.contains("admin:*:*") {
      return true;
    }

    let permission_parts: Vec<&str> = requested_permission.split(':').collect();
    if permission_parts.len() >= 2 {
      // Check platform wildcard (epsx:*:*)
      let platform_wildcard = format!("{}:*:*", permission_parts[0]);
      if permission_set.contains(&platform_wildcard) {
        return true;
      }

      // Check resource wildcard (platform:resource:*)
      if permission_parts.len() >= 2 {
        let resource_wildcard = format!(
          "{}:{}:*",
          permission_parts[0],
          permission_parts[1]
        );
        if permission_set.contains(&resource_wildcard) {
          return true;
        }
      }
    }

    false
  }
}

/// Permission statistics for a wallet
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionStats {
  pub total_permissions: u32,
  pub permanent_permissions: u32,
  pub temporary_permissions: u32,
  pub expired_permissions: u32,
}

/// Group membership information
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupMembership {
  pub id: Uuid,
  pub group_id: Uuid,
  pub group_name: String,
  pub group_type: String,
  pub assigned_at: DateTime<Utc>,
  pub expires_at: Option<DateTime<Utc>>,
  pub is_active: bool,
  pub assignment_source: String,
  pub assignment_reason: Option<String>,
  pub assigned_by: Option<String>,
  pub payment_reference: Option<String>,
  pub subscription_id: Option<String>,
  pub auto_renew: bool,
  pub next_billing_date: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct GroupMembershipRow {
  id: Uuid,
  group_id: Uuid,
  group_name: String,
  group_type: String,
  assigned_at: DateTime<Utc>,
  expires_at: Option<DateTime<Utc>>,
  is_active: bool,
  assignment_source: String,
  assignment_reason: Option<String>,
  assigned_by: Option<String>,
  payment_reference: Option<String>,
  subscription_id: Option<String>,
  auto_renew: Option<bool>,
  next_billing_date: Option<DateTime<Utc>>,
}
