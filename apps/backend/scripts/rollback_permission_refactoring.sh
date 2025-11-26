#!/bin/bash

# ================================================================================================
# PERMISSION REFACTORING ROLLBACK SCRIPT
# ================================================================================================
# This script provides a safe rollback mechanism for the unified permissions system migration.
# It restores the system to the previous state and includes validation.
#
# Safety Features:
# - Database backup restoration
# - Migration rollback execution
# - Health check after rollback
# - Comprehensive logging
# ================================================================================================

set -euo pipefail  # Exit on error, undefined variable, or pipe fail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0;0m' # No Color

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

log_rollback() {
    echo -e "${RED}[ROLLBACK]${NC} $1"
}

# Configuration
ROLLBACK_DIR="/tmp/epsx_rollback_$(date +%Y%m%d_%H%M%S)"
HEALTH_CHECK_URL="http://localhost:8080/health"

# Find most recent backup directory
find_latest_backup() {
    local latest_backup=$(find /tmp -name "epsx_backup_*" -type d | sort -r | head -1)
    if [ -n "$latest_backup" ] && [ -d "$latest_backup" ]; then
        echo "$latest_backup"
    else
        log_error "No backup directory found"
        return 1
    fi
}

# Pre-rollback checks
pre_rollback_checks() {
    log_info "Running pre-rollback checks..."

    # Check database connectivity
    if ! timeout 30 psql "$DATABASE_URL" -c "SELECT 1;" > /dev/null 2>&1; then
        log_error "Database connection failed"
        exit 1
    fi
    log_success "✓ Database connection verified"

    # Find backup directory
    BACKUP_DIR=$(find_latest_backup)
    log_info "Using backup directory: $BACKUP_DIR"

    # Verify backup files exist
    if [ ! -f "$BACKUP_DIR/database_backup.sql" ]; then
        log_error "Database backup not found: $BACKUP_DIR/database_backup.sql"
        exit 1
    fi

    log_success "✓ Pre-rollback checks completed"
}

# Rollback database migrations
rollback_database_migrations() {
    log_rollback "Rolling back database migrations..."

    # Rollback legacy data migration
    log_rollback "  - Rolling back legacy data migration..."
    if psql "$DATABASE_URL" -f "migrations/20251118_006_migrate_legacy_permissions/down.sql" 2>/dev/null; then
        log_success "    ✅ Legacy data migration rolled back"
    else
        log_error "    ✗ Legacy data migration rollback failed"
        return 1
    fi

    # Rollback table structure changes
    log_rollback "  - Rolling back table structure changes..."
    if psql "$DATABASE_URL" -f "migrations/20251118_005_update_existing_permissions/down.sql" 2>/dev/null; then
        log_success "    ✅ Table structure changes rolled back"
    else
        log_error "    ✗ Table structure rollback failed"
        return 1
    fi

    return 0
}

# Restore database from backup
restore_database() {
    log_rollback "Restoring database from backup..."

    local start_time=$(date +%s)

    if psql "$DATABASE_URL" < "$BACKUP_DIR/database_backup.sql" 2>/dev/null; then
        local duration=$(($(date +%s) - start_time))
        log_success "✓ Database restored in ${duration}s"
        return 0
    else
        log_error "✗ Database restore failed"
        return 1
    fi
}

# Validate rollback success
validate_rollback() {
    log_info "Validating rollback success..."

    # Check that unified permissions columns no longer exist
    local columns_removed=$(psql "$DATABASE_URL" -t -c "
        SELECT COUNT(*)
        FROM information_schema.columns
        WHERE table_name = 'permissions'
        AND column_name IN ('wallet_address', 'source_type', 'source_id', 'granted_at', 'expires_at')
        AND table_schema = 'public'
    " 2>/dev/null | tr -d ' ')

    if [ "$columns_removed" -eq 0 ]; then
        log_success "✅ Unified permissions columns removed"
    else
        log_error "✗ Unified permissions columns still exist ($columns_removed)"
        return 1
    fi

    # Check that materialized view no longer exists
    local view_removed=$(psql "$DATABASE_URL" -t -c "
        SELECT COUNT(*) FROM information_schema.views
        WHERE table_name = 'wallet_permissions_view'
        AND table_schema = 'public'
    " 2>/dev/null | tr -d ' ')

    if [ "$view_removed" -eq 0 ]; then
        log_success "✅ Materialized view removed"
    else
        log_warning "⚠️ Materialized view still exists (may require manual cleanup)"
    fi

    # Verify database connectivity
    if timeout 30 psql "$DATABASE_URL" -c "SELECT COUNT(*) FROM permissions LIMIT 1;" > /dev/null 2>&1; then
        log_success "✓ Database connectivity verified"
    else
        log_error "✗ Database connectivity failed after rollback"
        return 1
    fi

    return 0
}

# Health check after rollback
post_rollback_health_check() {
    log_info "Running post-rollback health check..."

    # Wait a moment for services to stabilize
    sleep 5

    if curl -f -s "$HEALTH_CHECK_URL" > /dev/null; then
        local health_status=$(curl -s "$HEALTH_CHECK_URL" | jq -r '.status // "unknown"' 2>/dev/null || echo "unknown")
        if [ "$health_status" = "healthy" ]; then
            log_success "✅ Backend service healthy after rollback"
        else
            log_warning "⚠️ Backend service status: $health_status"
        fi
    else
        log_warning "⚠️ Backend service not responding"
    fi

    return 0
}

# Create rollback backup directory for safety
create_rollback_backup() {
    mkdir -p "$ROLLBACK_DIR"

    # Backup current state for safety
    log_info "Creating rollback backup in: $ROLLBACK_DIR"

    # Backup database state
    local db_backup="$ROLLBACK_DIR/post_rollback_backup.sql"
    if pg_dump "$DATABASE_URL" > "$db_backup" 2>/dev/null; then
        log_success "✓ Created rollback database backup"
    else
        log_warning "⚠️ Could not create rollback database backup"
    fi

    # Backup application state
    local app_backup="$ROLLBACK_DIR/application_state.tar.gz"
    if tar -czf "$app_backup" --exclude='node_modules' --exclude='target' . > /dev/null 2>&1; then
        log_success "✓ Created rollback application backup"
    else
        log_warning "⚠️ Could not create rollback application backup"
    fi
}

# Main rollback function
main() {
    log_rollback "=== PERMISSION REFACTORING ROLLBACK ==="
    log_rollback "Rollback Directory: $ROLLBACK_DIR"
    log_rollback "Timestamp: $(date)"

    local phases=(
        "pre_rollback_checks"
        "create_rollback_backup"
        "rollback_database_migrations"
        "restore_database"
        "validate_rollback"
        "post_rollback_health_check"
    )

    local phase_num=1
    local total_phases=${#phases[@]}

    for phase in "${phases[@]}"; do
        log_info "Phase $phase_num/$total_phases: $phase"

        case $phase in
            "pre_rollback_checks")
                if ! pre_rollback_checks; then
                    log_error "Rollback failed at pre-checks phase"
                    exit 1
                fi
                ;;
            "create_rollback_backup")
                if ! create_rollback_backup; then
                    log_warning "Rollback backup failed, continuing..."
                fi
                ;;
            "rollback_database_migrations")
                if ! rollback_database_migrations; then
                    log_error "Database migration rollback failed"
                    exit 1
                fi
                ;;
            "restore_database")
                if ! restore_database; then
                    log_error "Database restore failed"
                    exit 1
                fi
                ;;
            "validate_rollback")
                if ! validate_rollback; then
                    log_error "Rollback validation failed"
                    exit 1
                fi
                ;;
            "post_rollback_health_check")
                if ! post_rollback_health_check; then
                    log_warning "Health check failed after rollback"
                    exit 1
                fi
                ;;
        esac

        ((phase_num++))
    done

    # Rollback completed successfully
    log_success "=== ROLLBACK COMPLETED SUCCESSFULLY ==="
    log_info "Original Backup Directory: $BACKUP_DIR"
    log_info("Rollback Directory: $ROLLBACK_DIR")

    # Recovery instructions
    echo ""
    log_info "=== RECOVERY INSTRUCTIONS ==="
    echo "1. Verify system is working in previous state"
    echo "2. Check application logs for any issues"
    echo "3. Update configuration if needed"
    echo "4. Consider fixing any issues that caused the rollback"
    echo "5. Plan a corrected deployment"
    echo ""
    log_info "Original files preserved in:"
    echo "  - Database backup: $BACKUP_DIR/database_backup.sql"
    echo "  - Application backup: $ROLLBACK_DIR/application_state.tar.gz"

    exit 0
}

# Trap for cleanup and error handling
trap 'log_error "Rollback interrupted"; exit 1' INT TERM

# Execute rollback
main