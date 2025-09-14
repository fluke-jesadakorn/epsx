#!/bin/bash
set -e

echo "🧪 Deploying to Development Environment (Unified)"
echo "================================================="

# Get script directory and load utilities
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/utils.sh"

# Load and validate development environment using unified schema
load_and_validate_env "development"

# Get environment configuration
get_env_config "development"

# Check prerequisites
check_deployment_prerequisites

# Setup Google Cloud configuration
print_info "Setting up Google Cloud configuration..."
gcloud config set project $PROJECT_ID
gcloud auth configure-docker $REGISTRY

print_info "Building and deploying to development with unified environment..."
print_info "Development URLs: ${BACKEND_URL}, ${FRONTEND_URL}, ${ADMIN_FRONTEND_URL}"

# Function to build and deploy a service with unified environment
deploy_service() {
    local SERVICE=$1
    local APP_PATH=$2
    local PORT=$3
    local SERVICE_NAME="epsx-${SERVICE}-dev"
    
    print_info "Deploying $SERVICE to development..."
    
    # Build Docker image with unified environment variables
    echo "📦 Building $SERVICE..."
    if [ "$SERVICE" = "backend" ]; then
        docker build \
            --platform linux/amd64 \
            -f $APP_PATH/Dockerfile \
            --build-arg ENV=development \
            --build-arg RUST_ENV=development \
            -t $REGISTRY/$PROJECT_ID/epsx/$SERVICE:$(git rev-parse --short HEAD) \
            -t $REGISTRY/$PROJECT_ID/epsx/$SERVICE:latest \
            .
    else
        # Frontend/Admin build with all unified environment variables (from project root like backend)
        docker build \
            --platform linux/amd64 \
            -f $APP_PATH/Dockerfile \
            $(get_docker_build_args "development") \
            -t $REGISTRY/$PROJECT_ID/epsx/$SERVICE:$(git rev-parse --short HEAD) \
            -t $REGISTRY/$PROJECT_ID/epsx/$SERVICE:latest \
            .
    fi
    
    # Push to Artifact Registry
    echo "📤 Pushing $SERVICE to registry..."
    docker push $REGISTRY/$PROJECT_ID/epsx/$SERVICE:$(git rev-parse --short HEAD)
    docker push $REGISTRY/$PROJECT_ID/epsx/$SERVICE:latest
    
    # Deploy to Cloud Run with unified environment variables
    echo "🚀 Deploying $SERVICE to Cloud Run..."
    if [ "$SERVICE" = "backend" ]; then
        gcloud run deploy $SERVICE_NAME \
            --image=$REGISTRY/$PROJECT_ID/epsx/$SERVICE:$(git rev-parse --short HEAD) \
            --platform=managed \
            --region=$REGION \
            --allow-unauthenticated \
            --port=$PORT \
            $RESOURCES_BACKEND \
            --concurrency=20 \
            --timeout=300s \
            --execution-environment=gen2 \
            --set-env-vars="$(get_backend_env_vars "development")"
    else
        gcloud run deploy $SERVICE_NAME \
            --image=$REGISTRY/$PROJECT_ID/epsx/$SERVICE:$(git rev-parse --short HEAD) \
            --platform=managed \
            --region=$REGION \
            --allow-unauthenticated \
            --port=$PORT \
            $([ "$SERVICE" = "admin" ] && echo "$RESOURCES_ADMIN" || echo "$RESOURCES_FRONTEND") \
            --set-env-vars="$(get_frontend_env_vars "development")"
    fi
        
    print_status "$SERVICE deployed successfully to development!"
    
    # Determine service URL based on service type
    local SERVICE_URL=""
    if [ "$SERVICE" = "frontend" ]; then
        SERVICE_URL="${FRONTEND_URL}"
    elif [ "$SERVICE" = "admin" ]; then
        SERVICE_URL="${ADMIN_FRONTEND_URL}"
    else
        SERVICE_URL="${BACKEND_URL}"
    fi
    
    print_info "Service URL: $SERVICE_URL"
    echo ""
}

# Deploy services with development resource settings
print_info "Starting development deployment of all services..."

deploy_service "backend" "apps/backend" "8080"
deploy_service "frontend" "apps/frontend" "3000"  
deploy_service "admin" "apps/admin-frontend" "3000"

echo "🎉 Development deployment complete!"
echo ""
print_status "Your EPSX development environment is ready:"
echo "🧪 Backend:  ${BACKEND_URL}"
echo "🧪 Frontend: ${FRONTEND_URL}"
echo "🧪 Admin:    ${ADMIN_FRONTEND_URL}"
echo ""
print_info "Development features:"
echo "  ✓ Unified environment configuration"
echo "  ✓ Debug logging enabled"
echo "  ✓ Lower resource usage for cost optimization"
echo "  ✓ Faster iteration cycles"
echo ""
print_warning "Development testing checklist:"
echo "  1. Test all authentication flows"
echo "  2. Verify Firebase functionality"
echo "  3. Check OIDC token exchange"
echo "  4. Test new features safely"