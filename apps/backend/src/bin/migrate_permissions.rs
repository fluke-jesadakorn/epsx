use std::collections::{HashMap, HashSet};
use diesel::prelude::*;
use uuid::Uuid;

use epsx_backend::core::db;
use epsx_backend::infrastructure::adapters::repositories::diesel::models::{
    User, UserPermission,
};

#[derive(Debug)]
struct PermissionMapping {
    permission_string: String,
    platform: String,
    resource: String,
    action: String,
    role_mapping: Option<String>,
}

#[derive(Debug)]
struct RoleTemplate {
    name: String,
    description: String,
    permissions: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting RBAC Permission Migration");
    
    let mut conn = db::establish_connection().await?;
    
    // Step 1: Analyze existing permissions
    println!("📊 Analyzing existing permissions...");
    let existing_permissions = analyze_existing_permissions(&mut conn).await?;
    
    // Step 2: Create permission mappings
    println!("🗺️  Creating permission mappings...");
    let permission_mappings = create_permission_mappings(&existing_permissions);
    
    // Step 3: Create role templates
    println!("👥 Creating role templates...");
    let role_templates = create_role_templates();
    
    // Step 4: Migrate permissions to RBAC
    println!("🔄 Migrating permissions to RBAC...");
    migrate_to_rbac(&mut conn, &permission_mappings, &role_templates).await?;
    
    // Step 5: Verify migration
    println!("✅ Verifying migration...");
    verify_migration(&mut conn).await?;
    
    println!("🎉 Migration completed successfully!");
    Ok(())
}

async fn analyze_existing_permissions(
    conn: &mut PgConnection,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use epsx_backend::infrastructure::adapters::repositories::diesel::schema::user_permissions::dsl::*;
    
    let existing: Vec<String> = user_permissions
        .select(permission)
        .distinct()
        .load(conn)?;
    
    println!("Found {} unique permission strings", existing.len());
    
    for perm in &existing {
        println!("  - {}", perm);
    }
    
    Ok(existing)
}

fn create_permission_mappings(existing_permissions: &[String]) -> Vec<PermissionMapping> {
    let mut mappings = Vec::new();
    
    for perm in existing_permissions {
        let mapping = if perm.contains(':') {
            // Already structured permission
            let parts: Vec<&str> = perm.split(':').collect();
            if parts.len() >= 3 {
                PermissionMapping {
                    permission_string: perm.clone(),
                    platform: parts[0].to_string(),
                    resource: parts[1].to_string(),
                    action: parts[2].to_string(),
                    role_mapping: determine_role_mapping(perm),
                }
            } else {
                // Malformed structured permission - map to epsx platform
                PermissionMapping {
                    permission_string: perm.clone(),
                    platform: "epsx".to_string(),
                    resource: "unknown".to_string(),
                    action: "access".to_string(),
                    role_mapping: None,
                }
            }
        } else {
            // Legacy permission - map to epsx platform
            map_legacy_permission(perm)
        };
        
        mappings.push(mapping);
    }
    
    mappings
}

fn map_legacy_permission(permission: &str) -> PermissionMapping {
    let (resource, action) = match permission {
        // Analytics permissions
        "view_eps" | "eps_view" => ("analytics", "view"),
        "export_data" | "data_export" => ("analytics", "export"),
        "advanced_filters" | "filters_advanced" => ("analytics", "advanced"),
        
        // Profile permissions
        "profile_manage" | "manage_profile" => ("profile", "manage"),
        "profile_edit" | "edit_profile" => ("profile", "edit"),
        
        // Notification permissions
        "notifications" | "receive_notifications" => ("notifications", "receive"),
        
        // Real-time permissions
        "realtime" | "realtime_access" => ("realtime", "access"),
        
        // Billing permissions
        "billing" | "billing_manage" => ("billing", "manage"),
        
        // Admin permissions
        "admin" | "admin_access" => ("*", "*"),
        "user_management" | "manage_users" => ("users", "manage"),
        "system_management" | "manage_system" => ("system", "manage"),
        
        // Default mapping
        _ => ("unknown", "access"),
    };
    
    let platform = if permission.contains("admin") || resource == "users" || resource == "system" {
        "admin"
    } else {
        "epsx"
    };
    
    PermissionMapping {
        permission_string: permission.to_string(),
        platform: platform.to_string(),
        resource: resource.to_string(),
        action: action.to_string(),
        role_mapping: determine_role_mapping(&format!("{}:{}:{}", platform, resource, action)),
    }
}

fn determine_role_mapping(permission: &str) -> Option<String> {
    match permission {
        p if p.starts_with("admin:*:*") || p == "admin:*:*" => Some("super_admin".to_string()),
        p if p.starts_with("admin:users:") => Some("user_admin".to_string()),
        p if p.starts_with("admin:system:") => Some("system_admin".to_string()),
        p if p.starts_with("epsx:analytics:") => Some("analytics_user".to_string()),
        p if p.starts_with("epsx:") => Some("epsx_user".to_string()),
        _ => None,
    }
}

fn create_role_templates() -> Vec<RoleTemplate> {
    vec![
        RoleTemplate {
            name: "super_admin".to_string(),
            description: "Super administrator with full system access".to_string(),
            permissions: vec!["admin:*:*".to_string()],
        },
        RoleTemplate {
            name: "user_admin".to_string(),
            description: "User administrator with user management capabilities".to_string(),
            permissions: vec![
                "admin:users:*".to_string(),
                "admin:audit:read".to_string(),
            ],
        },
        RoleTemplate {
            name: "system_admin".to_string(),
            description: "System administrator with system management capabilities".to_string(),
            permissions: vec![
                "admin:system:*".to_string(),
                "admin:audit:read".to_string(),
            ],
        },
        RoleTemplate {
            name: "analytics_admin".to_string(),
            description: "Analytics administrator with full analytics access".to_string(),
            permissions: vec![
                "epsx:analytics:*".to_string(),
                "epsx:realtime:access".to_string(),
            ],
        },
        RoleTemplate {
            name: "analytics_user".to_string(),
            description: "Analytics user with advanced analytics features".to_string(),
            permissions: vec![
                "epsx:analytics:view".to_string(),
                "epsx:analytics:export".to_string(),
                "epsx:analytics:advanced".to_string(),
                "epsx:realtime:access".to_string(),
                "epsx:profile:manage".to_string(),
                "epsx:notifications:receive".to_string(),
            ],
        },
        RoleTemplate {
            name: "epsx_user".to_string(),
            description: "Basic EPSX user with core platform access".to_string(),
            permissions: vec![
                "epsx:analytics:view".to_string(),
                "epsx:profile:manage".to_string(),
                "epsx:notifications:receive".to_string(),
            ],
        },
        RoleTemplate {
            name: "premium_user".to_string(),
            description: "Premium user with enhanced features".to_string(),
            permissions: vec![
                "epsx:analytics:view".to_string(),
                "epsx:analytics:export".to_string(),
                "epsx:profile:manage".to_string(),
                "epsx:notifications:receive".to_string(),
                "epsx:billing:manage".to_string(),
            ],
        },
    ]
}

async fn migrate_to_rbac(
    conn: &mut PgConnection,
    mappings: &[PermissionMapping],
    templates: &[RoleTemplate],
) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::sql_query;
    
    // Create permissions
    println!("Creating RBAC permissions...");
    for mapping in mappings {
        let permission_name = format!("{}:{}:{}", mapping.platform, mapping.resource, mapping.action);
        
        sql_query(
            "INSERT INTO rbac_permissions (id, name, platform, resource, action, description, is_system_permission, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
             ON CONFLICT (name) DO NOTHING"
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Text, _>(&permission_name)
        .bind::<diesel::sql_types::Text, _>(&mapping.platform)
        .bind::<diesel::sql_types::Text, _>(&mapping.resource)
        .bind::<diesel::sql_types::Text, _>(&mapping.action)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(&format!("Migrated from: {}", mapping.permission_string)))
        .bind::<diesel::sql_types::Bool, _>(false)
        .execute(conn)?;
    }
    
    // Create roles from templates
    println!("Creating RBAC roles...");
    for template in templates {
        let role_id = Uuid::new_v4();
        
        // Create role
        sql_query(
            "INSERT INTO rbac_roles (id, name, description, is_system_role, created_at, updated_at)
             VALUES ($1, $2, $3, $4, NOW(), NOW())
             ON CONFLICT (name) DO NOTHING"
        )
        .bind::<diesel::sql_types::Uuid, _>(role_id)
        .bind::<diesel::sql_types::Text, _>(&template.name)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(&template.description))
        .bind::<diesel::sql_types::Bool, _>(true)
        .execute(conn)?;
        
        // Assign permissions to role
        for permission_name in &template.permissions {
            sql_query(
                "INSERT INTO rbac_role_permissions (role_id, permission_id, created_at, updated_at)
                 SELECT $1, p.id, NOW(), NOW()
                 FROM rbac_permissions p
                 WHERE p.name = $2
                 ON CONFLICT (role_id, permission_id) DO NOTHING"
            )
            .bind::<diesel::sql_types::Uuid, _>(role_id)
            .bind::<diesel::sql_types::Text, _>(permission_name)
            .execute(conn)?;
        }
    }
    
    // Migrate user permissions to user roles
    println!("Migrating user permissions to roles...");
    migrate_user_permissions(conn, mappings).await?;
    
    Ok(())
}

async fn migrate_user_permissions(
    conn: &mut PgConnection,
    mappings: &[PermissionMapping],
) -> Result<(), Box<dyn std::error::Error>> {
    use epsx_backend::infrastructure::adapters::repositories::diesel::schema::user_permissions::dsl::*;
    use diesel::sql_query;
    
    // Get all user permissions
    let user_perms: Vec<(Uuid, String)> = user_permissions
        .select((user_id, permission))
        .load(conn)?;
    
    // Group by user
    let mut user_permission_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for (uid, perm) in user_perms {
        user_permission_map.entry(uid).or_default().push(perm);
    }
    
    // For each user, determine the best role assignment
    for (uid, perms) in user_permission_map {
        let best_role = determine_best_role(&perms, mappings);
        
        if let Some(role_name) = best_role {
            // Assign role to user
            sql_query(
                "INSERT INTO rbac_user_roles (user_id, role_id, created_at, updated_at)
                 SELECT $1, r.id, NOW(), NOW()
                 FROM rbac_roles r
                 WHERE r.name = $2
                 ON CONFLICT (user_id, role_id) DO NOTHING"
            )
            .bind::<diesel::sql_types::Uuid, _>(uid)
            .bind::<diesel::sql_types::Text, _>(&role_name)
            .execute(conn)?;
            
            println!("Assigned role '{}' to user {}", role_name, uid);
        }
        
        // Also migrate individual permissions that don't fit into roles
        for perm in &perms {
            let mapped_perm = get_mapped_permission(perm, mappings);
            if let Some(perm_name) = mapped_perm {
                sql_query(
                    "INSERT INTO rbac_user_permissions (user_id, permission_id, permission_type, created_at, updated_at)
                     SELECT $1, p.id, 'grant', NOW(), NOW()
                     FROM rbac_permissions p
                     WHERE p.name = $2
                     ON CONFLICT (user_id, permission_id) DO NOTHING"
                )
                .bind::<diesel::sql_types::Uuid, _>(uid)
                .bind::<diesel::sql_types::Text, _>(&perm_name)
                .execute(conn)?;
            }
        }
    }
    
    Ok(())
}

fn determine_best_role(user_permissions: &[String], mappings: &[PermissionMapping]) -> Option<String> {
    let mapped_perms: HashSet<String> = user_permissions
        .iter()
        .filter_map(|p| get_mapped_permission(p, mappings))
        .collect();
    
    // Check for admin permissions first
    if mapped_perms.contains("admin:*:*") {
        return Some("super_admin".to_string());
    }
    
    if mapped_perms.iter().any(|p| p.starts_with("admin:users:")) {
        return Some("user_admin".to_string());
    }
    
    if mapped_perms.iter().any(|p| p.starts_with("admin:system:")) {
        return Some("system_admin".to_string());
    }
    
    // Check for analytics permissions
    let analytics_perms: Vec<_> = mapped_perms.iter().filter(|p| p.starts_with("epsx:analytics:")).collect();
    if analytics_perms.len() >= 3 {
        return Some("analytics_user".to_string());
    }
    
    // Check for premium features
    if mapped_perms.contains("epsx:billing:manage") || mapped_perms.contains("epsx:analytics:export") {
        return Some("premium_user".to_string());
    }
    
    // Default to basic user if has any epsx permissions
    if mapped_perms.iter().any(|p| p.starts_with("epsx:")) {
        return Some("epsx_user".to_string());
    }
    
    None
}

fn get_mapped_permission(permission: &str, mappings: &[PermissionMapping]) -> Option<String> {
    mappings
        .iter()
        .find(|m| m.permission_string == permission)
        .map(|m| format!("{}:{}:{}", m.platform, m.resource, m.action))
}

async fn verify_migration(
    conn: &mut PgConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::sql_query;
    
    println!("📊 Migration Statistics:");
    
    // Count permissions
    let perm_count: i64 = sql_query("SELECT COUNT(*) as count FROM rbac_permissions")
        .get_result::<(i64,)>(conn)?
        .0;
    println!("  ✅ Created {} permissions", perm_count);
    
    // Count roles
    let role_count: i64 = sql_query("SELECT COUNT(*) as count FROM rbac_roles")
        .get_result::<(i64,)>(conn)?
        .0;
    println!("  ✅ Created {} roles", role_count);
    
    // Count user role assignments
    let user_role_count: i64 = sql_query("SELECT COUNT(*) as count FROM rbac_user_roles")
        .get_result::<(i64,)>(conn)?
        .0;
    println!("  ✅ Created {} user role assignments", user_role_count);
    
    // Count user permission assignments
    let user_perm_count: i64 = sql_query("SELECT COUNT(*) as count FROM rbac_user_permissions")
        .get_result::<(i64,)>(conn)?
        .0;
    println!("  ✅ Created {} user permission assignments", user_perm_count);
    
    Ok(())
}