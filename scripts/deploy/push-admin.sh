#!/bin/bash
set -e

# EPSX Admin Frontend - Manual Push Script
# Push admin image to trigger auto-revision deployment

PROJECT_ID="epsx-469400"
REGION="us-central1"
REPOSITORY="epsx"
SERVICE_NAME="admin"

echo "🚀 Pushing EPSX Admin Frontend to trigger auto-revision..."
echo "📋 Push Configuration:"
echo "   Service: $SERVICE_NAME"
echo "   Project: $PROJECT_ID"
echo "   Region: $REGION"
echo "   Repository: $REPOSITORY"
echo ""

# Define image tags
LOCAL_TAG="epsx-admin:local"
CLOUD_TAG="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/$SERVICE_NAME"

# Check if local image exists
if ! docker images "$LOCAL_TAG" --format "{{.Repository}}:{{.Tag}}" | grep -q "$LOCAL_TAG"; then
    echo "❌ Error: Local image '$LOCAL_TAG' not found"
    echo "💡 Run './scripts/build/local-admin.sh' first to build the image"
    exit 1
fi

echo "📤 Pushing image to Artifact Registry..."
echo "   From: $LOCAL_TAG"
echo "   To: $CLOUD_TAG:latest"
echo ""

# Push image (this will trigger Cloud Build auto-revision)
docker push "$CLOUD_TAG:latest"

echo ""
echo "✅ Admin Frontend image pushed successfully!"
echo "🔄 Cloud Build will automatically create new revision"
echo ""
echo "📊 Monitor deployment:"
echo "   ./scripts/deploy/status.sh admin"
echo "   ./scripts/deploy/logs.sh admin"
echo ""
echo "🌐 Service URL will be available after deployment completes"