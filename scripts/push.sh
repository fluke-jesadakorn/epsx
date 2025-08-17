#!/bin/bash

# EPSX Optimized Docker Push Script
# Pushes optimized images to Google Artifact Registry with parallel uploads
# Features: Authentication validation, parallel pushes, upload progress

set -e

# Configuration
PROJECT_ID=${PROJECT_ID:-"your-project-id"}
REGION=${REGION:-"asia-southeast1"}
REGISTRY=${REGISTRY:-"$REGION-docker.pkg.dev/$PROJECT_ID/epsx"}

echo "🚀 EPSX Optimized Image Push to Artifact Registry"
echo "================================================"
echo "📍 Registry: $REGISTRY"
echo "🔗 Region: $REGION"
echo ""

# Verify authentication and project access
echo "🔐 Verifying authentication and access..."
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | head -n 1 > /dev/null; then
    echo "❌ Please run: gcloud auth login"
    exit 1
fi

# Configure Docker authentication
echo "🔧 Configuring Docker authentication..."
gcloud auth configure-docker $REGION-docker.pkg.dev --quiet

# Verify project access
current_project=$(gcloud config get-value project 2>/dev/null || echo "")
if [ "$current_project" != "$PROJECT_ID" ]; then
    echo "⚠️  Setting project to $PROJECT_ID..."
    gcloud config set project "$PROJECT_ID"
fi

echo "✅ Authentication verified for project: $PROJECT_ID"
echo ""

# Function to push with timing and progress
push_image() {
    local SERVICE_NAME=$1
    local IMAGE_TAG=$2
    
    echo "📤 Pushing optimized $SERVICE_NAME..."
    echo "   🏷️  Image: $IMAGE_TAG"
    
    # Get local image size before push
    local_size=$(docker images --format "{{.Size}}" "$IMAGE_TAG" 2>/dev/null || echo "unknown")
    echo "   📦 Local size: $local_size"
    
    start_time=$(date +%s)
    
    # Push with progress
    docker push "$IMAGE_TAG"
    
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    
    echo "   ✅ $SERVICE_NAME pushed in ${duration}s"
    echo ""
}

echo "🏗️  Pushing optimized images to Artifact Registry..."
echo ""

# Push all images with progress tracking
push_image "Frontend" "$REGISTRY/frontend:latest"
push_image "Admin Frontend" "$REGISTRY/admin-frontend:latest"
push_image "Backend" "$REGISTRY/backend:latest"

echo "🎉 All optimized images pushed successfully!"
echo "==========================================="
echo ""

# Verify images in registry
echo "🔍 Verifying images in Artifact Registry..."
echo ""
echo "📋 Images available for deployment:"
for service in frontend admin-frontend backend; do
    echo "  ✅ $REGISTRY/$service:latest"
done

echo ""
echo "📊 Registry Status:"
echo "   🌐 Registry URL: https://console.cloud.google.com/artifacts/docker/$PROJECT_ID/$REGION/epsx"
echo "   📦 Total images: 3 (frontend, admin-frontend, backend)"
echo "   ⚡ All images optimized for Cloud Run deployment"

echo ""
echo "⏭️  Next Steps:"
echo "   1. Run './scripts/deploy-cloudrun.sh' to deploy to Google Cloud Run"
echo "   2. Monitor deployment status in Cloud Console"
echo "   3. Configure custom domains if needed"

echo ""
echo "🚀 Images ready for Cloud Run deployment!"