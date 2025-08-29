#!/bin/bash
set -e

# EPSX Frontend - Deploy Script
# Run from project root: ./scripts/deploy-frontend.sh
# Deploys the frontend to Google Cloud Run with production configuration

PROJECT_ID="epsx-469400"
REGION="us-central1"
REPOSITORY="epsx"
SERVICE_NAME="epsx-frontend"

echo "🚀 Deploying EPSX Frontend to Cloud Run..."
echo "📋 Deploy Configuration:"
echo "   Project: $PROJECT_ID"
echo "   Region: $REGION"
echo "   Service: $SERVICE_NAME"
echo ""

# Use latest image
FRONTEND_IMAGE="$REGION-docker.pkg.dev/$PROJECT_ID/$REPOSITORY/frontend:latest"
echo "📦 Image: $FRONTEND_IMAGE"
echo ""

# Deploy to Cloud Run with production settings
echo "🚀 Starting deployment..."
gcloud run deploy "$SERVICE_NAME" \
  --image="$FRONTEND_IMAGE" \
  --platform=managed \
  --region="$REGION" \
  --allow-unauthenticated \
  --port=3000 \
  --memory=2Gi \
  --cpu=2 \
  --min-instances=0 \
  --max-instances=10 \
  --timeout=300s \
  --concurrency=80 \
  --execution-environment=gen2 \
  --set-env-vars="NODE_ENV=production,NEXTAUTH_SECRET=prod-epsx-jwt-secret-2024-ultra-secure-32-chars-minimum,NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io,APP_URL=https://epsx.io,BACKEND_URL=https://api.epsx.io"

# Get service URL
SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" --region="$REGION" --format="value(status.url)" 2>/dev/null || echo "")

echo ""
echo "✅ Frontend deployed successfully!"
echo "🌐 Service URL: $SERVICE_URL"
echo "🔗 Custom Domain: https://epsx.io"
echo ""
echo "🎨 Frontend ready with clean production CSS!"