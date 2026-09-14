#!/bin/bash
set -e

# Configuration
SERVER_IP="100.97.9.56"
REMOTE_DIR="~/epsx"

# WalletConnect Project ID for Production
WC_PROJECT_ID="04e0a500abfa1e095bf8f64b15fa2812"

echo "🚀 Starting EPSX Production Deployment (Remote)"
echo "Target: $SERVER_IP"

# 1. Build Images
echo ""
echo "📦 Building Production Images (linux/arm64)..."
export DOCKER_DEFAULT_PLATFORM=linux/arm64

echo "   - Building Frontend (Rust/Dioxus — runtime env only, no NEXT_PUBLIC)..."
docker build -f apps/frontend/Dockerfile -t epsx-frontend:prod .

echo "   - Building Admin (Rust/Dioxus — runtime env only)..."
docker build -f apps/admin/Dockerfile -t epsx-admin:prod .

echo "   - Building Backend..."
docker build -f apps/backend/Dockerfile -t epsx-backend:prod .

# 2. Transfer & Load
echo ""
echo "📤 Transferring and loading images directly to server (this may take a while)..."
docker save epsx-frontend:prod epsx-admin-frontend:prod epsx-backend:prod | gzip | ssh $USER@$SERVER_IP "gzip -d | docker load"

# 3. Deploy
echo ""
echo "🚀 Deploying on server..."
ssh $USER@$SERVER_IP << EOF
  cd $REMOTE_DIR/prod
  
  echo "   - Updating Production Stack (Zero-Downtime)..."
  docker compose --env-file .env.prod -f docker-compose.prod.yml up -d
  
  echo "   - Cleaning up old images..."
  docker image prune -f
  
  echo "   - Verifying service health..."
  # Wait for up to 120 seconds for services to become healthy
  for i in {1..24}; do
    if docker compose --env-file .env.prod -f docker-compose.prod.yml ps | grep -q "(unhealthy)"; then
      echo "     ⚠️ Services unhealthy, retrying..."
      sleep 5
      continue
    fi
    
    # Check if containers are actually running
    if ! docker compose --env-file .env.prod -f docker-compose.prod.yml ps | grep -q "Up"; then
         echo "     ⏳ Waiting for services to start..."
         sleep 5
         continue
    fi
    
    echo "     ✅ Services are healthy!"
    break
  done
  
  echo ""
  echo "   ⚠️ Reminder: If there are database migrations, please run:"
  echo "      ssh -L 5434:localhost:5433 $USER@$SERVER_IP -Nf"
  echo "      export DATABASE_URL=postgres://epsx_user:password@localhost:5434/epsx_prod"
  echo "      sqlx migrate run --source apps/backend/migrations/core"
  echo "      (See GEMINI.md for more details)"
EOF

echo ""
echo "✅ Deployment Complete!"
echo "   Frontend: https://epsx.io"
echo "   Admin:    https://admin.epsx.io"
echo "   API:      https://api.epsx.io"
