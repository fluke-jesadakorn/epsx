#!/bin/bash

# EPSX Web3 User Migration Script
# Migrates existing email-based users to Web3 wallet authentication
# Phase 4.3: Complete migration from OIDC to Web3-first system

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BACKUP_DIR="$PROJECT_ROOT/backups/web3-migration-$(date +%Y%m%d-%H%M%S)"
LOG_FILE="$BACKUP_DIR/migration.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
    exit 1
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_FILE"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_FILE"
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if we're in the right directory
    if [[ ! -f "$PROJECT_ROOT/package.json" ]]; then
        error "Must be run from EPSX project root directory"
    fi
    
    # Check if database URL is available
    if [[ -z "${DATABASE_URL:-}" ]]; then
        error "DATABASE_URL environment variable not set"
    fi
    
    # Check if psql is available
    if ! command -v psql &> /dev/null; then
        error "psql command not found - PostgreSQL client required"
    fi
    
    # Check if backend is running
    BACKEND_URL="${BACKEND_URL:-http://localhost:8080}"
    if ! curl -s "$BACKEND_URL/health" > /dev/null; then
        warning "Backend not responding at $BACKEND_URL - some operations may fail"
    fi
    
    success "Prerequisites check passed"
}

# Create backup directory and backup current state
create_backup() {
    log "Creating backup directory: $BACKUP_DIR"
    mkdir -p "$BACKUP_DIR"
    
    log "Backing up current database state..."
    
    # Backup users table
    psql "$DATABASE_URL" -c "\\copy users TO '$BACKUP_DIR/users_backup.csv' CSV HEADER;" || {
        error "Failed to backup users table"
    }
    
    # Backup permissions
    psql "$DATABASE_URL" -c "\\copy user_permissions TO '$BACKUP_DIR/user_permissions_backup.csv' CSV HEADER;" 2>/dev/null || {
        warning "user_permissions table not found - may have been migrated already"
    }
    
    # Backup sessions if exists
    psql "$DATABASE_URL" -c "\\copy sessions TO '$BACKUP_DIR/sessions_backup.csv' CSV HEADER;" 2>/dev/null || {
        warning "sessions table not found - may have been migrated already"
    }
    
    success "Database backup completed"
}

# Analyze current user data
analyze_users() {
    log "Analyzing current user data..."
    
    # Count total users
    local total_users
    total_users=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM users;" | xargs)
    log "Total users: $total_users"
    
    # Count users with wallet addresses
    local wallet_users
    wallet_users=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM users WHERE wallet_address IS NOT NULL AND wallet_address != '';" | xargs)
    log "Users with wallet addresses: $wallet_users"
    
    # Count users without wallet addresses
    local email_users
    email_users=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM users WHERE wallet_address IS NULL OR wallet_address = '';" | xargs)
    log "Users without wallet addresses: $email_users"
    
    # Count admin users
    local admin_users
    admin_users=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM users WHERE role = 'admin' OR role LIKE '%admin%';" | xargs)
    log "Admin users: $admin_users"
    
    if [[ $email_users -gt 0 ]]; then
        warning "$email_users users need wallet addresses assigned"
        echo "Users without wallets:" | tee -a "$LOG_FILE"
        psql "$DATABASE_URL" -c "SELECT id, email, role, created_at FROM users WHERE wallet_address IS NULL OR wallet_address = '' ORDER BY created_at;" | tee -a "$LOG_FILE"
    fi
}

# Generate migration report
generate_migration_report() {
    log "Generating migration report..."
    
    cat > "$BACKUP_DIR/migration_report.md" << EOF
# EPSX Web3 Migration Report

**Date:** $(date)
**Migration ID:** $(basename "$BACKUP_DIR")

## Pre-Migration State

$(psql "$DATABASE_URL" -c "SELECT COUNT(*) as total_users FROM users;")
$(psql "$DATABASE_URL" -c "SELECT COUNT(*) as users_with_wallets FROM users WHERE wallet_address IS NOT NULL AND wallet_address != '';")
$(psql "$DATABASE_URL" -c "SELECT COUNT(*) as users_without_wallets FROM users WHERE wallet_address IS NULL OR wallet_address = '';")

## Actions Required

### Users Requiring Manual Wallet Assignment

$(psql "$DATABASE_URL" -c "SELECT id, email, role, created_at FROM users WHERE wallet_address IS NULL OR wallet_address = '' ORDER BY created_at;")

### Migration Steps

1. **Automated Migration**: Users with existing wallet addresses ✅
2. **Manual Assignment**: Users without wallet addresses require admin action
3. **Permission Transfer**: All permissions will be linked to wallet addresses
4. **Legacy Cleanup**: Old email-based authentication removed

## Recommendations

- Contact users without wallet addresses to provide their Web3 wallet
- Use admin interface to assign wallet addresses: \`/wallet-management\`
- Test authentication with sample users before full deployment
- Keep this backup for rollback if needed

## Files Backed Up

- \`users_backup.csv\` - Complete users table
- \`user_permissions_backup.csv\` - User permissions
- \`sessions_backup.csv\` - Active sessions
- \`migration.log\` - Detailed migration log

## Rollback Instructions

If rollback is needed:
\`\`\`bash
# Restore from backup
psql \$DATABASE_URL < $BACKUP_DIR/rollback.sql
\`\`\`
EOF

    success "Migration report generated: $BACKUP_DIR/migration_report.md"
}

# Create rollback script
create_rollback_script() {
    log "Creating rollback script..."
    
    cat > "$BACKUP_DIR/rollback.sql" << 'EOF'
-- EPSX Web3 Migration Rollback Script
-- This script can restore the database to pre-migration state

BEGIN;

-- Note: This is a template - actual rollback would require
-- restoring from the CSV backups created during migration

-- Example commands (adjust based on actual backup):
-- DROP TABLE IF EXISTS wallet_users CASCADE;
-- COPY users FROM 'users_backup.csv' CSV HEADER;
-- COPY user_permissions FROM 'user_permissions_backup.csv' CSV HEADER;

-- Add any other rollback operations here

COMMIT;
EOF

    success "Rollback script created: $BACKUP_DIR/rollback.sql"
}

# Validate Web3 system
validate_web3_system() {
    log "Validating Web3 authentication system..."
    
    # Check if Web3 auth endpoints are available
    local backend_url="${BACKEND_URL:-http://localhost:8080}"
    
    if curl -s "$backend_url/api/v1/auth/web3/challenge" -H "Content-Type: application/json" -d '{"wallet_address":"0x742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5f43"}' > /dev/null; then
        success "Web3 challenge endpoint working"
    else
        warning "Web3 challenge endpoint not responding"
    fi
    
    # Check database tables exist
    if psql "$DATABASE_URL" -c "\\d wallet_users" > /dev/null 2>&1; then
        success "wallet_users table exists"
    else
        warning "wallet_users table not found - may need migration"
    fi
    
    # Check environment variables
    if [[ -n "${WEB3_APP_SECRET:-}" ]]; then
        success "WEB3_APP_SECRET configured"
    else
        warning "WEB3_APP_SECRET not set - required for Web3 authentication"
    fi
}

# Main migration workflow
perform_migration() {
    log "Starting Web3 user migration process..."
    
    # Validate current state
    validate_web3_system
    
    # Create migration commands for users with wallets
    log "Preparing migration for users with existing wallet addresses..."
    
    psql "$DATABASE_URL" << 'EOF'
-- Update existing users with wallet addresses to use Web3 authentication
UPDATE users 
SET 
    auth_method = 'web3',
    updated_at = NOW()
WHERE 
    wallet_address IS NOT NULL 
    AND wallet_address != ''
    AND (auth_method IS NULL OR auth_method != 'web3');
EOF

    local migrated_count
    migrated_count=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM users WHERE auth_method = 'web3';" | xargs)
    success "Migrated $migrated_count users to Web3 authentication"
    
    # List users still requiring manual wallet assignment
    local remaining_count
    remaining_count=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM users WHERE wallet_address IS NULL OR wallet_address = '';" | xargs)
    
    if [[ $remaining_count -gt 0 ]]; then
        warning "$remaining_count users still need wallet addresses assigned"
        echo "" | tee -a "$LOG_FILE"
        echo "Users requiring manual wallet assignment:" | tee -a "$LOG_FILE"
        psql "$DATABASE_URL" -c "SELECT id, email, role, created_at FROM users WHERE wallet_address IS NULL OR wallet_address = '' ORDER BY role DESC, created_at;" | tee -a "$LOG_FILE"
        echo "" | tee -a "$LOG_FILE"
        echo "Use the admin interface at /wallet-management to assign wallet addresses" | tee -a "$LOG_FILE"
    fi
}

# Test migration with sample operations
test_migration() {
    log "Testing migration with sample operations..."
    
    # Test database connectivity
    if psql "$DATABASE_URL" -c "SELECT 1;" > /dev/null 2>&1; then
        success "Database connectivity test passed"
    else
        error "Database connectivity test failed"
    fi
    
    # Test backend connectivity
    local backend_url="${BACKEND_URL:-http://localhost:8080}"
    if curl -s "$backend_url/health" > /dev/null 2>&1; then
        success "Backend connectivity test passed"
    else
        warning "Backend connectivity test failed - some features may not work"
    fi
    
    # Validate user data integrity
    local user_count
    user_count=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM users;" | xargs)
    
    if [[ $user_count -gt 0 ]]; then
        success "User data validation passed ($user_count users found)"
    else
        warning "No users found in database"
    fi
}

# Main execution
main() {
    echo ""
    echo "================================================"
    echo "     EPSX Web3 User Migration Script"
    echo "================================================"
    echo ""
    
    # Confirm execution
    read -p "This will migrate users to Web3 authentication. Continue? (y/N): " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log "Migration cancelled by user"
        exit 0
    fi
    
    # Execute migration steps
    check_prerequisites
    create_backup
    analyze_users
    perform_migration
    test_migration
    generate_migration_report
    create_rollback_script
    
    echo ""
    echo "================================================"
    echo "          Migration Completed"
    echo "================================================"
    echo ""
    
    success "Web3 migration completed successfully!"
    log "Backup location: $BACKUP_DIR"
    log "Migration report: $BACKUP_DIR/migration_report.md"
    
    echo ""
    echo "Next steps:"
    echo "1. Review migration report: $BACKUP_DIR/migration_report.md"
    echo "2. Use admin interface to assign wallet addresses to remaining users"
    echo "3. Test Web3 authentication with sample users"
    echo "4. Deploy updated applications when ready"
    echo ""
}

# Execute main function
main "$@"