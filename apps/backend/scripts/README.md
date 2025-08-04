# Admin Scripts

This directory contains administrative scripts for managing user permissions and roles.

## Scripts

### `grant_full_access.sh`
General script to promote any user to SuperAdmin with full system access.

**Usage:**
```bash
./scripts/grant_full_access.sh <email> [reason]
```

**Example:**
```bash
./scripts/grant_full_access.sh user@example.com "System administrator access"
```

**What it does:**
- Promotes user to SuperAdmin role
- Grants ALL available system permissions
- Creates audit trail with reason

### `promote_jesadakorn.sh`
Specific script to promote jesadakorn.kirtnu@gmail.com to full access.

**Usage:**
```bash
./scripts/promote_jesadakorn.sh
```

**What it does:**
- Prompts for confirmation
- Promotes jesadakorn.kirtnu@gmail.com to SuperAdmin
- Grants all system permissions

### `validate-config.sh`
Configuration validation script.

## Permissions Granted

When promoting to SuperAdmin with `--super-admin` flag, the following permissions are granted:

- `READ_ALL` - Full read access to all data
- `WRITE_ALL` - Full write access to all data  
- `DELETE_ALL` - Full delete access
- `MANAGE_USERS` - User management capabilities
- `DELETE_USERS` - User deletion capabilities
- `MANAGE_SYSTEM` - System management
- `MANAGE_ADMIN` - Admin management
- `MODERATE_CONTENT` - Content moderation
- `MODERATE_USERS` - User moderation
- `WRITE_CONTENT` - Content creation
- `ACCESS_PREMIUM` - Premium feature access
- `ACCESS_PREMIUM_FEATURES` - Premium features
- `READ_PREMIUM` - Premium content read
- `READ_ADVANCED_ANALYTICS` - Advanced analytics
- `READ_ALL_DATA` - All data access
- `WRITE_USER_DATA` - User data modification
- `READ_USER_REPORTS` - User report access

## Prerequisites

1. Rust toolchain installed
2. Database connection configured
3. Environment variables set (DATABASE_URL or config values)
4. Run from `apps/backend/` directory

## Security Note

⚠️ **Warning**: These scripts grant FULL SYSTEM ACCESS. Use with extreme caution and only for trusted administrators.

## Manual Usage

You can also run the promote_admin binary directly:

```bash
cargo build --bin promote_admin --release
./target/release/promote_admin --email user@example.com --reason "Admin access" --super-admin
```