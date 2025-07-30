# Admin Scripts

This directory contains administrative scripts for managing users and permissions in the EPSX system.

## Scripts

### 1. promote-user-admin.js

Multi-purpose script for promoting users to SuperAdmin role with various access levels and assigning IAM/ACL permission profiles.

#### Usage

```bash
# Promote user to SuperAdmin (standard permissions)
node scripts/promote-user-admin.js promote user@example.com
node scripts/promote-user-admin.js promote user@example.com "Emergency access needed"

# Promote user to SuperAdmin with ALL ACCESS (recommended for full system access)
node scripts/promote-user-admin.js super-admin user@example.com
node scripts/promote-user-admin.js super-admin user@example.com "Full system access granted"

# Assign permission profile
node scripts/promote-user-admin.js assign user@example.com user-premium-002
node scripts/promote-user-admin.js assign user@example.com mod-standard-003 --reason "Moderator promotion"

# Legacy usage (backward compatible)
node scripts/promote-user-admin.js user@example.com
```


## Package.json Scripts

For convenience, you can use the following npm/pnpm scripts:

```bash
# Promote user to admin (standard)
pnpm promote-admin user@example.com "Emergency access"

# Promote user to super admin with all access
pnpm super-admin user@example.com "Full system access granted"

# Assign IAM profile using the main script
pnpm promote-admin assign user@example.com user-premium-002 --reason "Upgrade"
```

## Backend Binaries

The scripts invoke Rust binaries in the backend:

### promote_admin
- Located: `apps/backend/src/bin/promote_admin.rs`
- Purpose: Promote users to SuperAdmin role with configurable permission levels
- Usage: 
  - Standard: `cargo run --bin promote_admin -- --email="user@example.com" --reason="Emergency"`
  - Full Access: `cargo run --bin promote_admin -- --email="user@example.com" --reason="Full access" --super-admin`

### assign_iam
- Located: `apps/backend/src/bin/assign_iam.rs`
- Purpose: Assign IAM/ACL permission profiles to users
- Usage: `cargo run --bin assign_iam -- --email="user@example.com" --profile_id="user-premium-002"`

## Available Permission Profiles

Current system supports these permission profiles:

- **user-basic-001**: Basic user permissions (Bronze tier)
- **user-premium-002**: Premium user permissions (Silver tier)
- **moderator-standard-003**: Standard moderator permissions (Gold tier)
- **admin-full-004**: Full administrative permissions (Platinum tier)

## Options

### Common Options
- `--reason`: Reason for the assignment/promotion
- `--admin-id`: Admin user ID performing the action

### IAM Assignment Options
- `--merge-permissions`: Whether to merge with existing permissions (default: true)
- `--expires-at`: Expiration date in ISO 8601 format (e.g., 2024-12-31T23:59:59Z)

## Examples

### Promote User to SuperAdmin
```bash
# Using script directly (standard permissions)
node scripts/promote-user-admin.js promote admin@company.com "New admin setup"

# Using script directly (ALL ACCESS - recommended for full system control)
node scripts/promote-user-admin.js super-admin admin@company.com "Full system access granted"

# Using package script (standard)
pnpm promote-admin admin@company.com "New admin setup"

# Using package script (super admin with all access)
pnpm super-admin admin@company.com "Full system access granted"
```

### Assign Premium User Profile
```bash
# Using main script
node scripts/promote-user-admin.js assign user@example.com user-premium-002 --reason "Upgrade to premium plan"
```

### Assign Temporary Moderator Access
```bash
node scripts/promote-user-admin.js assign moderator@example.com mod-standard-003 \
  --reason "Temporary moderator access" \
  --expires-at "2024-12-31T23:59:59Z"
```

## Prerequisites

1. Backend must be built and available at `apps/backend/`
2. Database connection must be configured
3. `cargo` must be installed for Rust compilation
4. Node.js 18+ for running JavaScript scripts

## Error Handling

The scripts include comprehensive error handling for:
- Invalid email formats
- Missing backend binaries
- Database connection issues
- Invalid permission profile IDs
- Permission denied scenarios

## Cross-Platform Support

All scripts support Windows, macOS, and Linux with appropriate shell command generation.

## Security Notes

- These scripts bypass normal permission checks and payment requirements
- Use only for administrative purposes
- Always provide a reason for audit trails
- Assignments are logged in the level history system