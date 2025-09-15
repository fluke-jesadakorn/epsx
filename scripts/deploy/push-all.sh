#!/bin/bash
set -e

# EPSX - Push All Services Script
# Push all images to trigger auto-revision deployment

PROJECT_ID="epsx-469400"
REGION="us-central1"
REPOSITORY="epsx"

echo "🚀 Pushing All EPSX Services to trigger auto-revisions..."
echo "📋 Services to push:"
echo "   • Frontend"
echo "   • Admin Frontend"  
echo "   • Backend"
echo ""
echo "📋 Push Configuration:"
echo "   Project: $PROJECT_ID"
echo "   Region: $REGION"
echo "   Repository: $REPOSITORY"
echo ""

# Get push timestamp
PUSH_TIMESTAMP=$(date +%Y%m%d-%H%M%S)
echo "🔨 Push Session: $PUSH_TIMESTAMP"

# Navigate to script directory
SCRIPT_DIR="$(dirname "${BASH_SOURCE[0]}")"
cd "$SCRIPT_DIR"

# Check if all local images exist
MISSING_IMAGES=()

if ! docker images "epsx-frontend:local" --format "{{.Repository}}:{{.Tag}}" | grep -q "epsx-frontend:local"; then
    MISSING_IMAGES+=("epsx-frontend:local")
fi

if ! docker images "epsx-admin:local" --format "{{.Repository}}:{{.Tag}}" | grep -q "epsx-admin:local"; then
    MISSING_IMAGES+=("epsx-admin:local")
fi

if ! docker images "epsx-backend:local" --format "{{.Repository}}:{{.Tag}}" | grep -q "epsx-backend:local"; then
    MISSING_IMAGES+=("epsx-backend:local")
fi

if [ ${#MISSING_IMAGES[@]} -gt 0 ]; then
    echo "❌ Error: Missing local images:"
    for image in "${MISSING_IMAGES[@]}"; do
        echo "   • $image"
    done
    echo ""
    echo "💡 Build missing images first:"
    echo "   ./scripts/build/local-all.sh"
    exit 1
fi

echo ""
echo "======================================"
echo "1️⃣  Pushing Frontend..."
echo "======================================"
./push-frontend.sh

echo ""
echo "======================================"
echo "2️⃣  Pushing Admin Frontend..."
echo "======================================"
./push-admin.sh

echo ""
echo "======================================"
echo "3️⃣  Pushing Backend..."
echo "======================================"
./push-backend.sh

echo ""
echo "🎉 All images pushed successfully!"
echo "🔄 Cloud Build will automatically create new revisions for all services"
echo ""
echo "📊 Monitor all deployments:"
echo "   ./scripts/deploy/status.sh"
echo "   ./scripts/deploy/logs.sh"
echo ""
echo "⏱️  New revisions should be available within 2-5 minutes"