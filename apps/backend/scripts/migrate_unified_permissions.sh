#!/bin/bash

# ================================================================================================
# UNIFIED PERMISSIONS MIGRATION EXECUTION SCRIPT
# ================================================================================================
# This script executes the complete database migration for the unified permissions system.
# It runs migrations in the correct order and provides comprehensive validation.
#
# Migrations executed:
# 1. Update existing permissions table structure
# 2. Migrate data from legacy tables
# 3. Validate and verify migration success
#
# Safety features:
# - Database backup before migration
# - Transaction-based execution
# - Comprehensive validation
# - Rollback capability
# ================================================================================================

set -euo pipefail  # Exit on error, undefined variable, or pipe failure

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if DATABASE_URL is set
if [ -z "${DATABASE_URL:-}" ]; then
    log_error "DATABASE_URL environment variable is not set"
    echo "Please set DATABASE_URL before running this script"
    exit 1
fi

log_info "Starting Unified Permissions Migration"
log_info "Database: $(echo $DATABASE_URL | sed 's/.*@//' | sed 's/\/.*//' | sed 's/:.*/:****/')"

# Create backup directory
BACKUP_DIR="backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

log_info "Created backup directory: $BACKUP_DIR"

# Function to check if psql is available
check_psql() {
    if ! command -v psql &> /dev/null; then
        log_error "psql is not installed or not in PATH"
        exit 1
    fi
}

# Function to execute SQL with error handling
execute_sql() {
    local sql_file="$1"
    local description="$2"

    log_info "Executing: $description"
    log_info "SQL file: $sql_file"

    if psql "$DATABASE_URL" -f "$sql_file" > "$BACKUP_DIR/${description// /_}.log" 2>&1; then
        log_success "✓ $description"
        return 0
    else
        log_error "✗ $description"
        echo "Check log file: $BACKUP_DIR/${description// /_}.log"
        echo "Last few lines of error:"
        tail -10 "$BACKUP_DIR/${description// /_}.log"
        return 1
    fi
}

# Function to validate migration
validate_migration() {
    log_info "Validating migration..."

    # Check if new columns exist
    local columns_added=$(psql "$DATABASE_URL" -t -c "
        SELECT COUNT(*)
        FROM information_schema.columns
        WHERE table_name = 'permissions'
        AND column_name IN ('wallet_address', 'source_type', 'source_id', 'granted_at', 'expires_at', 'granted_by', 'grant_reason')
    " 2>/dev/null | tr -d ' ')

    if [ "$columns_added" -eq 7 ]; then
        log_success "✓ All new columns added to permissions table"
    else
        log_error "✗ Missing columns in permissions table (found: $columns_added/7)"
        return 1
    fi

    # Check if materialized view exists
    local view_exists=$(psql "$DATABASE_URL" -t -c "
        SELECT COUNT(*)
        FROM information_schema.views
        WHERE table_name = 'wallet_permissions_view'
    " 2>/dev/null | tr -d ' ')

    if [ "$view_exists" -eq 1 ]; then
        log_success "✓ Materialized view created"
    else
        log_error "✗ Materialized view not found"
        return 1
    fi

    # Check data migration
    local migrated_permissions=$(psql "$DATABASE_URL" -t -c "
        SELECT COUNT(*)
        FROM permissions
        WHERE wallet_address IS NOT NULL
        AND source_type IS NOT NULL
    " 2>/dev/null | tr -d ' ')

    log_info "Migrated permissions: $migrated_permissions"

    # Check statistics
    local total_permissions=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM permissions" 2>/dev/null | tr -d ' ')
    local wallet_permissions=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM permissions WHERE wallet_address IS NOT NULL" 2>/dev/null | tr -d ' ')

    log_success "Migration Statistics:"
    echo "  - Total permissions: $total_permissions"
    echo "  - Wallet permissions: $wallet_permissions"
    echo "  - Legacy permissions: $((total_permissions - wallet_permissions))"

    return 0
}

# Function to create database backup
create_backup() {
    log_info "Creating database backup..."

    local backup_file="$BACKUP_DIR/pre_migration_backup.sql"

    if pg_dump "$DATABASE_URL" > "$backup_file"; then
        log_success "✓ Database backup created: $backup_file"
        log_info "Backup size: $(du -h "$backup_file" | cut -f1)"
    else
        log_error "✗ Failed to create database backup"
        return 1
    fi
}

# Main migration execution
main() {
    log_info "=== UNIFIED PERMISSIONS MIGRATION START ==="

    # Pre-migration checks
    log_info "Pre-migration checks..."
    check_psql
    create_backup

    # Get current timestamp for validation
    MIGRATION_START=$(date +%s)

    # Step 1: Update permissions table structure
    log_info "Step 1: Updating permissions table structure..."
    if ! execute_sql "migrations/20251118_005_update_existing_permissions/up.sql" "Update existing permissions table"; then
        log_error "Step 1 failed. Check logs and restore from backup if needed."
        exit 1
    fi

    # Step 2: Migrate legacy data
    log_info "Step 2: Migrating legacy permission data..."
    if ! execute_sql "migrations/20251118_006_migrate_legacy_permissions/up.sql" "Migrate legacy permissions data"; then
        log_error "Step 2 failed. Check logs and restore from backup if needed."
        exit 1
    fi

    # Step 3: Validate migration
    log_info "Step 3: Validating migration..."
    if ! validate_migration; then
        log_error "Migration validation failed. Please check the migration logs."
        exit 1
    fi

    # Migration completed successfully
    local MIGRATION_END=$(date +%s)
    local MIGRATION_DURATION=$((MIGRATION_END - MIGRATION_START))

    log_success "=== MIGRATION COMPLETED SUCCESSFULLY ==="
    log_info "Migration duration: ${MIGRATION_DURATION}s"
    log_info "Backup location: $BACKUP_DIR"

    # Post-migration recommendations
    echo ""
    log_info "=== POST-MIGRATION NEXT STEPS ==="
    echo "1. Update application code to use new permission structure"
    echo "2. Run comprehensive tests to validate functionality"
    echo "3. Monitor application performance for improvements"
    echo "4. Remove legacy tables after validation (optional):"
    echo "   - wallet_direct_permissions"
    echo "   - wallet_group_memberships"
    echo "   - permission_group_memberships"
    echo "5. Update monitoring and alerting for new permission system"
    echo ""
    log_info "Migration files preserved in: migrations/20251118_*/"
    log_info "Rollback script available: migrations/20251118_006_migrate_legacy_permissions/down.sql"

    return 0
}

# Execute main function
main "$@"