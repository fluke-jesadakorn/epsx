# Admin Management Scripts

This directory contains scripts for managing admin users in the EPSX platform. These scripts replace the hardcoded admin logic that was previously embedded in the application code.

## Prerequisites

- PostgreSQL database must be running and accessible
- `DATABASE_URL` environment variable must be set
- Rust toolchain installed (for CLI tools)

## Scripts

### 1. Assign Specific Admin Modules

```bash
./scripts/assign-admin-modules.sh user@example.com "system_admin,user_management,analytics_access"
```

**Purpose**: Assign specific admin modules to a user for granular permission control.

**Available Modules**:
- `system_admin` - Full system administration
- `user_management` - User account management
- `analytics_access` - Analytics and reporting access
- `security_management` - Security settings and monitoring
- `audit_logs` - Access to audit logs and compliance
- `financial_oversight` - Financial data and transaction oversight
- `content_management` - Content moderation and management
- `support_access` - Customer support tools
- `database_management` - Database administration tools
- `developer_portal` - API keys and developer resources
- `module_management` - Manage admin modules and permissions

### 2. Promote to Full Admin

```bash
./scripts/promote-admin.sh admin@example.com
```

**Purpose**: Promote a user to full admin by assigning ALL available admin modules.

**Warning**: This grants complete administrative access to the platform.

### 3. Revoke Admin Access

```bash
./scripts/revoke-admin.sh user@example.com
```

**Purpose**: Remove all admin module assignments from a user.

**Warning**: This completely removes admin access from the user.

## Security Notes

1. **No Hardcoded Users**: Unlike the previous system, there are no hardcoded admin users (like `info@epsx.io`).

2. **Database-Driven**: All permissions are stored in the `user_admin_roles` table and validated against the database.

3. **Granular Control**: Instead of a single "SuperAdmin" role, use specific module assignments for principle of least privilege.

4. **Audit Trail**: All assignments include timestamps and reasons for compliance.

## Usage Examples

### Create a System Administrator
```bash
# Full admin access
./scripts/promote-admin.sh sysadmin@company.com
```

### Create a Support Manager
```bash
# Limited to user management and support
./scripts/assign-admin-modules.sh support@company.com "user_management,support_access"
```

### Create an Analytics Manager
```bash
# Analytics and reporting only
./scripts/assign-admin-modules.sh analytics@company.com "analytics_access,audit_logs"
```

### Remove Admin Access
```bash
# Revoke all admin privileges
./scripts/revoke-admin.sh former-admin@company.com
```

## Database Verification

You can verify admin assignments by querying the database:

```sql
-- Check admin modules for a user
SELECT u.email, uar.module_code, uar.granted_reason, uar.created_at
FROM users u 
JOIN user_admin_roles uar ON u.firebase_uid = uar.firebase_uid 
WHERE u.email = 'user@example.com' AND uar.is_active = true;

-- List all active admin users
SELECT u.email, COUNT(uar.module_code) as module_count, 
       STRING_AGG(uar.module_code, ', ') as modules
FROM users u 
JOIN user_admin_roles uar ON u.firebase_uid = uar.firebase_uid 
WHERE uar.is_active = true
GROUP BY u.email
ORDER BY module_count DESC;
```

## Migration from Old System

The new system replaces:

- ❌ Hardcoded `info@epsx.io` admin user
- ❌ `TEST_ADMIN_EMAIL` mock authentication
- ❌ SuperAdmin role hierarchy
- ❌ Firebase custom claims for admin access

With:

- ✅ Database-driven permission assignments
- ✅ Granular module-based permissions
- ✅ CLI tools for admin management
- ✅ Proper audit trails and compliance