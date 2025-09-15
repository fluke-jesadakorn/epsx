#!/bin/bash
set -e

# Deploy individual service script - Unified Environment
# Get script directory and load utilities
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/utils.sh"

SERVICE=$1
ENV=${ENV:-development}

if [ -z "$SERVICE" ]; then
    print_error "Usage: ./scripts/deploy-service.sh <backend|frontend|admin>"
    echo "Environment can be set via ENV variable (development|staging|production)"
    echo ""
    echo "Examples:"
    echo "  ENV=development ./scripts/deploy-service.sh backend"
    echo "  ENV=staging ./scripts/deploy-service.sh frontend"
    echo "  ENV=production ./scripts/deploy-service.sh admin"
    exit 1
fi

if [[ ! "$SERVICE" =~ ^(backend|frontend|admin)$ ]]; then
    print_error "Invalid service. Must be: backend, frontend, or admin"
    exit 1
fi

print_info "Deploying $SERVICE to $ENV environment using unified configuration..."

# Load and validate environment using unified schema
load_and_validate_env "$ENV"

# Get environment configuration using utilities
get_env_config "$ENV"

# Check prerequisites
check_deployment_prerequisites

# Production safety check
if [ "$ENV" = "production" ]; then
    print_warning "You are about to deploy $SERVICE to PRODUCTION!"
    print_info "Backend: ${BACKEND_URL}"
    print_info "Frontend: ${FRONTEND_URL}"  
    print_info "Admin: ${ADMIN_FRONTEND_URL}"
    read -p "Are you sure? (yes/no): " -r
    if [[ ! $REPLY =~ ^yes$ ]]; then
        echo "Deployment cancelled"
        exit 1
    fi
fi

# Setup Google Cloud configuration
print_info "Setting up Google Cloud configuration..."
gcloud config set project $PROJECT_ID
gcloud auth configure-docker $REGISTRY

# Set service-specific configuration
case $SERVICE in
    "backend")
        APP_PATH="apps/backend"
        PORT="8080"
        DOCKERFILE="Dockerfile"
        RESOURCES=$RESOURCES_BACKEND
        SERVICE_NAME="epsx-backend"
        if [ "$ENV" = "production" ]; then
            URL="https://api.epsx.io"
        else
            URL="https://${URL_PREFIX}api.epsx.io"
        fi
        ;;
    "frontend")
        APP_PATH="apps/frontend"
        PORT="3000"
        DOCKERFILE="Dockerfile"
        RESOURCES=$RESOURCES_FRONTEND
        SERVICE_NAME="epsx-frontend"
        if [ "$ENV" = "production" ]; then
            URL="https://epsx.io"
        else
            URL="https://${URL_PREFIX}epsx.io"
        fi
        ;;
    "admin")
        APP_PATH="apps/admin-frontend"
        PORT="3000"
        DOCKERFILE="Dockerfile"
        RESOURCES=$RESOURCES_ADMIN
        SERVICE_NAME="epsx-admin"
        if [ "$ENV" = "production" ]; then
            URL="https://admin.epsx.io"
        else
            URL="https://${URL_PREFIX}admin.epsx.io"
        fi
        ;;
esac

echo "📦 Building $SERVICE..."

# Build Docker image
if [ "$SERVICE" = "backend" ]; then
    docker build \
        --platform linux/amd64 \
        -f $APP_PATH/$DOCKERFILE \
        --build-arg ENV=$ENV \
        -t $REGISTRY/$PROJECT_ID/epsx/$SERVICE:$(git rev-parse --short HEAD) \
        -t $REGISTRY/$PROJECT_ID/epsx/$SERVICE:latest \
        .
else
    # Frontend/Admin build - use actual dev URLs
    if [ "$ENV" = "production" ]; then
        BASE_BACKEND_URL="https://api.epsx.io"
        BASE_APP_URL="https://epsx.io"
        BASE_ADMIN_URL="https://admin.epsx.io"
    else
        # Use environment variables from development.env for development
        BASE_BACKEND_URL="${NEXT_PUBLIC_BACKEND_URL}"
        BASE_APP_URL="${NEXT_PUBLIC_APP_URL}"
        BASE_ADMIN_URL="${NEXT_PUBLIC_ADMIN_URL}"
    fi
    
    docker build \
        --platform linux/amd64 \
        -f $APP_PATH/$DOCKERFILE \
        --build-arg NODE_ENV=$ENV \
        --build-arg ADMIN_FRONTEND_URL=$BASE_ADMIN_URL \
        --build-arg NEXTAUTH_SECRET=$NEXTAUTH_SECRET \
        --build-arg OIDC_ADMIN_CLIENT_ID=$OIDC_ADMIN_CLIENT_ID \
        --build-arg OIDC_ADMIN_CLIENT_SECRET=$OIDC_ADMIN_CLIENT_SECRET \
        --build-arg BACKEND_URL=$BASE_BACKEND_URL \
        -t $REGISTRY/$PROJECT_ID/epsx/$SERVICE:$(git rev-parse --short HEAD) \
        -t $REGISTRY/$PROJECT_ID/epsx/$SERVICE:latest \
        .
fi

# Push to registry
echo "📤 Pushing to Artifact Registry..."
docker push $REGISTRY/$PROJECT_ID/epsx/$SERVICE:$(git rev-parse --short HEAD)
docker push $REGISTRY/$PROJECT_ID/epsx/$SERVICE:latest

# Deploy to Cloud Run
echo "🚀 Deploying to Cloud Run..."

# Set revision tag for non-production deployments
REVISION_TAG=""
TRAFFIC_FLAGS=""
if [ "$ENV" != "production" ]; then
    REVISION_TAG="--tag=$ENV"
    TRAFFIC_FLAGS="--no-traffic"
    print_info "Deploying with tag '$ENV' and no traffic for manual promotion"
else
    print_info "Deploying to production with 100% traffic"
fi

if [ "$SERVICE" = "backend" ]; then
    # Backend deployment with environment variables
    gcloud run deploy $SERVICE_NAME \
        --image=$REGISTRY/$PROJECT_ID/epsx/$SERVICE:$(git rev-parse --short HEAD) \
        --platform=managed \
        --region=$REGION \
        --allow-unauthenticated \
        --port=$PORT \
        $RESOURCES \
        $REVISION_TAG \
        $TRAFFIC_FLAGS \
        --concurrency=$([ "$ENV" = "production" ] && echo "80" || echo "80") \
        --timeout=$([ "$ENV" = "production" ] && echo "900s" || echo "600s") \
        --execution-environment=gen2 \
        $([ "$ENV" = "production" ] && echo "--startup-probe-timeout-seconds=10 --startup-probe-period-seconds=15 --startup-probe-failure-threshold=8" || echo "") \
        --set-env-vars="ENV=$ENV,NODE_ENV=$ENV,RUST_ENV=$ENV,RUST_LOG=$([ "$ENV" = "production" ] && echo "info" || echo "debug")" \
        --set-env-vars="DATABASE_URL=${DATABASE_URL}" \
        --set-env-vars="NEXTAUTH_SECRET=${NEXTAUTH_SECRET}" \
        --set-env-vars="FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID}" \
        --set-env-vars="FIREBASE_PRIVATE_KEY=${FIREBASE_PRIVATE_KEY}" \
        --set-env-vars="FIREBASE_CLIENT_EMAIL=${FIREBASE_CLIENT_EMAIL}" \
        --set-env-vars="OIDC_CLIENT_SECRET=${OIDC_CLIENT_SECRET}" \
        --set-env-vars="OIDC_ADMIN_CLIENT_SECRET=${OIDC_ADMIN_CLIENT_SECRET}" \
        --set-env-vars="BACKEND_URL=$URL" \
        --set-env-vars="FRONTEND_URL=$([ "$ENV" = "production" ] && echo "https://epsx.io" || echo "https://${URL_PREFIX}epsx.io")" \
        --set-env-vars="ADMIN_FRONTEND_URL=$([ "$ENV" = "production" ] && echo "https://admin.epsx.io" || echo "https://${URL_PREFIX}admin.epsx.io")" \
        $([ "$ENV" = "production" ] && echo "--set-env-vars=\"DATABASE_ACQUIRE_TIMEOUT=90,DATABASE_MAX_CONNECTIONS=2,DATABASE_MIN_CONNECTIONS=1,SKIP_DB_TEST=true\"" || echo "")
else
    # Frontend/Admin deployment
    if [ "$ENV" = "production" ]; then
        BASE_BACKEND_URL="https://api.epsx.io"
        BASE_APP_URL="https://epsx.io"
        BASE_ADMIN_URL="https://admin.epsx.io"
    else
        # Use environment variables from development.env
        BASE_BACKEND_URL="${NEXT_PUBLIC_BACKEND_URL}"
        BASE_APP_URL="${NEXT_PUBLIC_APP_URL}"
        BASE_ADMIN_URL="${NEXT_PUBLIC_ADMIN_URL}"
    fi
    
    gcloud run deploy $SERVICE_NAME \
        --image=$REGISTRY/$PROJECT_ID/epsx/$SERVICE:$(git rev-parse --short HEAD) \
        --platform=managed \
        --region=$REGION \
        --allow-unauthenticated \
        --port=$PORT \
        $RESOURCES \
        $REVISION_TAG \
        $TRAFFIC_FLAGS \
        --set-env-vars="NODE_ENV=$ENV" \
        --set-env-vars="NEXT_PUBLIC_BACKEND_URL=$BASE_BACKEND_URL" \
        --set-env-vars="NEXT_PUBLIC_APP_URL=$BASE_APP_URL" \
        --set-env-vars="NEXT_PUBLIC_ADMIN_URL=$BASE_ADMIN_URL" \
        --set-env-vars="NEXT_PUBLIC_OAUTH_CLIENT_ID=${OIDC_CLIENT_ID}" \
        $([ -n "$NEXT_PUBLIC_FIREBASE_API_KEY" ] && echo "--set-env-vars=\"NEXT_PUBLIC_FIREBASE_API_KEY=${NEXT_PUBLIC_FIREBASE_API_KEY}\"" || echo "") \
        $([ -n "$NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN" ] && echo "--set-env-vars=\"NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=${NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN}\"" || echo "") \
        $([ -n "$NEXT_PUBLIC_FIREBASE_PROJECT_ID" ] && echo "--set-env-vars=\"NEXT_PUBLIC_FIREBASE_PROJECT_ID=${NEXT_PUBLIC_FIREBASE_PROJECT_ID}\"" || echo "") \
        $([ -n "$NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET" ] && echo "--set-env-vars=\"NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET=${NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET}\"" || echo "") \
        $([ -n "$NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID" ] && echo "--set-env-vars=\"NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID=${NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID}\"" || echo "") \
        $([ -n "$NEXT_PUBLIC_FIREBASE_APP_ID" ] && echo "--set-env-vars=\"NEXT_PUBLIC_FIREBASE_APP_ID=${NEXT_PUBLIC_FIREBASE_APP_ID}\"" || echo "") \
        $([ -n "$NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID" ] && echo "--set-env-vars=\"NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID=${NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID}\"" || echo "")
fi

echo "✅ $SERVICE deployed successfully to $ENV!"

if [ "$ENV" != "production" ]; then
    echo "🏷️  Tagged URL: https://$ENV---$SERVICE_NAME-$(echo $PROJECT_ID | tr -d '-').$REGION.run.app"
    echo "📊 Traffic: 0% (manual promotion required)"
    echo ""
    echo "To promote this revision to receive traffic:"
    echo "  ./scripts/deploy/promote.sh $SERVICE $ENV"
    echo "  Or manually: gcloud run services update-traffic $SERVICE_NAME --to-revisions=$ENV=100"
else
    echo "🌐 Production URL: $URL"
    
    # Health check for backend in production
    if [ "$SERVICE" = "backend" ]; then
        echo "🧪 Testing backend health..."
        sleep 5  # Give it a moment to start
        HEALTH_RESPONSE=$(curl -s -w "%{http_code}" "$URL/health" -o /dev/null 2>/dev/null || echo "000")
        if [ "$HEALTH_RESPONSE" = "200" ]; then
            echo "✅ Backend health check passed"
        else
            echo "⚠️  Backend health check returned: $HEALTH_RESPONSE"
        fi
    fi
fi

echo "🎉 Individual service deployment complete!"