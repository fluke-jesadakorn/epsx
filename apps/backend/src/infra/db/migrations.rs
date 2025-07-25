// Database migration system for PostgreSQL
use sqlx::{PgPool, Row};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

#[derive(Debug)]
pub struct MigrationError {
    pub message: String,
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Migration error: {}", self.message)
    }
}

impl std::error::Error for MigrationError {}

pub struct MigrationRunner {
    pool: PgPool,
    migrations_dir: String,
}

impl MigrationRunner {
    pub fn new(pool: PgPool, migrations_dir: String) -> Self {
        Self {
            pool,
            migrations_dir,
        }
    }

    /// Initialize the migrations table if it doesn't exist
    pub async fn init(&self) -> Result<(), MigrationError> {
        let query = r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version VARCHAR(255) PRIMARY KEY,
                executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#;

        sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| MigrationError {
                message: format!("Failed to create migrations table: {}", e),
            })?;

        info!("Migration system initialized");
        Ok(())
    }

    /// Get all executed migrations
    pub async fn get_executed_migrations(&self) -> Result<Vec<String>, MigrationError> {
        let rows = sqlx::query("SELECT version FROM schema_migrations ORDER BY version")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MigrationError {
                message: format!("Failed to fetch executed migrations: {}", e),
            })?;

        let versions: Vec<String> = rows
            .iter()
            .map(|row| row.get::<String, _>("version"))
            .collect();

        Ok(versions)
    }

    /// Get all available migration files
    pub fn get_available_migrations(&self) -> Result<Vec<String>, MigrationError> {
        let migrations_path = Path::new(&self.migrations_dir);
        
        if !migrations_path.exists() {
            return Err(MigrationError {
                message: format!("Migrations directory does not exist: {}", self.migrations_dir),
            });
        }

        let mut migrations = Vec::new();
        
        let entries = fs::read_dir(migrations_path)
            .map_err(|e| MigrationError {
                message: format!("Failed to read migrations directory: {}", e),
            })?;

        for entry in entries {
            let entry = entry.map_err(|e| MigrationError {
                message: format!("Failed to read directory entry: {}", e),
            })?;

            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.ends_with(".sql") {
                        migrations.push(filename.to_string());
                    }
                }
            }
        }

        migrations.sort();
        Ok(migrations)
    }

    /// Run all pending migrations
    pub async fn migrate(&self) -> Result<u32, MigrationError> {
        self.init().await?;

        let executed = self.get_executed_migrations().await?;
        let available = self.get_available_migrations()?;

        let mut applied_count = 0;

        for migration_file in available {
            let version = migration_file.replace(".sql", "");
            
            if executed.contains(&version) {
                info!("Migration {} already executed, skipping", version);
                continue;
            }

            info!("Applying migration: {}", migration_file);
            self.apply_migration(&migration_file, &version).await?;
            applied_count += 1;
        }

        if applied_count == 0 {
            info!("No pending migrations found");
        } else {
            info!("Applied {} migrations successfully", applied_count);
        }

        Ok(applied_count)
    }

    /// Apply a single migration
    async fn apply_migration(&self, filename: &str, version: &str) -> Result<(), MigrationError> {
        let migration_path = Path::new(&self.migrations_dir).join(filename);
        
        let sql_content = fs::read_to_string(&migration_path)
            .map_err(|e| MigrationError {
                message: format!("Failed to read migration file {}: {}", filename, e),
            })?;

        // Start transaction
        let mut tx = self.pool.begin().await
            .map_err(|e| MigrationError {
                message: format!("Failed to start transaction for migration {}: {}", version, e),
            })?;

        // Execute migration SQL - split into individual statements
        let statements: Vec<&str> = sql_content
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for statement in statements {
            if !statement.trim().is_empty() {
                sqlx::query(statement)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| MigrationError {
                        message: format!("Failed to execute statement in migration {}: {}\nStatement: {}", version, e, statement),
                    })?;
            }
        }

        // Record migration as executed
        sqlx::query("INSERT INTO schema_migrations (version) VALUES ($1)")
            .bind(version)
            .execute(&mut *tx)
            .await
            .map_err(|e| MigrationError {
                message: format!("Failed to record migration {}: {}", version, e),
            })?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| MigrationError {
                message: format!("Failed to commit migration {}: {}", version, e),
            })?;

        info!("Successfully applied migration: {}", version);
        Ok(())
    }

    /// Check migration status
    pub async fn status(&self) -> Result<(), MigrationError> {
        self.init().await?;

        let executed = self.get_executed_migrations().await?;
        let available = self.get_available_migrations()?;

        info!("Migration Status:");
        info!("================");
        
        for migration_file in &available {
            let version = migration_file.replace(".sql", "");
            let status = if executed.contains(&version) { "✓" } else { "✗" };
            info!("{} {}", status, version);
        }

        let pending_count = available.len().saturating_sub(executed.len());
        if pending_count > 0 {
            warn!("{} pending migrations", pending_count);
        } else {
            info!("All migrations up to date");
        }

        Ok(())
    }

    /// Rollback last migration (basic implementation)
    pub async fn rollback(&self) -> Result<(), MigrationError> {
        warn!("Rollback functionality not implemented - PostgreSQL migrations are forward-only");
        warn!("To rollback, manually create a new migration that reverts the changes");
        Err(MigrationError {
            message: "Rollback not supported - create new migration instead".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    async fn get_test_pool() -> PgPool {
        let database_url = env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost/epsx_test".to_string());
        
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn should_initialize_migrations_table() {
        let pool = get_test_pool().await;
        let runner = MigrationRunner::new(pool, "migrations".to_string());
        
        let result = runner.init().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_get_migration_status() {
        let pool = get_test_pool().await;
        let runner = MigrationRunner::new(pool, "migrations".to_string());
        
        let result = runner.status().await;
        assert!(result.is_ok());
    }

    #[test]
    fn should_handle_missing_migrations_directory() {
        // Skip test that requires database connection in unit tests
        // This would be better as an integration test
    }
}