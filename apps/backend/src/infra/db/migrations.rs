// Database migration system for Diesel
use crate::infra::db::diesel::DbPool;
use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::sync::Arc;
use tracing::{info, debug};

// Embed the same migrations used by the migrate binary
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("diesel_migrations");

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

impl From<diesel::result::Error> for MigrationError {
    fn from(err: diesel::result::Error) -> Self {
        MigrationError {
            message: format!("Database error: {}", err),
        }
    }
}

impl From<diesel_migrations::MigrationError> for MigrationError {
    fn from(err: diesel_migrations::MigrationError) -> Self {
        MigrationError {
            message: format!("Migration error: {}", err),
        }
    }
}

pub struct MigrationRunner {
    pool: Arc<DbPool>,
}

impl MigrationRunner {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Run all pending migrations
    pub async fn run_migrations(&self) -> Result<(), MigrationError> {
        info!("Running pending migrations with Diesel...");
        
        let mut conn = self.get_connection().await?;
        
        let migration_versions = conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| MigrationError {
                message: format!("Failed to run migrations: {}", e)
            })?;

        if migration_versions.is_empty() {
            info!("✅ No pending migrations - database is up to date!");
        } else {
            info!("✅ Applied {} migrations successfully!", migration_versions.len());
            for version in &migration_versions {
                debug!("   - Migration: {}", version);
            }
        }

        Ok(())
    }

    /// Check migration status and return applied migrations
    pub async fn check_migration_status(&self) -> Result<Vec<String>, MigrationError> {
        debug!("Checking migration status...");
        
        let mut conn = self.get_connection().await?;
        
        let applied_migrations = conn.applied_migrations()
            .map_err(|e| MigrationError {
                message: format!("Failed to check migration status: {}", e)
            })?;
        
        let migration_list: Vec<String> = applied_migrations
            .iter()
            .map(|m| m.to_string())
            .collect();
        
        info!("📊 Applied migrations: {}", migration_list.len());
        for migration in &migration_list {
            debug!("   - {}", migration);
        }
        
        Ok(migration_list)
    }

    /// Rollback last migration
    pub async fn rollback_last_migration(&self) -> Result<(), MigrationError> {
        info!("Rolling back last migration...");
        
        let mut conn = self.get_connection().await?;
        
        conn.revert_last_migration(MIGRATIONS)
            .map_err(|e| MigrationError {
                message: format!("Failed to rollback migration: {}", e)
            })?;
            
        info!("✅ Migration rollback completed successfully");
        Ok(())
    }

    /// Helper to get a synchronous connection from the pool
    async fn get_connection(&self) -> Result<PgConnection, MigrationError> {
        // Get connection from pool
        let _conn = self.pool.get().await
            .map_err(|e| MigrationError {
                message: format!("Failed to get connection from pool: {}", e)
            })?;
        
        // Convert async connection to sync for migrations
        // Note: This is a simplified approach - in production you might want to use
        // a separate sync connection pool specifically for migrations
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| MigrationError {
                message: "DATABASE_URL environment variable not set".to_string()
            })?;
            
        let sync_conn = PgConnection::establish(&database_url)
            .map_err(|e| MigrationError {
                message: format!("Failed to establish sync connection: {}", e)
            })?;
            
        Ok(sync_conn)
    }
}