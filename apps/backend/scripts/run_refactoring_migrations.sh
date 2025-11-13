#!/usr/bin/env bash

# ============================================================================
# Database Refactoring Migration Runner
# ============================================================================
#
# This script runs all database refactoring migrations in sequence with
# proper error handling, backups, and verification.
#
# Usage:
#   ./scripts/run_refactoring_migrations.sh [--phase N] [--skip-backup] [--dry-run]
#
# Options:
#   --phase N        Run only specific phase (1-5)
#   --skip-backup    Skip database backup (NOT RECOMMENDED)
#   --dry-run        Show what would be executed without running
#   --help           Show this help message
#
# ============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(dirname "$SCRIPT_DIR")"
MIGRATIONS_DIR="$BACKEND_DIR/migrations"

# Configuration
SPECIFIC_PHASE=""
SKIP_BACKUP=false
DRY_RUN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --phase)
            SPECIFIC_PHASE="$2"
            shift 2
            ;;
        --skip-backup)
            SKIP_BACKUP=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --help)
            grep '^#' "$0" | grep -v '#!/bin/bash' | sed 's/^# //'
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Check for required environment variable
if [[ -z "${DATABASE_URL:-}" ]]; then
    echo -e "${RED}ERROR: DATABASE_URL environment variable is not set${NC}"
    echo "Please set DATABASE_URL in your .env file or environment"
    exit 1
fi

# Functions
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

# Extract database name from connection string
get_db_name() {
    echo "$DATABASE_URL" | sed -n 's/.*\/\([^?]*\).*/\1/p'
}

# Create database backup
create_backup() {
    local db_name
    db_name=$(get_db_name)
    local backup_file="$BACKEND_DIR/backups/db_backup_$(date +%Y%m%d_%H%M%S).sql"

    log_info "Creating database backup..."

    # Create backups directory if it doesn't exist
    mkdir -p "$BACKEND_DIR/backups"

    # Create backup using pg_dump
    if pg_dump "$DATABASE_URL" > "$backup_file"; then
        log_success "Backup created: $backup_file"
        echo "$backup_file"
    else
        log_error "Failed to create backup"
        exit 1
    fi
}

# Run SQL migration file
run_migration() {
    local migration_file=$1
    local phase_name=$2

    log_info "Running $phase_name..."
    log_info "Migration file: $migration_file"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_warning "DRY RUN: Would execute $migration_file"
        return 0
    fi

    # Run migration
    if psql "$DATABASE_URL" -f "$migration_file"; then
        log_success "$phase_name completed successfully"
        return 0
    else
        log_error "$phase_name failed"
        return 1
    fi
}

# Verify migration
verify_migration() {
    local phase=$1

    log_info "Verifying phase $phase..."

    case $phase in
        1)
            # Verify schema normalization
            psql "$DATABASE_URL" -t -c "
                SELECT COUNT(*) FROM information_schema.columns
                WHERE table_name = 'wallet_users' AND column_name = 'permission_groups';
            " | grep -q "0" && log_success "Phase 1 verification passed" || log_error "Phase 1 verification failed"
            ;;
        2)
            # Verify index optimization
            psql "$DATABASE_URL" -t -c "
                SELECT COUNT(*) FROM get_unused_indexes(1);
            " > /dev/null && log_success "Phase 2 verification passed" || log_warning "Phase 2 verification warning"
            ;;
        3)
            # Verify partitioning
            psql "$DATABASE_URL" -t -c "
                SELECT COUNT(*) FROM pg_class WHERE relname LIKE 'permission_audit_log_%';
            " | grep -qv "0" && log_success "Phase 3 verification passed" || log_error "Phase 3 verification failed"
            ;;
        4)
            # Verify read model optimization
            psql "$DATABASE_URL" -t -c "
                SELECT COUNT(*) FROM user_effective_permissions LIMIT 1;
            " > /dev/null && log_success "Phase 4 verification passed" || log_error "Phase 4 verification failed"
            ;;
        5)
            # Verify saga pattern
            psql "$DATABASE_URL" -t -c "
                SELECT COUNT(*) FROM saga_instances LIMIT 1;
            " > /dev/null && log_success "Phase 5 verification passed" || log_error "Phase 5 verification failed"
            ;;
    esac
}

# Main execution
main() {
    echo "============================================================================"
    echo "  EPSX Database Refactoring Migration Runner"
    echo "============================================================================"
    echo ""

    # Show configuration
    log_info "Configuration:"
    echo "  Database: $(get_db_name)"
    echo "  Migrations dir: $MIGRATIONS_DIR"
    echo "  Skip backup: $SKIP_BACKUP"
    echo "  Dry run: $DRY_RUN"
    [[ -n "$SPECIFIC_PHASE" ]] && echo "  Specific phase: $SPECIFIC_PHASE"
    echo ""

    # Create backup
    if [[ "$SKIP_BACKUP" == "false" ]] && [[ "$DRY_RUN" == "false" ]]; then
        backup_file=$(create_backup)
        echo ""
    else
        log_warning "Skipping backup (use with caution!)"
        echo ""
    fi

    # Migration phases
    declare -A migrations=(
        [1]="$MIGRATIONS_DIR/002_normalize_schema.sql:Phase 1: Schema Normalization"
        [2]="$MIGRATIONS_DIR/003_optimize_indexes.sql:Phase 2: Index Optimization"
        [3]="$MIGRATIONS_DIR/004_partition_tables.sql:Phase 3: Partitioning & Archival"
        [4]="$MIGRATIONS_DIR/005_read_model_optimization.sql:Phase 4: Read Model Optimization"
        [5]="$MIGRATIONS_DIR/006_saga_pattern.sql:Phase 5: Saga Pattern Implementation"
    )

    # Determine which phases to run
    if [[ -n "$SPECIFIC_PHASE" ]]; then
        phases=("$SPECIFIC_PHASE")
    else
        phases=(1 2 3 4 5)
    fi

    # Run migrations
    for phase in "${phases[@]}"; do
        migration_info="${migrations[$phase]}"
        migration_file="${migration_info%%:*}"
        phase_name="${migration_info##*:}"

        echo "============================================================================"
        echo "  $phase_name"
        echo "============================================================================"
        echo ""

        # Check if migration file exists
        if [[ ! -f "$migration_file" ]]; then
            log_error "Migration file not found: $migration_file"
            exit 1
        fi

        # Run migration
        if run_migration "$migration_file" "$phase_name"; then
            echo ""
            verify_migration "$phase"
            echo ""
        else
            log_error "Migration failed at $phase_name"

            if [[ -n "${backup_file:-}" ]]; then
                log_warning "You can restore from backup: $backup_file"
                echo "Restore command: psql \$DATABASE_URL < $backup_file"
            fi

            exit 1
        fi
    done

    # Success message
    echo "============================================================================"
    echo -e "  ${GREEN}✅ All migrations completed successfully!${NC}"
    echo "============================================================================"
    echo ""

    log_success "Database refactoring complete!"
    echo ""
    echo "Next steps:"
    echo "  1. Verify data integrity"
    echo "  2. Run application tests"
    echo "  3. Monitor query performance"
    echo "  4. Update backend code to use new views/functions"
    echo ""

    if [[ -n "${backup_file:-}" ]]; then
        echo "Backup saved at: $backup_file"
        echo "Keep this backup for at least 30 days"
        echo ""
    fi

    echo "Useful monitoring commands:"
    echo "  SELECT * FROM get_index_usage_stats();"
    echo "  SELECT * FROM v_partition_info;"
    echo "  SELECT * FROM v_mv_refresh_performance;"
    echo "  SELECT * FROM check_partition_health();"
    echo ""
}

# Run main function
main
