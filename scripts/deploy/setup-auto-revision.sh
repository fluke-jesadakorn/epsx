#!/bin/bash
set -e

# EPSX - Setup Auto-Revision Cloud Build Triggers
# One-time setup to enable automatic revision deployment when images are pushed

PROJECT_ID="epsx-469400"
REGION="us-central1"
REPOSITORY="epsx"

echo "🚀 Setting up Auto-Revision Cloud Build Triggers..."
echo "📋 Configuration:"
echo "   Project: $PROJECT_ID"
echo "   Region: $REGION"
echo "   Repository: $REPOSITORY"
echo ""

# Navigate to project root
SCRIPT_DIR="$(dirname "${BASH_SOURCE[0]}")"
cd "$SCRIPT_DIR/../.."
echo "📁 Working from: $(pwd)"

# Check if cloudbuild-auto-revision.yaml exists
if [ ! -f "cloudbuild-auto-revision.yaml" ]; then
    echo "❌ Error: cloudbuild-auto-revision.yaml not found"
    echo "💡 Ensure the file exists in project root"
    exit 1
fi

echo "⚡ Creating Cloud Build triggers for auto-revision deployment..."
echo ""

# Create trigger for Frontend
echo "1️⃣  Creating Frontend auto-revision trigger..."
gcloud builds triggers create pubsub \
    --name="auto-revision-frontend" \
    --topic="projects/$PROJECT_ID/topics/gcr" \
    --build-config="cloudbuild-auto-revision.yaml" \
    --substitutions="_REGION=$REGION,_IMAGE_NAME=$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend,_IMAGE_TAG=latest" \
    --filter="data.action='INSERT' && data.tag=='latest' && data.name.indexOf('/$REPOSITORY/frontend') > -1" \
    --description="Auto-deploy frontend when image is pushed to Artifact Registry" \
    --quiet || echo "Frontend trigger may already exist"

echo ""

# Create trigger for Admin
echo "2️⃣  Creating Admin auto-revision trigger..."
gcloud builds triggers create pubsub \
    --name="auto-revision-admin" \
    --topic="projects/$PROJECT_ID/topics/gcr" \
    --build-config="cloudbuild-auto-revision.yaml" \
    --substitutions="_REGION=$REGION,_IMAGE_NAME=$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin,_IMAGE_TAG=latest" \
    --filter="data.action=='INSERT' && data.tag=='latest' && data.name.indexOf('/$REPOSITORY/admin') > -1" \
    --description="Auto-deploy admin when image is pushed to Artifact Registry" \
    --quiet || echo "Admin trigger may already exist"

echo ""

# Create trigger for Backend
echo "3️⃣  Creating Backend auto-revision trigger..."
gcloud builds triggers create pubsub \
    --name="auto-revision-backend" \
    --topic="projects/$PROJECT_ID/topics/gcr" \
    --build-config="cloudbuild-auto-revision.yaml" \
    --substitutions="_REGION=$REGION,_IMAGE_NAME=$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/backend,_IMAGE_TAG=latest" \
    --filter="data.action=='INSERT' && data.tag=='latest' && data.name.indexOf('/$REPOSITORY/backend') > -1" \
    --description="Auto-deploy backend when image is pushed to Artifact Registry" \
    --quiet || echo "Backend trigger may already exist"

echo ""
echo "✅ Auto-revision triggers setup completed!"
echo ""
echo "📋 Created triggers:"
echo "   • auto-revision-frontend"
echo "   • auto-revision-admin"
echo "   • auto-revision-backend"
echo ""
echo "🔄 How it works:"
echo "   1. Push image: docker push $REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/[service]:latest"
echo "   2. Trigger fires: Cloud Build automatically deploys new revision"
echo "   3. Service updates: New revision becomes active"
echo ""
echo "📤 Manual push commands:"
echo "   ./scripts/deploy/push-frontend.sh    # Push frontend"
echo "   ./scripts/deploy/push-admin.sh       # Push admin"
echo "   ./scripts/deploy/push-backend.sh     # Push backend"
echo "   ./scripts/deploy/push-all.sh         # Push all services"
echo ""
echo "📊 Monitor deployments:"
echo "   ./scripts/deploy/status.sh"
echo "   ./scripts/deploy/logs.sh"
echo ""
echo "🎉 Auto-revision deployment is now active!"