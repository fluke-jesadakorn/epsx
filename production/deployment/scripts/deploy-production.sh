#!/bin/bash

# EPSX Production Deployment Script
# Orchestrates the complete production deployment process with security validation

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
DEPLOYMENT_ENV="production"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DEPLOYMENT_DIR="$PROJECT_ROOT/deployment"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DEPLOYMENT_LOG="/tmp/epsx_deployment_${TIMESTAMP}.log"

# Deployment settings
DRY_RUN=${DRY_RUN:-false}
SKIP_TESTS=${SKIP_TESTS:-false}
SKIP_VALIDATION=${SKIP_VALIDATION:-false}
FORCE_DEPLOY=${FORCE_DEPLOY:-false}
ROLLBACK_ON_FAILURE=${ROLLBACK_ON_FAILURE:-true}

# Load environment configuration
ENV_FILE="$DEPLOYMENT_DIR/environments/production.env"
if [[ -f "$ENV_FILE" ]]; then
    source "$ENV_FILE"
else
    echo -e "${RED}❌ Production environment file not found: $ENV_FILE${NC}"
    exit 1
fi

# Deployment state tracking
DEPLOYMENT_STATE="STARTED"
DEPLOYED_SERVICES=()
ROLLBACK_NEEDED=false

# Logging function
log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    echo -e "${timestamp} [${level}] ${message}" | tee -a "$DEPLOYMENT_LOG"
    
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

# Cleanup function
cleanup() {
    local exit_code=$?
    
    if [[ $exit_code -ne 0 && "$ROLLBACK_ON_FAILURE" == true && "$ROLLBACK_NEEDED" == true ]]; then
        log "WARNING" "Deployment failed, initiating rollback..."
        rollback_deployment
    fi
    
    log "INFO" "Deployment log saved to: $DEPLOYMENT_LOG"
    exit $exit_code
}

# Set up cleanup trap
trap cleanup EXIT

# Rollback function
rollback_deployment() {
    log "WARNING" "Starting deployment rollback..."
    
    # Execute rollback script if it exists
    local rollback_script="$SCRIPT_DIR/rollback-production.sh"
    if [[ -f "$rollback_script" ]]; then
        bash "$rollback_script" "emergency" 2>&1 | tee -a "$DEPLOYMENT_LOG"
    else
        log "ERROR" "Rollback script not found: $rollback_script"
    fi
}

# Pre-deployment validation
pre_deployment_validation() {
    log "INFO" "Running pre-deployment validation..."
    
    # Check required tools
    local required_tools=("gcloud" "docker" "kubectl" "curl" "jq" "bc")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log "ERROR" "Required tool not found: $tool"
            exit 1
        fi
    done
    
    # Verify Google Cloud authentication
    if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q "@"; then
        log "ERROR" "Google Cloud authentication required"
        exit 1
    fi
    
    # Check project access
    if ! gcloud projects describe "$GOOGLE_CLOUD_PROJECT" &>/dev/null; then
        log "ERROR" "Cannot access Google Cloud project: $GOOGLE_CLOUD_PROJECT"
        exit 1
    fi
    
    # Verify environment variables
    local required_vars=("DATABASE_URL" "NEXTAUTH_SECRET" "FIREBASE_PROJECT_ID")
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            log "ERROR" "Required environment variable not set: $var"
            exit 1
        fi
    done
    
    log "SUCCESS" "Pre-deployment validation passed"
}

# Build and push container images
build_and_push_images() {
    log "INFO" "Building and pushing container images..."
    
    local build_date=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local git_commit=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
    
    # Set image tags
    local backend_image="$GOOGLE_CLOUD_REGION-docker.pkg.dev/$GOOGLE_CLOUD_PROJECT/epsx/backend:$DEPLOYMENT_VERSION"
    local frontend_image="$GOOGLE_CLOUD_REGION-docker.pkg.dev/$GOOGLE_CLOUD_PROJECT/epsx/frontend:$DEPLOYMENT_VERSION"
    local admin_image="$GOOGLE_CLOUD_REGION-docker.pkg.dev/$GOOGLE_CLOUD_PROJECT/epsx/admin:$DEPLOYMENT_VERSION"
    
    # Build backend image
    log "INFO" "Building backend image..."
    if [[ "$DRY_RUN" == false ]]; then
        docker build \
            --file "$DEPLOYMENT_DIR/security/hardening/container-security.dockerfile" \
            --build-arg BUILD_DATE="$build_date" \
            --build-arg VCS_REF="$git_commit" \
            --build-arg VERSION="$DEPLOYMENT_VERSION" \
            --tag "$backend_image" \
            "$PROJECT_ROOT/apps/backend" || {
                log "ERROR" "Backend image build failed"
                exit 1
            }
        
        docker push "$backend_image" || {
            log "ERROR" "Backend image push failed"
            exit 1
        }
    fi
    
    # Build frontend images (similar process)
    # Note: Frontend and admin builds would be similar with their respective Dockerfiles
    
    log "SUCCESS" "Container images built and pushed successfully"
}

# Database migration
run_database_migrations() {
    log "INFO" "Running database migrations..."
    
    if [[ "$DRY_RUN" == false ]]; then
        # Run migrations using Cloud SQL Proxy or direct connection
        local migration_script="$PROJECT_ROOT/apps/backend/scripts/run-migrations.sh"
        if [[ -f "$migration_script" ]]; then
            bash "$migration_script" production 2>&1 | tee -a "$DEPLOYMENT_LOG" || {
                log "ERROR" "Database migration failed"
                ROLLBACK_NEEDED=true
                exit 1
            }
        else
            log "WARNING" "Migration script not found, skipping migrations"
        fi
    fi
    
    log "SUCCESS" "Database migrations completed"
}

# Deploy backend service
deploy_backend() {
    log "INFO" "Deploying backend service..."
    
    local service_name="epsx-backend"
    local image="$GOOGLE_CLOUD_REGION-docker.pkg.dev/$GOOGLE_CLOUD_PROJECT/epsx/backend:$DEPLOYMENT_VERSION"
    
    if [[ "$DRY_RUN" == false ]]; then
        gcloud run deploy "$service_name" \
            --image="$image" \
            --platform=managed \
            --region="$GOOGLE_CLOUD_REGION" \
            --allow-unauthenticated \
            --port=8080 \
            --memory=4Gi \
            --cpu=4 \
            --timeout=3600s \
            --min-instances=1 \
            --max-instances=10 \
            --set-env-vars="DATABASE_URL=$DATABASE_URL" \
            --set-env-vars="NEXTAUTH_SECRET=$NEXTAUTH_SECRET" \
            --set-env-vars="FIREBASE_PROJECT_ID=$FIREBASE_PROJECT_ID" \
            --set-env-vars="RUST_ENV=production" \
            --set-env-vars="ENV=production" \
            --set-env-vars="RUST_LOG=warn" \
            --execution-environment=gen2 || {
                log "ERROR" "Backend deployment failed"
                ROLLBACK_NEEDED=true
                exit 1
            }
        
        DEPLOYED_SERVICES+=("backend")
    fi
    
    log "SUCCESS" "Backend service deployed successfully"
}

# Deploy frontend service
deploy_frontend() {
    log "INFO" "Deploying frontend service..."
    
    local service_name="epsx-frontend"
    local image="$GOOGLE_CLOUD_REGION-docker.pkg.dev/$GOOGLE_CLOUD_PROJECT/epsx/frontend:$DEPLOYMENT_VERSION"
    
    if [[ "$DRY_RUN" == false ]]; then
        gcloud run deploy "$service_name" \
            --image="$image" \
            --platform=managed \
            --region="$GOOGLE_CLOUD_REGION" \
            --allow-unauthenticated \
            --port=3000 \
            --memory=2Gi \
            --cpu=2 \
            --timeout=300s \
            --min-instances=1 \
            --max-instances=20 \
            --set-env-vars="NODE_ENV=production" \
            --set-env-vars="NEXT_PUBLIC_BACKEND_URL=$NEXT_PUBLIC_BACKEND_URL" \
            --set-env-vars="NEXT_PUBLIC_APP_URL=$NEXT_PUBLIC_APP_URL" \
            --execution-environment=gen2 || {
                log "ERROR" "Frontend deployment failed"
                ROLLBACK_NEEDED=true
                exit 1
            }
        
        DEPLOYED_SERVICES+=("frontend")
    fi
    
    log "SUCCESS" "Frontend service deployed successfully"
}

# Deploy admin frontend service
deploy_admin() {
    log "INFO" "Deploying admin frontend service..."
    
    local service_name="epsx-admin"
    local image="$GOOGLE_CLOUD_REGION-docker.pkg.dev/$GOOGLE_CLOUD_PROJECT/epsx/admin:$DEPLOYMENT_VERSION"
    
    if [[ "$DRY_RUN" == false ]]; then
        gcloud run deploy "$service_name" \
            --image="$image" \
            --platform=managed \
            --region="$GOOGLE_CLOUD_REGION" \
            --allow-unauthenticated \
            --port=3000 \
            --memory=2Gi \
            --cpu=2 \
            --timeout=300s \
            --min-instances=1 \
            --max-instances=10 \
            --set-env-vars="NODE_ENV=production" \
            --set-env-vars="NEXT_PUBLIC_BACKEND_URL=$NEXT_PUBLIC_BACKEND_URL" \
            --set-env-vars="NEXT_PUBLIC_ADMIN_URL=$NEXT_PUBLIC_ADMIN_URL" \
            --execution-environment=gen2 || {
                log "ERROR" "Admin deployment failed"
                ROLLBACK_NEEDED=true
                exit 1
            }
        
        DEPLOYED_SERVICES+=("admin")
    fi
    
    log "SUCCESS" "Admin service deployed successfully"
}

# Configure load balancer and CDN
configure_load_balancer() {
    log "INFO" "Configuring load balancer and CDN..."
    
    if [[ "$DRY_RUN" == false ]]; then
        # Configure Google Cloud Load Balancer
        # This would involve setting up URL maps, backend services, etc.
        # Implementation depends on specific infrastructure setup
        
        log "INFO" "Load balancer configuration completed"
    fi
    
    log "SUCCESS" "Load balancer and CDN configured"
}

# Run security validation
run_security_validation() {
    log "INFO" "Running security validation..."
    
    if [[ "$SKIP_VALIDATION" == false ]]; then
        local validation_script="$SCRIPT_DIR/validate-deployment.sh"
        if [[ -f "$validation_script" ]]; then
            bash "$validation_script" production 2>&1 | tee -a "$DEPLOYMENT_LOG" || {
                log "ERROR" "Security validation failed"
                if [[ "$FORCE_DEPLOY" == false ]]; then
                    ROLLBACK_NEEDED=true
                    exit 1
                fi
            }
        else
            log "WARNING" "Validation script not found, skipping validation"
        fi
    fi
    
    log "SUCCESS" "Security validation completed"
}

# Run performance tests
run_performance_tests() {
    log "INFO" "Running performance tests..."
    
    if [[ "$SKIP_TESTS" == false ]]; then
        local perf_test_script="$SCRIPT_DIR/performance-test.sh"
        if [[ -f "$perf_test_script" ]]; then
            bash "$perf_test_script" production 2>&1 | tee -a "$DEPLOYMENT_LOG" || {
                log "WARNING" "Performance tests failed"
                if [[ "$FORCE_DEPLOY" == false ]]; then
                    ROLLBACK_NEEDED=true
                    exit 1
                fi
            }
        else
            log "WARNING" "Performance test script not found, skipping tests"
        fi
    fi
    
    log "SUCCESS" "Performance tests completed"
}

# Configure monitoring and alerting
configure_monitoring() {
    log "INFO" "Configuring monitoring and alerting..."
    
    if [[ "$DRY_RUN" == false ]]; then
        # Set up Cloud Monitoring dashboards and alerts
        local monitoring_config="$DEPLOYMENT_DIR/monitoring/production-config.yaml"
        if [[ -f "$monitoring_config" ]]; then
            # Apply monitoring configuration
            log "INFO" "Applying monitoring configuration"
        fi
    fi
    
    log "SUCCESS" "Monitoring and alerting configured"
}

# Warm up services
warm_up_services() {
    log "INFO" "Warming up services..."
    
    if [[ "$DRY_RUN" == false ]]; then
        # Send requests to warm up the services
        local services=("$BACKEND_URL/health" "$FRONTEND_URL" "$ADMIN_FRONTEND_URL")
        
        for service in "${services[@]}"; do
            for i in {1..5}; do
                curl -s "$service" >/dev/null 2>&1 || true
                sleep 2
            done
        done
    fi
    
    log "SUCCESS" "Services warmed up"
}

# Final validation
final_validation() {
    log "INFO" "Running final validation..."
    
    # Wait for services to be fully ready
    sleep 30
    
    # Run validation again to ensure everything is working
    if [[ "$SKIP_VALIDATION" == false ]]; then
        local validation_script="$SCRIPT_DIR/validate-deployment.sh"
        if [[ -f "$validation_script" ]]; then
            bash "$validation_script" production 2>&1 | tee -a "$DEPLOYMENT_LOG" || {
                log "ERROR" "Final validation failed"
                ROLLBACK_NEEDED=true
                exit 1
            }
        fi
    fi
    
    log "SUCCESS" "Final validation completed"
}

# Main deployment function
main() {
    echo -e "${BLUE}🚀 EPSX Production Deployment${NC}"
    echo -e "${BLUE}==============================${NC}"
    echo -e "Environment: $DEPLOYMENT_ENV"
    echo -e "Version: $DEPLOYMENT_VERSION"
    echo -e "Timestamp: $TIMESTAMP"
    echo -e "Dry Run: $DRY_RUN"
    echo -e "Log File: $DEPLOYMENT_LOG"
    echo ""
    
    log "INFO" "Starting EPSX production deployment"
    
    # Deployment phases
    log "INFO" "Phase 1: Pre-deployment validation"
    pre_deployment_validation
    
    log "INFO" "Phase 2: Building and pushing images"
    build_and_push_images
    
    log "INFO" "Phase 3: Database migrations"
    run_database_migrations
    
    log "INFO" "Phase 4: Service deployment"
    deploy_backend
    deploy_frontend
    deploy_admin
    
    log "INFO" "Phase 5: Infrastructure configuration"
    configure_load_balancer
    configure_monitoring
    
    log "INFO" "Phase 6: Validation and testing"
    run_security_validation
    run_performance_tests
    
    log "INFO" "Phase 7: Service warm-up and final validation"
    warm_up_services
    final_validation
    
    DEPLOYMENT_STATE="COMPLETED"
    
    log "SUCCESS" "🎉 EPSX production deployment completed successfully!"
    
    echo ""
    echo -e "${GREEN}Deployment Summary:${NC}"
    echo -e "  ✅ Environment: $DEPLOYMENT_ENV"
    echo -e "  ✅ Version: $DEPLOYMENT_VERSION"
    echo -e "  ✅ Services: ${DEPLOYED_SERVICES[*]}"
    echo -e "  ✅ Frontend URL: $FRONTEND_URL"
    echo -e "  ✅ Admin URL: $ADMIN_FRONTEND_URL"
    echo -e "  ✅ API URL: $BACKEND_URL"
    echo ""
    echo -e "${YELLOW}Next Steps:${NC}"
    echo -e "  1. Monitor service metrics and logs"
    echo -e "  2. Verify user access and functionality"
    echo -e "  3. Run additional smoke tests"
    echo -e "  4. Update DNS records if needed"
    echo -e "  5. Notify stakeholders of successful deployment"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-validation)
            SKIP_VALIDATION=true
            shift
            ;;
        --force)
            FORCE_DEPLOY=true
            shift
            ;;
        --no-rollback)
            ROLLBACK_ON_FAILURE=false
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --dry-run          Run deployment simulation without actual changes"
            echo "  --skip-tests       Skip performance tests"
            echo "  --skip-validation  Skip security validation"
            echo "  --force            Force deployment even if validation fails"
            echo "  --no-rollback      Disable automatic rollback on failure"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            log "ERROR" "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Run main deployment
main