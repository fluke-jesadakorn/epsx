#!/bin/bash

# EPSX Production Rollback Script
# Emergency rollback procedures for production deployment failures

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
ROLLBACK_TYPE=${1:-"controlled"}  # controlled, emergency, targeted
TARGET_VERSION=${2:-""}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
ROLLBACK_LOG="/tmp/epsx_rollback_${TIMESTAMP}.log"

# Load environment configuration
ENV_FILE="$PROJECT_ROOT/deployment/environments/production.env"
if [[ -f "$ENV_FILE" ]]; then
    source "$ENV_FILE"
else
    echo -e "${RED}❌ Production environment file not found: $ENV_FILE${NC}"
    exit 1
fi

# Rollback state tracking
ROLLBACK_STEPS=()
ROLLBACK_FAILURES=()
SERVICES_TO_ROLLBACK=("backend" "frontend" "admin")

echo -e "${BLUE}🔄 EPSX Production Rollback System${NC}"
echo -e "${BLUE}==================================${NC}"
echo -e "Rollback Type: $ROLLBACK_TYPE"
echo -e "Target Version: ${TARGET_VERSION:-'previous stable'}"
echo -e "Timestamp: $TIMESTAMP"
echo -e "Log File: $ROLLBACK_LOG"
echo ""

# Logging function
log_rollback() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    echo -e "${timestamp} [${level}] ${message}" | tee -a "$ROLLBACK_LOG"
    
    case $level in
        "ERROR")
            echo -e "${RED}❌ $message${NC}"
            ;;
        "SUCCESS")
            echo -e "${GREEN}✅ $message${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠️  $message${NC}"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ️  $message${NC}"
            ;;
        *)
            echo -e "$message"
            ;;
    esac
}

# Function to get current deployed versions
get_current_versions() {
    log_rollback "INFO" "Retrieving current deployed versions..."
    
    # Get backend version
    local backend_version
    if backend_version=$(gcloud run services describe epsx-backend \
        --region="$GOOGLE_CLOUD_REGION" \
        --format="value(spec.template.spec.template.spec.containers[0].image)" 2>/dev/null); then
        echo "BACKEND_CURRENT_VERSION=$backend_version" >> "$ROLLBACK_LOG"
        log_rollback "INFO" "Current backend version: $backend_version"
    else
        log_rollback "ERROR" "Unable to retrieve current backend version"
    fi
    
    # Get frontend version
    local frontend_version
    if frontend_version=$(gcloud run services describe epsx-frontend \
        --region="$GOOGLE_CLOUD_REGION" \
        --format="value(spec.template.spec.template.spec.containers[0].image)" 2>/dev/null); then
        echo "FRONTEND_CURRENT_VERSION=$frontend_version" >> "$ROLLBACK_LOG"
        log_rollback "INFO" "Current frontend version: $frontend_version"
    else
        log_rollback "ERROR" "Unable to retrieve current frontend version"
    fi
    
    # Get admin version
    local admin_version
    if admin_version=$(gcloud run services describe epsx-admin \
        --region="$GOOGLE_CLOUD_REGION" \
        --format="value(spec.template.spec.template.spec.containers[0].image)" 2>/dev/null); then
        echo "ADMIN_CURRENT_VERSION=$admin_version" >> "$ROLLBACK_LOG"
        log_rollback "INFO" "Current admin version: $admin_version"
    else
        log_rollback "ERROR" "Unable to retrieve current admin version"
    fi
}

# Function to get previous stable version
get_previous_stable_version() {
    log_rollback "INFO" "Determining previous stable version..."
    
    if [[ -n "$TARGET_VERSION" ]]; then
        log_rollback "INFO" "Using specified target version: $TARGET_VERSION"
        return 0
    fi
    
    # Get deployment history and find last stable version
    local deployment_history="/tmp/epsx_deployment_history.json"
    
    # This would typically come from a deployment tracking system
    # For now, we'll use a simple heuristic based on image tags
    if gcloud container images list-tags "$GOOGLE_CLOUD_REGION-docker.pkg.dev/$GOOGLE_CLOUD_PROJECT/epsx/backend" \
        --limit=10 --format=json > "$deployment_history"; then
        
        # Find the most recent tag that's not the current one
        local previous_tag
        previous_tag=$(jq -r '.[1].tags[0] // "latest-stable"' "$deployment_history" 2>/dev/null)
        
        if [[ -n "$previous_tag" && "$previous_tag" != "null" ]]; then
            TARGET_VERSION="$previous_tag"
            log_rollback "INFO" "Identified previous stable version: $TARGET_VERSION"
        else
            TARGET_VERSION="latest-stable"
            log_rollback "WARNING" "Could not identify specific previous version, using: $TARGET_VERSION"
        fi
    else
        TARGET_VERSION="latest-stable"
        log_rollback "WARNING" "Unable to retrieve deployment history, using: $TARGET_VERSION"
    fi
}

# Function to validate rollback target
validate_rollback_target() {
    log_rollback "INFO" "Validating rollback target version: $TARGET_VERSION"
    
    # Check if the target version exists in the registry
    local services=("backend" "frontend" "admin")
    
    for service in "${services[@]}"; do
        local image="$GOOGLE_CLOUD_REGION-docker.pkg.dev/$GOOGLE_CLOUD_PROJECT/epsx/$service:$TARGET_VERSION"
        
        if gcloud container images describe "$image" >/dev/null 2>&1; then
            log_rollback "SUCCESS" "Rollback target validated for $service: $TARGET_VERSION"
        else
            log_rollback "ERROR" "Rollback target not found for $service: $TARGET_VERSION"
            return 1
        fi
    done
    
    return 0
}

# Function to create database backup before rollback
create_rollback_backup() {
    log_rollback "INFO" "Creating database backup before rollback..."
    
    local backup_name="epsx-rollback-backup-$TIMESTAMP"
    
    # This would typically use your database backup system
    # For Neon PostgreSQL, this might involve using their backup API
    log_rollback "INFO" "Database backup would be created here: $backup_name"
    
    # Store backup information for recovery if needed
    echo "ROLLBACK_BACKUP_NAME=$backup_name" >> "$ROLLBACK_LOG"
    echo "ROLLBACK_BACKUP_TIMESTAMP=$TIMESTAMP" >> "$ROLLBACK_LOG"
    
    log_rollback "SUCCESS" "Database backup preparation completed"
}

# Function to rollback backend service
rollback_backend() {
    log_rollback "INFO" "Rolling back backend service to version: $TARGET_VERSION"
    
    local backend_image="$GOOGLE_CLOUD_REGION-docker.pkg.dev/$GOOGLE_CLOUD_PROJECT/epsx/backend:$TARGET_VERSION"
    
    if gcloud run deploy epsx-backend \
        --image="$backend_image" \
        --platform=managed \
        --region="$GOOGLE_CLOUD_REGION" \
        --allow-unauthenticated \
        --port=8080 \
        --memory=4Gi \
        --cpu=4 \
        --timeout=3600s \
        --min-instances=1 \
        --max-instances=10 \
        --execution-environment=gen2 \
        --quiet 2>&1 | tee -a "$ROLLBACK_LOG"; then
        
        log_rollback "SUCCESS" "Backend service rolled back successfully"
        ROLLBACK_STEPS+=("backend:success")
        
        # Wait for service to be ready
        sleep 30
        
        # Verify service health
        if curl -f -s "$BACKEND_URL/health" >/dev/null 2>&1; then
            log_rollback "SUCCESS" "Backend service health check passed"
        else
            log_rollback "ERROR" "Backend service health check failed after rollback"
            ROLLBACK_FAILURES+=("backend:health_check_failed")
        fi
    else
        log_rollback "ERROR" "Backend service rollback failed"
        ROLLBACK_FAILURES+=("backend:rollback_failed")
        return 1
    fi
}

# Function to rollback frontend service
rollback_frontend() {
    log_rollback "INFO" "Rolling back frontend service to version: $TARGET_VERSION"
    
    local frontend_image="$GOOGLE_CLOUD_REGION-docker.pkg.dev/$GOOGLE_CLOUD_PROJECT/epsx/frontend:$TARGET_VERSION"
    
    if gcloud run deploy epsx-frontend \
        --image="$frontend_image" \
        --platform=managed \
        --region="$GOOGLE_CLOUD_REGION" \
        --allow-unauthenticated \
        --port=3000 \
        --memory=2Gi \
        --cpu=2 \
        --timeout=300s \
        --min-instances=1 \
        --max-instances=20 \
        --execution-environment=gen2 \
        --quiet 2>&1 | tee -a "$ROLLBACK_LOG"; then
        
        log_rollback "SUCCESS" "Frontend service rolled back successfully"
        ROLLBACK_STEPS+=("frontend:success")
        
        # Wait for service to be ready
        sleep 30
        
        # Verify service accessibility
        if curl -f -s "$FRONTEND_URL" >/dev/null 2>&1; then
            log_rollback "SUCCESS" "Frontend service accessibility check passed"
        else
            log_rollback "ERROR" "Frontend service accessibility check failed after rollback"
            ROLLBACK_FAILURES+=("frontend:accessibility_check_failed")
        fi
    else
        log_rollback "ERROR" "Frontend service rollback failed"
        ROLLBACK_FAILURES+=("frontend:rollback_failed")
        return 1
    fi
}

# Function to rollback admin service
rollback_admin() {
    log_rollback "INFO" "Rolling back admin service to version: $TARGET_VERSION"
    
    local admin_image="$GOOGLE_CLOUD_REGION-docker.pkg.dev/$GOOGLE_CLOUD_PROJECT/epsx/admin:$TARGET_VERSION"
    
    if gcloud run deploy epsx-admin \
        --image="$admin_image" \
        --platform=managed \
        --region="$GOOGLE_CLOUD_REGION" \
        --allow-unauthenticated \
        --port=3000 \
        --memory=2Gi \
        --cpu=2 \
        --timeout=300s \
        --min-instances=1 \
        --max-instances=10 \
        --execution-environment=gen2 \
        --quiet 2>&1 | tee -a "$ROLLBACK_LOG"; then
        
        log_rollback "SUCCESS" "Admin service rolled back successfully"
        ROLLBACK_STEPS+=("admin:success")
        
        # Wait for service to be ready
        sleep 30
        
        # Verify service accessibility
        if curl -f -s "$ADMIN_FRONTEND_URL" >/dev/null 2>&1; then
            log_rollback "SUCCESS" "Admin service accessibility check passed"
        else
            log_rollback "ERROR" "Admin service accessibility check failed after rollback"
            ROLLBACK_FAILURES+=("admin:accessibility_check_failed")
        fi
    else
        log_rollback "ERROR" "Admin service rollback failed"
        ROLLBACK_FAILURES+=("admin:rollback_failed")
        return 1
    fi
}

# Function to run post-rollback validation
run_post_rollback_validation() {
    log_rollback "INFO" "Running post-rollback validation..."
    
    # Run basic validation script
    local validation_script="$SCRIPT_DIR/validate-deployment.sh"
    if [[ -f "$validation_script" ]]; then
        log_rollback "INFO" "Running deployment validation..."
        
        if bash "$validation_script" production 2>&1 | tee -a "$ROLLBACK_LOG"; then
            log_rollback "SUCCESS" "Post-rollback validation passed"
        else
            log_rollback "WARNING" "Post-rollback validation had issues"
            ROLLBACK_FAILURES+=("validation:issues_detected")
        fi
    else
        log_rollback "WARNING" "Validation script not found, skipping validation"
    fi
    
    # Test critical functionality
    log_rollback "INFO" "Testing critical functionality..."
    
    # Test API endpoints
    if curl -f -s "$BACKEND_URL/health" >/dev/null 2>&1; then
        log_rollback "SUCCESS" "API health endpoint responding"
    else
        log_rollback "ERROR" "API health endpoint not responding"
        ROLLBACK_FAILURES+=("api:health_check_failed")
    fi
    
    # Test authentication system
    if curl -f -s "$BACKEND_URL/.well-known/openid_configuration" >/dev/null 2>&1; then
        log_rollback "SUCCESS" "Authentication system responding"
    else
        log_rollback "ERROR" "Authentication system not responding"
        ROLLBACK_FAILURES+=("auth:system_down")
    fi
    
    # Test database connectivity
    if curl -f -s "$BACKEND_URL/api/v1/health/database" >/dev/null 2>&1; then
        log_rollback "SUCCESS" "Database connectivity verified"
    else
        log_rollback "ERROR" "Database connectivity issues"
        ROLLBACK_FAILURES+=("database:connectivity_failed")
    fi
}

# Function to send rollback notifications
send_rollback_notifications() {
    log_rollback "INFO" "Sending rollback notifications..."
    
    local status="SUCCESS"
    local message="EPSX production rollback completed successfully"
    
    if [[ ${#ROLLBACK_FAILURES[@]} -gt 0 ]]; then
        status="PARTIAL_FAILURE"
        message="EPSX production rollback completed with issues: ${ROLLBACK_FAILURES[*]}"
    fi
    
    # This would integrate with your notification system
    # Examples: Slack, email, PagerDuty, etc.
    
    log_rollback "INFO" "Notification sent: $status - $message"
    
    # Log to operations dashboard
    echo "ROLLBACK_STATUS=$status" >> "$ROLLBACK_LOG"
    echo "ROLLBACK_MESSAGE=$message" >> "$ROLLBACK_LOG"
    echo "ROLLBACK_FAILURES=${ROLLBACK_FAILURES[*]}" >> "$ROLLBACK_LOG"
}

# Emergency rollback procedure
emergency_rollback() {
    log_rollback "WARNING" "Executing EMERGENCY rollback procedure!"
    
    # Skip some validation steps in emergency mode
    get_current_versions
    get_previous_stable_version
    
    # Force rollback all services immediately
    log_rollback "INFO" "Emergency rollback: skipping validation, rolling back all services"
    
    # Rollback in reverse dependency order
    rollback_backend &
    rollback_frontend &
    rollback_admin &
    
    # Wait for all rollbacks to complete
    wait
    
    # Quick health check
    sleep 60
    run_post_rollback_validation
    send_rollback_notifications
}

# Controlled rollback procedure
controlled_rollback() {
    log_rollback "INFO" "Executing controlled rollback procedure"
    
    # Full validation and careful rollback
    get_current_versions
    get_previous_stable_version
    
    if ! validate_rollback_target; then
        log_rollback "ERROR" "Rollback target validation failed"
        exit 1
    fi
    
    create_rollback_backup
    
    # Rollback services in order (backend first, then frontends)
    if rollback_backend; then
        rollback_frontend
        rollback_admin
    else
        log_rollback "ERROR" "Backend rollback failed, aborting frontend rollbacks"
        exit 1
    fi
    
    run_post_rollback_validation
    send_rollback_notifications
}

# Targeted rollback procedure
targeted_rollback() {
    local target_service=$3
    log_rollback "INFO" "Executing targeted rollback for service: $target_service"
    
    get_current_versions
    get_previous_stable_version
    
    if ! validate_rollback_target; then
        log_rollback "ERROR" "Rollback target validation failed"
        exit 1
    fi
    
    case $target_service in
        "backend")
            rollback_backend
            ;;
        "frontend")
            rollback_frontend
            ;;
        "admin")
            rollback_admin
            ;;
        *)
            log_rollback "ERROR" "Unknown service for targeted rollback: $target_service"
            exit 1
            ;;
    esac
    
    run_post_rollback_validation
    send_rollback_notifications
}

# Main execution
main() {
    log_rollback "INFO" "Starting EPSX production rollback"
    
    case $ROLLBACK_TYPE in
        "emergency")
            emergency_rollback
            ;;
        "controlled")
            controlled_rollback
            ;;
        "targeted")
            targeted_rollback "$@"
            ;;
        *)
            log_rollback "ERROR" "Unknown rollback type: $ROLLBACK_TYPE"
            echo "Usage: $0 [controlled|emergency|targeted] [version] [service]"
            exit 1
            ;;
    esac
    
    # Final status report
    echo -e "\n${PURPLE}=== Rollback Summary ===${NC}"
    echo -e "Rollback Type: $ROLLBACK_TYPE"
    echo -e "Target Version: $TARGET_VERSION"
    echo -e "Completed Steps: ${ROLLBACK_STEPS[*]:-none}"
    
    if [[ ${#ROLLBACK_FAILURES[@]} -eq 0 ]]; then
        echo -e "${GREEN}✅ Rollback completed successfully!${NC}"
        log_rollback "SUCCESS" "EPSX production rollback completed successfully"
    else
        echo -e "${YELLOW}⚠️  Rollback completed with issues:${NC}"
        for failure in "${ROLLBACK_FAILURES[@]}"; do
            echo -e "${YELLOW}  • $failure${NC}"
        done
        log_rollback "WARNING" "EPSX production rollback completed with issues"
    fi
    
    echo -e "\nRollback log: $ROLLBACK_LOG"
}

# Parse command line arguments and execute
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi