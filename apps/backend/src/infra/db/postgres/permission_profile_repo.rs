use async_trait::async_trait;
use chrono::{ DateTime, Utc };
use sqlx::{ PgPool, Row };
use uuid::Uuid;

use crate::{
  app::ports::repositories::{ PermissionProfileRepo, PermissionAssignment },
  dom::entities::permission_profile::{
    PermissionProfile,
    PermissionProfileId,
    PermissionProfileQuery,
    ApplyPermissionProfileRequest,
    ApplyPermissionProfileResult,
    PermissionProfileError,
    PermissionProfileCategory,
  },
  dom::values::identifiers::UserId,
  dom::entities::iam::{ Permission, PackageTier },
};

pub struct PostgresPermissionProfileRepo {
  pool: PgPool,
}

impl PostgresPermissionProfileRepo {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  fn map_row_to_permission_profile(
    row: &sqlx::postgres::PgRow
  ) -> Result<PermissionProfile, PermissionProfileError> {
    let id: Uuid = row
      .try_get("id")
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
    let name: String = row
      .try_get("name")
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
    let description: Option<String> = row.try_get("description").ok();
    let category: String = row
      .try_get("category")
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
    let profile_data: serde_json::Value = row
      .try_get("profile_data")
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
    let is_active: bool =
      row
        .try_get::<&str, _>("status")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))? ==
      "active";
    let created_by: Option<Uuid> = row.try_get("created_by").ok();
    let created_at: DateTime<Utc> = row
      .try_get("created_at")
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
    let updated_at: DateTime<Utc> = row
      .try_get("updated_at")
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let profile_category = match category.as_str() {
      "user" => PermissionProfileCategory::User,
      "moderator" => PermissionProfileCategory::Moderator,
      "admin" => PermissionProfileCategory::Admin,
      "custom" => PermissionProfileCategory::Custom,
      "system" => PermissionProfileCategory::System,
      "business" => PermissionProfileCategory::Business,
      "technical" => PermissionProfileCategory::Technical,
      "administrative" => PermissionProfileCategory::Administrative,
      "compliance" => PermissionProfileCategory::Compliance,
      _ => PermissionProfileCategory::User,
    };

    // Extract permissions from profile_data
    let permissions_vec = if
      let Some(features) = profile_data
        .get("features")
        .and_then(|f| f.as_array())
    {
      features
        .iter()
        .filter_map(|v| v.as_str())
        .map(|p| Permission::new("feature".to_string(), p.to_string()))
        .collect()
    } else {
      Vec::new()
    };

    Ok(
      PermissionProfile::from_db(
        PermissionProfileId::from(id),
        name,
        description.unwrap_or_default(),
        // Need to determine target_tier from database or default
        PackageTier::Bronze,
        profile_category,
        is_active,
        permissions_vec,
        Vec::new(), // Empty policy attachments for now
        UserId::from(created_by.unwrap_or(Uuid::nil())),
        created_at,
        updated_at
      )
    )
  }
}

#[async_trait]
impl PermissionProfileRepo for PostgresPermissionProfileRepo {
  async fn create(
    &self,
    profile: PermissionProfile
  ) -> Result<PermissionProfile, PermissionProfileError> {
    let profile_data: serde_json::Value =
      serde_json::json!({
            "features": profile.default_permissions().iter().map(|p| p.action()).collect::<Vec<_>>(),
            "modules": [],
            "limits": {}
        });

    let pricing_tier = serde_json::json!({});
    let auto_assignment_rules = serde_json::json!({});
    let api_endpoints = serde_json::json!({});
    let frontend_routes = serde_json::json!({});

    let profile_id_str = profile.id().value().to_string();
    sqlx
      ::query(
        "INSERT INTO permission_profiles (id, name, description, category, profile_data, pricing_tier, auto_assignment_rules, api_endpoints, frontend_routes, compliance_level, created_by, created_at, updated_at, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"
      )
      .bind(&profile_id_str)
      .bind(profile.name())
      .bind(profile.description())
      .bind(match profile.category() {
        PermissionProfileCategory::User => "user",
        PermissionProfileCategory::Moderator => "moderator",
        PermissionProfileCategory::Admin => "admin",
        PermissionProfileCategory::Custom => "custom",
        PermissionProfileCategory::System => "system",
        PermissionProfileCategory::Business => "business",
        PermissionProfileCategory::Technical => "technical",
        PermissionProfileCategory::Administrative => "administrative",
        PermissionProfileCategory::Compliance => "compliance",
      })
      .bind(profile_data)
      .bind(pricing_tier)
      .bind(auto_assignment_rules)
      .bind(api_endpoints)
      .bind(frontend_routes)
      .bind("educational")
      .bind(*profile.created_by().value())
      .bind(profile.created_at())
      .bind(profile.updated_at())
      .bind(if profile.is_active() { "active" } else { "inactive" })
      .execute(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    Ok(profile)
  }

  async fn get(
    &self,
    id: &PermissionProfileId
  ) -> Result<Option<PermissionProfile>, PermissionProfileError> {
    let uuid_id = Uuid::parse_str(id.value()).map_err(|e|
      PermissionProfileError::InvalidData(e.to_string())
    )?;
    let row = sqlx
      ::query(
        "SELECT id, name, description, category, profile_data, status, created_by, created_at, updated_at 
             FROM permission_profiles WHERE id = $1"
      )
      .bind(&uuid_id)
      .fetch_optional(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    match row {
      Some(row) => Ok(Some(Self::map_row_to_permission_profile(&row)?)),
      None => Ok(None),
    }
  }

  async fn update(
    &self,
    profile: PermissionProfile
  ) -> Result<PermissionProfile, PermissionProfileError> {
    let profile_data =
      serde_json::json!({
            "features": profile.default_permissions().iter().map(|p| p.action()).collect::<Vec<_>>(),
            "modules": [],
            "limits": {}
        });

    let profile_uuid = Uuid::parse_str(profile.id().value()).map_err(|e|
      PermissionProfileError::InvalidData(e.to_string())
    )?;

    let result = sqlx
      ::query(
        "UPDATE permission_profiles SET name = $1, description = $2, category = $3, profile_data = $4, updated_at = $5, status = $6
             WHERE id = $7"
      )
      .bind(profile.name())
      .bind(profile.description())
      .bind(match profile.category() {
        PermissionProfileCategory::User => "user",
        PermissionProfileCategory::Moderator => "moderator",
        PermissionProfileCategory::Admin => "admin",
        PermissionProfileCategory::Custom => "custom",
        PermissionProfileCategory::System => "system",
        PermissionProfileCategory::Business => "business",
        PermissionProfileCategory::Technical => "technical",
        PermissionProfileCategory::Administrative => "administrative",
        PermissionProfileCategory::Compliance => "compliance",
      })
      .bind(profile_data)
      .bind(profile.updated_at())
      .bind(if profile.is_active() { "active" } else { "inactive" })
      .bind(&profile_uuid)
      .execute(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
      return Err(PermissionProfileError::NotFound);
    }

    Ok(profile)
  }

  async fn delete(
    &self,
    id: &PermissionProfileId
  ) -> Result<(), PermissionProfileError> {
    let uuid_id = Uuid::parse_str(id.value()).map_err(|e|
      PermissionProfileError::InvalidData(e.to_string())
    )?;
    let result = sqlx
      ::query(
        "UPDATE permission_profiles SET status = 'inactive', updated_at = NOW() WHERE id = $1"
      )
      .bind(&uuid_id)
      .execute(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
      return Err(PermissionProfileError::NotFound);
    }

    Ok(())
  }

  async fn search(
    &self,
    query: &PermissionProfileQuery
  ) -> Result<Vec<PermissionProfile>, PermissionProfileError> {
    let mut sql =
      "SELECT id, name, description, category, profile_data, status, created_by, created_at, updated_at FROM permission_profiles WHERE 1=1".to_string();

    if let Some(name) = &query.name {
      sql.push_str(&format!(" AND name ILIKE '%{}%'", name));
    }

    if let Some(category) = &query.category {
      sql.push_str(
        &format!(" AND category = '{}'", match category {
          PermissionProfileCategory::User => "user",
          PermissionProfileCategory::Moderator => "moderator",
          PermissionProfileCategory::Admin => "admin",
          PermissionProfileCategory::Custom => "custom",
          PermissionProfileCategory::System => "system",
          PermissionProfileCategory::Business => "business",
          PermissionProfileCategory::Technical => "technical",
          PermissionProfileCategory::Administrative => "administrative",
          PermissionProfileCategory::Compliance => "compliance",
        })
      );
    }

    if query.active_only {
      sql.push_str(" AND status = 'active'");
    }

    sql.push_str(" ORDER BY name");

    if let Some(limit) = query.limit {
      sql.push_str(&format!(" LIMIT {}", limit));
    }

    if let Some(offset) = query.offset {
      sql.push_str(&format!(" OFFSET {}", offset));
    }

    let rows = sqlx
      ::query(&sql)
      .fetch_all(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let mut profiles = Vec::new();
    for row in rows {
      profiles.push(Self::map_row_to_permission_profile(&row)?);
    }

    Ok(profiles)
  }

  async fn count(
    &self,
    query: &PermissionProfileQuery
  ) -> Result<u64, PermissionProfileError> {
    let mut sql =
      "SELECT COUNT(*) as count FROM permission_profiles WHERE 1=1".to_string();

    if let Some(name) = &query.name {
      sql.push_str(&format!(" AND name ILIKE '%{}%'", name));
    }

    if let Some(category) = &query.category {
      sql.push_str(
        &format!(" AND category = '{}'", match category {
          PermissionProfileCategory::User => "user",
          PermissionProfileCategory::Moderator => "moderator",
          PermissionProfileCategory::Admin => "admin",
          PermissionProfileCategory::Custom => "custom",
          PermissionProfileCategory::System => "system",
          PermissionProfileCategory::Business => "business",
          PermissionProfileCategory::Technical => "technical",
          PermissionProfileCategory::Administrative => "administrative",
          PermissionProfileCategory::Compliance => "compliance",
        })
      );
    }

    if query.active_only {
      sql.push_str(" AND status = 'active'");
    }

    let row = sqlx
      ::query(&sql)
      .fetch_one(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let count: i64 = row
      .try_get("count")
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
    Ok(count as u64)
  }

  async fn get_by_category(
    &self,
    category: &PermissionProfileCategory
  ) -> Result<Vec<PermissionProfile>, PermissionProfileError> {
    let rows = sqlx
      ::query(
        "SELECT id, name, description, category, profile_data, status, created_by, created_at, updated_at 
             FROM permission_profiles WHERE category = $1 AND status = 'active' ORDER BY name"
      )
      .bind(match category {
        PermissionProfileCategory::User => "user",
        PermissionProfileCategory::Moderator => "moderator",
        PermissionProfileCategory::Admin => "admin",
        PermissionProfileCategory::Custom => "custom",
        PermissionProfileCategory::System => "system",
        PermissionProfileCategory::Business => "business",
        PermissionProfileCategory::Technical => "technical",
        PermissionProfileCategory::Administrative => "administrative",
        PermissionProfileCategory::Compliance => "compliance",
      })
      .fetch_all(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let mut profiles = Vec::new();
    for row in rows {
      profiles.push(Self::map_row_to_permission_profile(&row)?);
    }

    Ok(profiles)
  }

  async fn apply_permission_profile(
    &self,
    request: &ApplyPermissionProfileRequest
  ) -> Result<ApplyPermissionProfileResult, PermissionProfileError> {
    // Begin a transaction
    let mut tx = self.pool
      .begin().await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    // Record the application
    let application_id = Uuid::new_v4();
    let applied_at = Utc::now();

    let profile_uuid = Uuid::parse_str(request.profile_id().value()).map_err(|e|
      PermissionProfileError::InvalidData(e.to_string())
    )?;
    let user_uuid = *request.user_ids
        .first()
        .map(|id| id.value())
        .unwrap_or(&Uuid::nil());
    let applied_by_uuid = *request.applied_by().value();

    sqlx
      ::query(
        "INSERT INTO profile_applications (id, profile_id, user_id, applied_by, applied_at, status)
             VALUES ($1, $2, $3, $4, $5, 'success')"
      )
      .bind(application_id)
      .bind(&profile_uuid)
      .bind(&user_uuid)
      .bind(&applied_by_uuid)
      .bind(applied_at)
      .execute(&mut *tx).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    // Commit the transaction
    tx
      .commit().await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    Ok(
      ApplyPermissionProfileResult::new(
        request.clone(),
        request.user_ids().to_vec(),
        Vec::new(), // No failed users
        vec!["Permission profile applied successfully".to_string()],
        request.applied_by().clone()
      )
    )
  }

  async fn get_application_history(
    &self,
    profile_id: &PermissionProfileId,
    limit: u32
  ) -> Result<Vec<ApplyPermissionProfileResult>, PermissionProfileError> {
    let rows = sqlx
      ::query(
        "SELECT profile_id, user_id, applied_by, applied_at, status, error_message 
             FROM profile_applications 
             WHERE profile_id = $1 
             ORDER BY applied_at DESC 
             LIMIT $2"
      )
      .bind(
        Uuid::parse_str(profile_id.value()).map_err(|e|
          PermissionProfileError::InvalidData(e.to_string())
        )?
      )
      .bind(limit as i64)
      .fetch_all(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let mut results = Vec::new();
    for row in rows {
      // Create a basic request for history reconstruction
      let basic_request = ApplyPermissionProfileRequest {
        profile_id: PermissionProfileId::new(
          row.try_get::<String, _>("profile_id").unwrap_or_default()
        ),
        user_ids: vec![
          UserId::new(row.try_get::<String, _>("user_id").unwrap_or_default())
        ],
        permission_overrides: None,
        reason: None,
        merge_permissions: true,
        expires_at: None,
        applied_by: UserId::new(
          row.try_get::<String, _>("applied_by").unwrap_or_default()
        ),
      };

      let status = row.try_get::<String, _>("status").unwrap_or_default();
      let user_id = UserId::new(
        row.try_get::<String, _>("user_id").unwrap_or_default()
      );
      let (successful_users, failed_users) = if status == "success" {
        (vec![user_id], Vec::new())
      } else {
        let error_msg = row
          .try_get::<Option<String>, _>("error_message")
          .unwrap_or_default()
          .unwrap_or_default();
        (Vec::new(), vec![(user_id, error_msg)])
      };

      results.push(
        ApplyPermissionProfileResult::new(
          basic_request,
          successful_users,
          failed_users,
          vec![format!("Status: {}", status)],
          UserId::new(
            row.try_get::<String, _>("applied_by").unwrap_or_default()
          )
        )
      );
    }

    Ok(results)
  }

  async fn can_apply_to_user(
    &self,
    profile_id: &PermissionProfileId,
    _user_id: &UserId
  ) -> Result<bool, PermissionProfileError> {
    // Get profile to check prerequisites
    let profile = self
      .get(profile_id).await?
      .ok_or(PermissionProfileError::NotFound)?;

    if !profile.is_active() {
      return Ok(false);
    }

    // Check if user meets prerequisites (simplified check)
    // In a real implementation, you'd check user's current permissions/roles
    Ok(true)
  }

  async fn get_assignment_count(
    &self,
    profile_id: &PermissionProfileId
  ) -> Result<u32, PermissionProfileError> {
    let row = sqlx
      ::query(
        "SELECT COUNT(*) as count FROM profile_applications WHERE profile_id = $1 AND status = 'success'"
      )
      .bind(
        Uuid::parse_str(profile_id.value()).map_err(|e|
          PermissionProfileError::InvalidData(e.to_string())
        )?
      )
      .fetch_one(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    Ok(row.try_get::<i64, _>("count").unwrap_or(0) as u32)
  }

  async fn initialize_defaults(
    &self,
    admin_user_id: &UserId
  ) -> Result<Vec<PermissionProfile>, PermissionProfileError> {
    let profiles = vec![
      PermissionProfile::new(
        "Basic User".to_string(),
        "Standard user permissions for platform access".to_string(),
        crate::dom::entities::iam::PackageTier::Bronze,
        PermissionProfileCategory::Business,
        admin_user_id.clone()
      ),
      PermissionProfile::new(
        "Administrator".to_string(),
        "Full administrative access to all platform features".to_string(),
        crate::dom::entities::iam::PackageTier::Admin,
        PermissionProfileCategory::Administrative,
        admin_user_id.clone()
      )
    ];

    for profile in &profiles {
      self.create(profile.clone()).await?;
    }

    Ok(profiles)
  }

  // Job system requirements
  async fn find_assignments_expiring_before(
    &self,
    cutoff_date: DateTime<Utc>
  ) -> Result<Vec<PermissionAssignment>, PermissionProfileError> {
    let rows = sqlx
      ::query(
        "SELECT user_id, permission_profile_id, assigned_at, expires_at, assigned_by, assignment_reason as reason
             FROM user_permission_profile_assignments 
             WHERE expires_at IS NOT NULL AND expires_at <= $1 AND status = 'active'"
      )
      .bind(cutoff_date)
      .fetch_all(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let mut assignments = Vec::new();
    for row in rows {
      let user_id: Uuid = row
        .try_get("user_id")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let profile_id: Uuid = row
        .try_get("permission_profile_id")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let assigned_at: DateTime<Utc> = row
        .try_get("assigned_at")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let expires_at: Option<DateTime<Utc>> = row.try_get("expires_at").ok();
      let assigned_by: String = row
        .try_get("assigned_by")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let reason: String = row
        .try_get("reason")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

      assignments.push(PermissionAssignment {
        user_id: UserId::new(user_id.to_string()),
        permission_profile_id: PermissionProfileId::from(profile_id),
        assigned_at,
        expires_at,
        assigned_by,
        reason,
        is_active: true, // Active assignments from the database
      });
    }

    Ok(assignments)
  }

  async fn revoke_assignment(
    &self,
    user_id: &UserId,
    profile_id: &PermissionProfileId
  ) -> Result<(), PermissionProfileError> {
    let user_uuid = *user_id.value();
    let profile_uuid = Uuid::parse_str(profile_id.value()).map_err(|e|
      PermissionProfileError::InvalidData(format!("Invalid profile UUID: {}", e))
    )?;

    let result = sqlx
      ::query(
        "UPDATE user_permission_profile_assignments 
             SET status = 'inactive', deactivated_at = NOW(), updated_at = NOW()
             WHERE user_id = $1 AND permission_profile_id = $2 AND status = 'active'"
      )
      .bind(user_uuid)
      .bind(profile_uuid)
      .execute(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
      return Err(PermissionProfileError::NotFound);
    }

    Ok(())
  }

  async fn cleanup_expired_assignments(
    &self
  ) -> Result<i64, PermissionProfileError> {
    let result = sqlx
      ::query(
        "UPDATE user_permission_profile_assignments 
             SET status = 'inactive', deactivated_at = NOW(), deactivation_reason = 'Expired automatically', updated_at = NOW()
             WHERE expires_at IS NOT NULL AND expires_at <= NOW() AND status = 'active'"
      )
      .execute(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    Ok(result.rows_affected() as i64)
  }

  async fn count_active_profiles(&self) -> Result<i64, PermissionProfileError> {
    let row = sqlx
      ::query(
        "SELECT COUNT(*) as count FROM permission_profiles WHERE status = 'active'"
      )
      .fetch_one(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let count: i64 = row
      .try_get("count")
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
    Ok(count)
  }

  async fn count_total_assignments(
    &self
  ) -> Result<i64, PermissionProfileError> {
    let row = sqlx
      ::query(
        "SELECT COUNT(*) as count FROM user_permission_profile_assignments"
      )
      .fetch_one(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let count: i64 = row
      .try_get("count")
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
    Ok(count)
  }

  async fn find_user_assignments_with_expiration(
    &self,
    user_id: &UserId
  ) -> Result<Vec<PermissionAssignment>, PermissionProfileError> {
    let user_uuid = *user_id.value();

    let rows = sqlx
      ::query(
        "SELECT user_id, permission_profile_id, assigned_at, expires_at, assigned_by, assignment_reason as reason
             FROM user_permission_profile_assignments 
             WHERE user_id = $1 AND expires_at IS NOT NULL AND status = 'active'"
      )
      .bind(user_uuid)
      .fetch_all(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let mut assignments = Vec::new();
    for row in rows {
      let user_id: Uuid = row
        .try_get("user_id")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let profile_id: Uuid = row
        .try_get("permission_profile_id")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let assigned_at: DateTime<Utc> = row
        .try_get("assigned_at")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let expires_at: Option<DateTime<Utc>> = row.try_get("expires_at").ok();
      let assigned_by: String = row
        .try_get("assigned_by")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let reason: String = row
        .try_get("reason")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

      assignments.push(PermissionAssignment {
        user_id: UserId::new(user_id.to_string()),
        permission_profile_id: PermissionProfileId::from(profile_id),
        assigned_at,
        expires_at,
        assigned_by,
        reason,
        is_active: true, // Active assignments from the database
      });
    }

    Ok(assignments)
  }

  async fn extend_assignment_expiration(
    &self,
    user_id: &UserId,
    profile_id: &PermissionProfileId,
    new_expiration: DateTime<Utc>
  ) -> Result<(), PermissionProfileError> {
    let user_uuid = *user_id.value();
    let profile_uuid = Uuid::parse_str(profile_id.value()).map_err(|e|
      PermissionProfileError::InvalidData(format!("Invalid profile UUID: {}", e))
    )?;

    let result = sqlx
      ::query(
        "UPDATE user_permission_profile_assignments 
             SET expires_at = $3, updated_at = NOW()
             WHERE user_id = $1 AND permission_profile_id = $2 AND status = 'active'"
      )
      .bind(user_uuid)
      .bind(profile_uuid)
      .bind(new_expiration)
      .execute(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
      return Err(PermissionProfileError::NotFound);
    }

    Ok(())
  }

  async fn find_by_id(
    &self,
    id: &PermissionProfileId
  ) -> Result<Option<PermissionProfile>, PermissionProfileError> {
    self.get(id).await
  }

  async fn health_check(&self) -> Result<(), PermissionProfileError> {
    sqlx
      ::query("SELECT 1")
      .execute(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  // Tests would be implemented here with a test database
}
