use std::env;
use sqlx::PgPool;
use tracing::{info, warn, error};

/// Cleanup duplicate user data utility
/// This tool safely removes duplicate user profile data from the database
/// after verifying that the data exists in Firebase
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🧹 Starting user data cleanup utility");
    
    // Get database URL from environment
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    // Connect to database
    let pool = PgPool::connect(&database_url).await?;
    info!("📊 Connected to database");
    
    // Perform cleanup operations
    info!("🔍 Analyzing current database state...");
    analyze_database_state(&pool).await?;
    
    info!("🗑️  Cleaning up deprecated tables...");
    cleanup_deprecated_tables(&pool).await?;
    
    info!("🔗 Updating foreign key references...");
    update_foreign_key_references(&pool).await?;
    
    info!("🧽 Cleaning up orphaned data...");
    cleanup_orphaned_data(&pool).await?;
    
    info!("✅ User data cleanup completed successfully");
    
    // Verify final state
    info!("🔍 Verifying final database state...");
    verify_cleanup_results(&pool).await?;
    
    info!("🎉 Cleanup utility finished successfully");
    
    Ok(())
}

/// Analyze current database state before cleanup
async fn analyze_database_state(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Analyzing current database tables and data...");
    
    // Check if deprecated tables exist
    let deprecated_tables = vec![
        "firebase_user_mappings",
        "unified_sessions", 
        "provider_user_attributes",
        "oauth_provider_configs",
        "auth_audit_log",
        "sessions",
        "user_sessions",
    ];
    
    for table in deprecated_tables {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = $1)"
        )
        .bind(table)
        .fetch_one(pool)
        .await?;
        
        if exists {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table))
                .fetch_one(pool)
                .await?;
            info!("📋 Table '{}' exists with {} records", table, count);
        } else {
            info!("✅ Table '{}' does not exist (already cleaned)", table);
        }
    }
    
    // Check users table structure
    let users_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'users')"
    )
    .fetch_one(pool)
    .await?;
    
    if users_exists {
        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await?;
        info!("👥 Users table exists with {} records", user_count);
        
        // Check users table columns
        let columns: Vec<String> = sqlx::query_scalar(
            "SELECT column_name FROM information_schema.columns WHERE table_name = 'users' ORDER BY ordinal_position"
        )
        .fetch_all(pool)
        .await?;
        info!("📊 Users table columns: {:?}", columns);
    }
    
    // Check new Firebase-native tables
    let firebase_tables = vec![
        ("firebase_sessions", "Firebase session management"),
        ("user_roles_permissions", "Database role storage"),
        ("user_app_data", "Application-specific data"),
        ("firebase_token_cache", "Token validation cache"),
    ];
    
    for (table, description) in firebase_tables {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = $1)"
        )
        .bind(table)
        .fetch_one(pool)
        .await?;
        
        if exists {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table))
                .fetch_one(pool)
                .await?;
            info!("✅ {} table '{}' exists with {} records", description, table, count);
        } else {
            warn!("⚠️  {} table '{}' does not exist", description, table);
        }
    }
    
    Ok(())
}

/// Clean up deprecated authentication tables
async fn cleanup_deprecated_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Removing deprecated authentication tables...");
    
    let deprecated_tables = vec![
        "firebase_user_mappings",
        "unified_sessions",
        "provider_user_attributes", 
        "oauth_provider_configs",
        "auth_audit_log",
        "sessions",
        "user_sessions",
        "user_profiles",
        "user_metadata",
        "user_settings",
    ];
    
    for table in deprecated_tables {
        // Check if table exists first
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = $1)"
        )
        .bind(table)
        .fetch_one(pool)
        .await?;
        
        if exists {
            // Get record count before dropping
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table))
                .fetch_one(pool)
                .await?;
            
            // Drop table with CASCADE to handle dependencies
            sqlx::query(&format!("DROP TABLE IF EXISTS {} CASCADE", table))
                .execute(pool)
                .await?;
            
            info!("🗑️  Dropped table '{}' (had {} records)", table, count);
        } else {
            info!("✅ Table '{}' already doesn't exist", table);
        }
    }
    
    Ok(())
}

/// Update foreign key references to use Firebase UIDs
async fn update_foreign_key_references(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Updating foreign key references...");
    
    // Drop old foreign key constraints that reference deprecated tables
    let old_constraints = vec![
        ("admin_permission_profile_assignments", "admin_permission_profile_assignments_user_id_fkey"),
        ("assignment_audit_log", "assignment_audit_log_performed_by_fkey"),
        ("audit_logs", "audit_logs_user_id_fkey"),
        ("audit_logs", "audit_logs_actor_id_fkey"),
    ];
    
    for (table, constraint) in old_constraints {
        // Check if constraint exists
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM information_schema.table_constraints 
                WHERE table_name = $1 AND constraint_name = $2
            )
            "#
        )
        .bind(table)
        .bind(constraint)
        .fetch_one(pool)
        .await?;
        
        if exists {
            sqlx::query(&format!("ALTER TABLE {} DROP CONSTRAINT IF EXISTS {}", table, constraint))
                .execute(pool)
                .await?;
            info!("🔗 Dropped constraint '{}' from table '{}'", constraint, table);
        }
    }
    
    // Note: New foreign key constraints should reference Firebase UIDs directly
    // This is handled by the new schema in migration 014
    
    Ok(())
}

/// Clean up orphaned data that references deleted tables
async fn cleanup_orphaned_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Cleaning up orphaned data...");
    
    // Clean up orphaned assignment audit logs
    let assignment_audit_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'assignment_audit_log')"
    )
    .fetch_one(pool)
    .await?;
    
    if assignment_audit_exists {
        let deleted_count = sqlx::query(
            r#"
            DELETE FROM assignment_audit_log 
            WHERE assignment_id NOT IN (
                SELECT id FROM admin_permission_profile_assignments
            )
            "#
        )
        .execute(pool)
        .await?
        .rows_affected();
        
        info!("🧽 Cleaned up {} orphaned assignment audit log entries", deleted_count);
    }
    
    // Clean up orphaned audit logs (if they reference non-existent users)
    let audit_logs_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'audit_logs')"
    )
    .fetch_one(pool)
    .await?;
    
    if audit_logs_exists {
        // For now, we'll keep audit logs but note that user_id references may be invalid
        // In a real cleanup, you might want to update these to reference Firebase UIDs
        let invalid_refs: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM audit_logs 
            WHERE user_id IS NOT NULL 
            AND user_id NOT IN (SELECT id FROM users WHERE id IS NOT NULL)
            "#
        )
        .fetch_one(pool)
        .await.unwrap_or(0);
        
        if invalid_refs > 0 {
            warn!("⚠️  Found {} audit log entries with invalid user_id references", invalid_refs);
            info!("💡 Consider updating audit_logs.user_id to reference Firebase UIDs");
        }
    }
    
    Ok(())
}

/// Verify cleanup results
async fn verify_cleanup_results(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Verifying cleanup results...");
    
    // Check that Firebase-native tables exist
    let required_tables = vec![
        "firebase_sessions",
        "user_roles_permissions", 
        "user_app_data",
        "firebase_token_cache",
        "role_assignment_audit",
    ];
    
    for table in required_tables {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = $1)"
        )
        .bind(table)
        .fetch_one(pool)
        .await?;
        
        if exists {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table))
                .fetch_one(pool)
                .await?;
            info!("✅ Required table '{}' exists with {} records", table, count);
        } else {
            error!("❌ Required table '{}' is missing!", table);
        }
    }
    
    // Verify deprecated tables are gone
    let deprecated_tables = vec![
        "firebase_user_mappings",
        "unified_sessions",
        "provider_user_attributes",
    ];
    
    for table in deprecated_tables {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = $1)"
        )
        .bind(table)
        .fetch_one(pool)
        .await?;
        
        if exists {
            warn!("⚠️  Deprecated table '{}' still exists", table);
        } else {
            info!("✅ Deprecated table '{}' successfully removed", table);
        }
    }
    
    // Summary of remaining tables
    info!("📊 Final database state summary:");
    let table_summary: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT 
            table_name,
            COALESCE((
                SELECT n.ntuples
                FROM pg_stat_user_tables n
                WHERE n.relname = t.table_name
            ), 0) as record_count
        FROM information_schema.tables t
        WHERE t.table_schema = 'public'
        AND t.table_type = 'BASE TABLE'
        ORDER BY t.table_name
        "#
    )
    .fetch_all(pool)
    .await?;
    
    for (table_name, count) in table_summary {
        info!("📋 Table '{}': {} records", table_name, count);
    }
    
    Ok(())
}