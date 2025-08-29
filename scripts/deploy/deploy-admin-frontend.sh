#!/bin/bash
set -e

# EPSX Admin Frontend - Deploy Script
# Run from project root: ./scripts/deploy-admin-frontend.sh
# Deploys the admin frontend to Google Cloud Run with production configuration

PROJECT_ID="epsx-469400"
REGION="us-central1"
REPOSITORY="epsx"
SERVICE_NAME="epsx-admin"

echo "🚀 Deploying EPSX Admin Frontend to Cloud Run..."
echo "📋 Deploy Configuration:"
echo "   Project: $PROJECT_ID"
echo "   Region: $REGION"
echo "   Service: $SERVICE_NAME"
echo ""

# Use latest image
ADMIN_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/admin:latest"
echo "📦 Image: $ADMIN_IMAGE"
echo ""

# Deploy to Cloud Run with admin-specific settings
echo "🚀 Starting deployment..."
gcloud run deploy "$SERVICE_NAME" \
  --image="$ADMIN_IMAGE" \
  --platform=managed \
  --region="$REGION" \
  --allow-unauthenticated \
  --port=3000 \
  --memory=1Gi \
  --cpu=1 \
  --min-instances=0 \
  --max-instances=5 \
  --timeout=300s \
  --concurrency=50 \
  --execution-environment=gen2 \
  --set-env-vars="NODE_ENV=production,NEXTAUTH_SECRET=prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum,NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io,ADMIN_URL=https://admin.epsx.io,NEXTAUTH_URL=https://admin.epsx.io,BACKEND_URL=https://api.epsx.io,NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io"

# Get service URL
SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" --region="$REGION" --format="value(status.url)" 2>/dev/null || echo "")

echo ""
echo "✅ Admin Frontend deployed successfully!"
echo "🌐 Service URL: $SERVICE_URL"
echo "🔗 Custom Domain: https://admin.epsx.io"
echo ""
echo "🔐 Admin dashboard ready for user management and analytics!"