#!/bin/bash
set -e

# Configuration
SERVER_IP="100.97.9.56"
REMOTE_DIR="~/epsx"
DEPLOY_ARCHIVE="deploy-prod.tar.gz"

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

# 2. Package
echo ""
echo "💾 Saving images to $DEPLOY_ARCHIVE..."
docker save epsx-frontend:prod epsx-admin-frontend:prod epsx-backend:latest | gzip > $DEPLOY_ARCHIVE

# 3. Transfer
echo ""
echo "📤 Transferring to server (this may take a while)..."
scp $DEPLOY_ARCHIVE $USER@$SERVER_IP:$REMOTE_DIR/

# 4. Deploy
echo ""
echo "🚀 Deploying on server..."
ssh $USER@$SERVER_IP << EOF
  cd $REMOTE_DIR
  echo "   - Loading images..."
  gzip -d -c $DEPLOY_ARCHIVE | docker load
  
  cd prod
  echo "   - Restarting Production Stack..."
  docker compose --env-file .env.prod -f docker-compose.prod.yml up -d --force-recreate
EOF

echo ""
echo "✅ Deployment Complete!"
echo "   Frontend: https://epsx.io"
echo "   Admin:    https://admin.epsx.io"
echo "   API:      https://api.epsx.io"
