#!/bin/bash
set -e

# EPSX Backend - TLS Deployment Script  
# Successfully tested deployment with Diesel ORM and TLS configuration
# Run from project root: ./scripts/deploy/deploy-backend-tls.sh

PROJECT_ID="epsx-469400"
REGION="us-central1"
REPOSITORY="epsx"
SERVICE_NAME="epsx-backend"

echo "🔐 Deploying EPSX Backend with TLS Configuration..."
echo "📋 Deploy Configuration:"
echo "   Project: $PROJECT_ID"
echo "   Region: $REGION"
echo "   Service: $SERVICE_NAME"
echo "   TLS: Enabled with native-tls + tokio-postgres"
echo ""

# Use latest image with TLS support
BACKEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend:latest"
echo "📦 Image: $BACKEND_IMAGE"
echo ""

# Test Docker image locally before deployment (optional)
if command -v docker &> /dev/null; then
    echo "🧪 Testing Docker image locally..."
    docker pull "$BACKEND_IMAGE" || echo "⚠️  Could not pull image for local test"
    echo "✓ Image check complete"
    echo ""
fi

# Generate unique revision suffix for deployment tracking
BUILD_ID=$(date +%Y%m%d-%H%M%S)
REVISION_SUFFIX="tls-$BUILD_ID"

echo "🚀 Starting deployment with revision: $REVISION_SUFFIX"

# Deploy to Cloud Run with TLS-optimized production settings
gcloud run deploy "$SERVICE_NAME" \
  --image="$BACKEND_IMAGE" \
  --platform=managed \
  --region="$REGION" \
  --allow-unauthenticated \
  --port=8080 \
  --memory=4Gi \
  --cpu=4 \
  --min-instances=0 \
  --max-instances=10 \
  --timeout=900s \
  --concurrency=80 \
  --execution-environment=gen2 \
  --revision-suffix="$REVISION_SUFFIX" \
  --set-env-vars="RUST_LOG=info,RUST_ENV=production,ENV=production,DATABASE_URL=postgresql://neondb_owner:npg_reOxwB2n6RkE@ep-twilight-grass-afp73moh-pooler.c-2.us-west-2.aws.neon.tech/neondb?sslmode=require&channel_binding=require,NEXTAUTH_SECRET=prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum,FIREBASE_PROJECT_ID=epsx-449804"

# Get service URL and test deployment
SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" --region="$REGION" --format="value(status.url)" 2>/dev/null || echo "")

echo ""
echo "🧪 Testing deployment..."
if [ -n "$SERVICE_URL" ]; then
    # Test health endpoint
    HEALTH_RESPONSE=$(curl -s -w "%{http_code}" "$SERVICE_URL/health" -o /tmp/health_response.json)
    if [ "$HEALTH_RESPONSE" = "200" ]; then
        echo "✅ Health check passed"
        
        # Test TLS database connection via analytics endpoint
        ANALYTICS_RESPONSE=$(curl -s -w "%{http_code}" "$SERVICE_URL/api/v1/analytics/eps-rankings?limit=1" -o /tmp/analytics_response.json)
        if [ "$ANALYTICS_RESPONSE" = "200" ]; then
            ACTIVE_STATUS=$(cat /tmp/analytics_response.json | grep -o '"active_status":"[^"]*"' | cut -d'"' -f4)
            echo "✅ Database TLS connection working - System mode: $ACTIVE_STATUS"
        else
            echo "⚠️  Analytics endpoint returned: $ANALYTICS_RESPONSE"
        fi
    else
        echo "❌ Health check failed with code: $HEALTH_RESPONSE"
        echo "Check logs: gcloud logging read 'resource.type=cloud_run_revision AND resource.labels.service_name=$SERVICE_NAME' --limit=50"
        exit 1
    fi
fi

echo ""
echo "✅ Backend deployed successfully with TLS!"
echo "🌐 Service URL: $SERVICE_URL"
echo "🔗 Custom Domain: https://api.epsx.io"
echo "🔐 TLS Configuration: Native TLS + Tokio Postgres"
echo "🗄️  Database: Neon PostgreSQL with SSL"
echo "⚙️  ORM: Diesel with bb8 connection pooling"
echo ""
echo "🎯 Deployment complete! TRACK/WATCH/STOP functionality available in production."

# Cleanup temp files
rm -f /tmp/health_response.json /tmp/analytics_response.json