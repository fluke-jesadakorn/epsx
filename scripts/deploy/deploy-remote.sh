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

echo "   - Building Frontend..."
docker build \
  --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=$WC_PROJECT_ID \
  --build-arg NEXT_PUBLIC_APP_URL=https://epsx.io \
  --build-arg NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io \
  --build-arg NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io \
  --build-arg NEXT_PUBLIC_BLOCKCHAIN_NETWORK=mainnet \
  --build-arg NEXT_PUBLIC_CHAIN_ID=56 \
  --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-frontend \
  --build-arg NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET=0x56e44c9b61Aa24D47C22414e799DA8D76B345Db0 \
  --build-arg NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET=0xea64439c9cb1b9Aa588a8D1cE61292DB4036E3dF \
  -f apps/frontend/Dockerfile -t epsx-frontend:prod .

echo "   - Building Admin..."
docker build \
  --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=$WC_PROJECT_ID \
  --build-arg NEXT_PUBLIC_APP_URL=https://admin.epsx.io \
  --build-arg NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io \
  --build-arg NEXT_PUBLIC_ADMIN_URL=https://admin.epsx.io \
  --build-arg NEXT_PUBLIC_BLOCKCHAIN_NETWORK=mainnet \
  --build-arg NEXT_PUBLIC_CHAIN_ID=56 \
  --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-admin \
  --build-arg NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET=0x56e44c9b61Aa24D47C22414e799DA8D76B345Db0 \
  --build-arg NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET=0xea64439c9cb1b9Aa588a8D1cE61292DB4036E3dF \
  -f apps/admin-frontend/Dockerfile -t epsx-admin-frontend:prod .

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
  echo "      diesel migration run --config apps/backend/diesel.toml"
  echo "      (See GEMINI.md for more details)"
EOF

echo ""
echo "✅ Deployment Complete!"
echo "   Frontend: https://epsx.io"
echo "   Admin:    https://admin.epsx.io"
echo "   API:      https://api.epsx.io"
