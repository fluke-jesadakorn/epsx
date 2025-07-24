use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    app::ports::repositories::TemplateRepo,
    dom::entities::template::{
        RoleTemplate, TemplateId, TemplateQuery, ApplyTemplateRequest, 
        ApplyTemplateResult, TemplateError, TemplateCategory
    },
    dom::values::identifiers::UserId,
    dom::entities::iam::{Permission, PackageTier},
};

pub struct PostgresTemplateRepo {
    pool: PgPool,
}

impl PostgresTemplateRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row_to_template(row: &sqlx::postgres::PgRow) -> Result<RoleTemplate, TemplateError> {
        let id: Uuid = row.try_get("id").map_err(|e| TemplateError::DatabaseError(e.to_string()))?;
        let name: String = row.try_get("name").map_err(|e| TemplateError::DatabaseError(e.to_string()))?;
        let description: Option<String> = row.try_get("description").ok();
        let category: String = row.try_get("category").map_err(|e| TemplateError::DatabaseError(e.to_string()))?;
        let permissions: Vec<String> = row.try_get::<serde_json::Value, _>("permissions")
            .map_err(|e| TemplateError::DatabaseError(e.to_string()))?
            .as_array()
            .ok_or_else(|| TemplateError::InvalidData("permissions must be array".to_string()))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        let _prerequisites: Vec<String> = row.try_get::<serde_json::Value, _>("prerequisites")
            .map_err(|e| TemplateError::DatabaseError(e.to_string()))?
            .as_array()
            .ok_or_else(|| TemplateError::InvalidData("prerequisites must be array".to_string()))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        let is_active: bool = row.try_get("is_active").map_err(|e| TemplateError::DatabaseError(e.to_string()))?;
        let created_by: Uuid = row.try_get("created_by").map_err(|e| TemplateError::DatabaseError(e.to_string()))?;
        let created_at: DateTime<Utc> = row.try_get("created_at").map_err(|e| TemplateError::DatabaseError(e.to_string()))?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at").map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        let template_category = match category.as_str() {
            "business" => TemplateCategory::Business,
            "technical" => TemplateCategory::Technical,
            "administrative" => TemplateCategory::Administrative,
            "compliance" => TemplateCategory::Compliance,
            _ => return Err(TemplateError::InvalidData(format!("Invalid category: {}", category))),
        };

        // Convert string permissions to Permission objects
        let permissions_vec: Vec<Permission> = permissions
            .into_iter()
            .map(|p| {
                // Parse permission string (e.g., "users:read/*")
                let parts: Vec<&str> = p.splitn(2, '/').collect();
                if parts.len() == 2 {
                    Permission::new(parts[0].to_string(), parts[1].to_string())
                } else {
                    Permission::new(p, "*".to_string())
                }
            })
            .collect();

        Ok(RoleTemplate::from_db(
            TemplateId::from(id),
            name,
            description.unwrap_or_default(),
            // Need to determine target_tier from database or default
            PackageTier::Bronze,
            template_category,
            is_active,
            permissions_vec,
            Vec::new(), // Empty policy attachments for now
            UserId::new(created_by.to_string()),
            created_at,
            updated_at,
        ))
    }
}

#[async_trait]
impl TemplateRepo for PostgresTemplateRepo {
    async fn create(&self, template: RoleTemplate) -> Result<RoleTemplate, TemplateError> {
        let permissions_json = serde_json::to_value(template.default_permissions())
            .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;
        let prerequisites_json = serde_json::to_value(&template.metadata().prerequisites)
            .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO role_templates (id, name, description, category, permissions, prerequisites, is_active, created_by, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
        .bind(template.id().value())
        .bind(template.name())
        .bind(template.description())
        .bind(template.category().to_string())
        .bind(permissions_json)
        .bind(prerequisites_json)
        .bind(template.is_active())
        .bind(template.created_by().value())
        .bind(template.created_at())
        .bind(template.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        Ok(template)
    }

    async fn get(&self, id: &TemplateId) -> Result<Option<RoleTemplate>, TemplateError> {
        let row = sqlx::query(
            "SELECT id, name, description, category, permissions, prerequisites, is_active, created_by, created_at, updated_at 
             FROM role_templates WHERE id = $1")
        .bind(id.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(Self::map_row_to_template(&row)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, template: RoleTemplate) -> Result<RoleTemplate, TemplateError> {
        let permissions_json = serde_json::to_value(template.default_permissions())
            .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;
        let prerequisites_json = serde_json::to_value(&template.metadata().prerequisites)
            .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        let result = sqlx::query(
            "UPDATE role_templates SET name = $1, description = $2, category = $3, permissions = $4, prerequisites = $5, is_active = $6, updated_at = $7
             WHERE id = $8")
        .bind(template.name())
        .bind(template.description())
        .bind(template.category().to_string())
        .bind(permissions_json)
        .bind(prerequisites_json)
        .bind(template.is_active())
        .bind(template.updated_at())
        .bind(template.id().value())
        .execute(&self.pool)
        .await
        .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(TemplateError::NotFound);
        }

        Ok(template)
    }

    async fn delete(&self, id: &TemplateId) -> Result<(), TemplateError> {
        let result = sqlx::query(
            "UPDATE role_templates SET is_active = false, updated_at = NOW() WHERE id = $1")
        .bind(id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(TemplateError::NotFound);
        }

        Ok(())
    }

    async fn search(&self, query: &TemplateQuery) -> Result<Vec<RoleTemplate>, TemplateError> {
        let mut sql = "SELECT id, name, description, category, permissions, prerequisites, is_active, created_by, created_at, updated_at FROM role_templates WHERE 1=1".to_string();

        if let Some(name) = &query.name {
            sql.push_str(&format!(" AND name ILIKE '%{}%'", name));
        }

        if let Some(category) = &query.category {
            sql.push_str(&format!(" AND category = '{}'", category.to_string()));
        }

        if query.active_only {
            sql.push_str(" AND is_active = true");
        }

        sql.push_str(" ORDER BY name");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(Self::map_row_to_template(&row)?);
        }

        Ok(templates)
    }

    async fn count(&self, query: &TemplateQuery) -> Result<u64, TemplateError> {
        let mut sql = "SELECT COUNT(*) as count FROM role_templates WHERE 1=1".to_string();

        if let Some(name) = &query.name {
            sql.push_str(&format!(" AND name ILIKE '%{}%'", name));
        }

        if let Some(category) = &query.category {
            sql.push_str(&format!(" AND category = '{}'", category.to_string()));
        }

        if query.active_only {
            sql.push_str(" AND is_active = true");
        }

        let row = sqlx::query(&sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        let count: i64 = row.try_get("count").map_err(|e| TemplateError::DatabaseError(e.to_string()))?;
        Ok(count as u64)
    }

    async fn get_by_category(&self, category: &TemplateCategory) -> Result<Vec<RoleTemplate>, TemplateError> {
        let rows = sqlx::query(
            "SELECT id, name, description, category, permissions, prerequisites, is_active, created_by, created_at, updated_at 
             FROM role_templates WHERE category = $1 AND is_active = true ORDER BY name")
        .bind(category.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(Self::map_row_to_template(&row)?);
        }

        Ok(templates)
    }

    async fn apply_template(&self, request: &ApplyTemplateRequest) -> Result<ApplyTemplateResult, TemplateError> {
        // Begin a transaction
        let mut tx = self.pool.begin().await
            .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        // Record the application
        let application_id = Uuid::new_v4();
        let applied_at = Utc::now();

        sqlx::query(
            "INSERT INTO template_applications (id, template_id, user_id, applied_by, applied_at, status)
             VALUES ($1, $2, $3, $4, $5, 'success')")
        .bind(application_id)
        .bind(request.template_id.value())
        .bind(request.user_ids.first().map(|id| id.value()).unwrap_or(&uuid::Uuid::nil()))
        .bind(request.applied_by.value())
        .bind(applied_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        // Commit the transaction
        tx.commit().await
            .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        Ok(ApplyTemplateResult::new(
            request.clone(),
            request.user_ids.clone(),
            Vec::new(), // No failed users
            vec!["Template applied successfully".to_string()],
            request.applied_by.clone(),
        ))
    }

    async fn get_application_history(&self, template_id: &TemplateId, limit: u32) -> Result<Vec<ApplyTemplateResult>, TemplateError> {
        let rows = sqlx::query(
            "SELECT template_id, user_id, applied_by, applied_at, status, error_message 
             FROM template_applications 
             WHERE template_id = $1 
             ORDER BY applied_at DESC 
             LIMIT $2")
        .bind(template_id.value())
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            // Create a basic request for history reconstruction
            let basic_request = ApplyTemplateRequest {
                template_id: TemplateId::from(row.try_get::<String, _>("template_id").unwrap_or_default()),
                user_ids: vec![UserId::from(row.try_get::<String, _>("user_id").unwrap_or_default())],
                permission_overrides: None,
                reason: None,
                merge_permissions: true,
                expires_at: None,
                applied_by: UserId::from(row.try_get::<String, _>("applied_by").unwrap_or_default()),
            };
            
            let status = row.try_get::<String, _>("status").unwrap_or_default();
            let user_id = UserId::from(row.try_get::<String, _>("user_id").unwrap_or_default());
            let (successful_users, failed_users) = if status == "success" {
                (vec![user_id], Vec::new())
            } else {
                let error_msg = row.try_get::<Option<String>, _>("error_message").unwrap_or_default().unwrap_or_default();
                (Vec::new(), vec![(user_id, error_msg)])
            };
            
            results.push(ApplyTemplateResult::new(
                basic_request,
                successful_users,
                failed_users,
                vec![format!("Status: {}", status)],
                UserId::from(row.try_get::<String, _>("applied_by").unwrap_or_default()),
            ));
        }

        Ok(results)
    }

    async fn can_apply_to_user(&self, template_id: &TemplateId, _user_id: &UserId) -> Result<bool, TemplateError> {
        // Get template to check prerequisites
        let template = self.get(template_id).await?
            .ok_or(TemplateError::NotFound)?;

        if !template.is_active() {
            return Ok(false);
        }

        // Check if user meets prerequisites (simplified check)
        // In a real implementation, you'd check user's current permissions/roles
        Ok(true)
    }

    async fn get_assignment_count(&self, template_id: &TemplateId) -> Result<u32, TemplateError> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM template_applications WHERE template_id = $1 AND status = 'success'")
        .bind(template_id.value())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| TemplateError::DatabaseError(e.to_string()))?;

        Ok(row.try_get::<i64, _>("count").unwrap_or(0) as u32)
    }

    async fn initialize_defaults(&self, admin_user_id: &UserId) -> Result<Vec<RoleTemplate>, TemplateError> {
        let templates = vec![
            RoleTemplate::new(
                "Basic User".to_string(),
                "Standard user permissions for platform access".to_string(),
                crate::dom::entities::iam::PackageTier::Bronze,
                TemplateCategory::Business,
                admin_user_id.clone(),
            ),
            RoleTemplate::new(
                "Administrator".to_string(),
                "Full administrative access to all platform features".to_string(),
                crate::dom::entities::iam::PackageTier::Admin,
                TemplateCategory::Administrative,
                admin_user_id.clone(),
            ),
        ];

        for template in &templates {
            self.create(template.clone()).await?;
        }

        Ok(templates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Tests would be implemented here with a test database
}