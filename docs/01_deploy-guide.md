# EPSX Deployment Guide

Priority-ordered checklist for deploying EPSX to production.

---

## 1. Wallet Setup (FIRST PRIORITY)

### Required Wallets

| Wallet | Purpose | Used In |
|--------|---------|---------|
| **Deployer Wallet** | Deploys smart contracts, holds MANAGER_ROLE | Contract deployment |
| **Admin Wallet** | DEFAULT_ADMIN_ROLE on PaymentEscrow, receives payments | Backend + Frontend env |
| **Company Wallet** | Company treasury for fund management | Backend env |

### Current Production Wallets

```
Deployer/Admin:  0xea64439c9cb1b9Aa588a8D1cE61292DB4036E3dF
Company Wallet:  0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7
```

### Steps

1. **Create deployer wallet** — Import into MetaMask or use hardware wallet
2. **Fund with BNB** — Need BNB on BSC for gas fees (~0.1 BNB for contract deployment)
3. **Secure private key** — Store in `PRIVATE_KEY` env var for Forge deployment only. Never commit.
4. **Set admin wallet** — Can be same as deployer or separate. This wallet receives payments and controls the escrow contract.

### WalletConnect Setup

1. Go to [cloud.walletconnect.com](https://cloud.walletconnect.com)
2. Create a project, get your Project ID
3. Set `WALLETCONNECT_PROJECT_ID` in env

---

## 2. Contract Deployment

### Supported Networks

| Network | Chain ID | RPC | Explorer |
|---------|----------|-----|----------|
| BSC Mainnet | 56 | `https://bsc-dataseed1.binance.org` | bscscan.com |
| BSC Testnet | 97 | `https://data-seed-prebsc-1-s1.binance.org:8545` | testnet.bscscan.com |
| Local Anvil | 31337 | `http://127.0.0.1:8545` | — |

### Deploy PaymentEscrow

```bash
cd apps/contracts

# Testnet first
forge script script/Deploy.s.sol \
  --rpc-url https://data-seed-prebsc-1-s1.binance.org:8545 \
  --broadcast --private-key $PRIVATE_KEY

# Mainnet (after testnet verification)
forge script script/Deploy.s.sol \
  --rpc-url https://bsc-dataseed1.binance.org \
  --broadcast --private-key $PRIVATE_KEY
```

### Current Contract Addresses

**Mainnet (Chain 56):**
```
PaymentEscrow:  0x56e44c9b61Aa24D47C22414e799DA8D76B345Db0
```

**Testnet (Chain 97):**
```
PaymentEscrow:  0x0B58400f86D89ce4c62E8386B74E232c7f410c6A
```

### Supported Payment Tokens

| Token | BSC Mainnet | Decimals |
|-------|-------------|----------|
| USDT | `0x55d398326f99059fF775485246999027B3197955` | 18 |
| USDC | `0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d` | 18 |

After deployment, note the contract address and update env vars in next step.

---

## 3. Environment Variables

Copy template and fill in values:

```bash
cp infrastructure/docker/.env.prod.template infrastructure/docker/.env.prod
```

### Critical Variables (must set first)

```env
# --- Platform ---
ENV=production
NODE_ENV=production
RUST_ENV=production
DOCKER_PLATFORM=linux/arm64          # or linux/amd64

# --- URLs ---
BACKEND_URL=https://api.epsx.io
FRONTEND_URL=https://epsx.io
ADMIN_FRONTEND_URL=https://admin.epsx.io

# --- Database ---
DB_NAME=epsx_prod
DB_USER=epsx_user
DB_PASSWORD=<generate_strong_password>
DB_PORT=5432

# --- Redis ---
REDIS_PASSWORD=<generate_strong_password>

# --- Auth Secrets (generate with: openssl rand -hex 32) ---
JWT_SECRET=<hex_64_chars>
WEB3_APP_SECRET=<hex_64_chars>
WALLET_SIGNATURE_SECRET=<hex_64_chars>
WEB3_SESSION_SECRET=<hex_64_chars>
WEB3_SESSION_DURATION_HOURS=24
WEB3_SIGNATURE_TIMEOUT_MINUTES=10

# --- RSA Keys (generate with: openssl genrsa 2048) ---
RSA_KEY_ID=prod-key-01
RSA_PRIVATE_KEY=<PEM_encoded>
RSA_PUBLIC_KEY=<PEM_encoded>

# --- Blockchain ---
BLOCKCHAIN_NETWORK=mainnet
NEXT_PUBLIC_BLOCKCHAIN_NETWORK=mainnet
CHAIN_ID=56
NEXT_PUBLIC_CHAIN_ID=56
BSC_MAINNET_RPC_URL=https://bsc-dataseed1.binance.org
BSC_TESTNET_RPC_URL=https://data-seed-prebsc-1-s1.binance.org:8545
BSC_REQUIRED_CONFIRMATIONS=12

# --- Contract Addresses ---
PAYMENT_ESCROW_CONTRACT_MAINNET=0x56e44c9b61Aa24D47C22414e799DA8D76B345Db0
NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET=0x56e44c9b61Aa24D47C22414e799DA8D76B345Db0
NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET=0xea64439c9cb1b9Aa588a8D1cE61292DB4036E3dF

# --- Wallets ---
COMPANY_WALLET_MAINNET=0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7

# --- WalletConnect ---
WALLETCONNECT_PROJECT_ID=<your_project_id>
OAUTH_CLIENT_ID=epsx-frontend

# --- Cloudflare ---
CLOUDFLARE_TUNNEL_TOKEN=<tunnel_token>
```

### Generate Secrets

```bash
# JWT & auth secrets
openssl rand -hex 32

# RSA key pair
openssl genrsa -out private.pem 2048
openssl rsa -in private.pem -pubout -out public.pem
```

---

## 4. Infrastructure Setup

### Prerequisites

- Docker & Docker Compose
- Cloudflare account with Tunnel configured
- Server with 16GB+ RAM (Mac Mini arm64 recommended)

### Cloudflare Tunnel

1. Create tunnel: `cloudflared tunnel create epsx-prod`
2. Get tunnel token: `cloudflared tunnel token epsx-prod`
3. Set DNS CNAME records (proxied):

```
epsx.io         → <tunnel-id>.cfargotunnel.com
admin.epsx.io   → <tunnel-id>.cfargotunnel.com
api.epsx.io     → <tunnel-id>.cfargotunnel.com
```

4. Place credentials in `~/.cloudflared/`:
   - `config.yml` — Ingress rules
   - `<tunnel-id>.json` — Credentials file

### Build & Deploy

```bash
# Source env vars
set -a && source infrastructure/docker/.env.prod && set +a
export DOCKER_DEFAULT_PLATFORM=$DOCKER_PLATFORM

# Build all images
docker build \
  --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=$WALLETCONNECT_PROJECT_ID \
  --build-arg NEXT_PUBLIC_APP_URL=$FRONTEND_URL \
  --build-arg NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL \
  --build-arg NEXT_PUBLIC_ADMIN_URL=$ADMIN_FRONTEND_URL \
  --build-arg NEXT_PUBLIC_BLOCKCHAIN_NETWORK=$NEXT_PUBLIC_BLOCKCHAIN_NETWORK \
  --build-arg NEXT_PUBLIC_CHAIN_ID=$NEXT_PUBLIC_CHAIN_ID \
  --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID=$OAUTH_CLIENT_ID \
  --build-arg NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET=$NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET \
  --build-arg NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET=$NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET \
  -f apps/frontend/Dockerfile -t epsx-frontend:prod .

docker build \
  --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=$WALLETCONNECT_PROJECT_ID \
  --build-arg NEXT_PUBLIC_APP_URL=$ADMIN_FRONTEND_URL \
  --build-arg NEXT_PUBLIC_BACKEND_URL=$BACKEND_URL \
  --build-arg NEXT_PUBLIC_ADMIN_URL=$ADMIN_FRONTEND_URL \
  --build-arg NEXT_PUBLIC_BLOCKCHAIN_NETWORK=$NEXT_PUBLIC_BLOCKCHAIN_NETWORK \
  --build-arg NEXT_PUBLIC_CHAIN_ID=$NEXT_PUBLIC_CHAIN_ID \
  --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-admin \
  --build-arg NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET=$NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET \
  --build-arg NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET=$NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET \
  -f apps/admin-frontend/Dockerfile -t epsx-admin-frontend:prod .

docker build -f apps/backend/Dockerfile -t epsx-backend:prod .

# Deploy
cd infrastructure/docker
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d --force-recreate
```

### Verify

```bash
curl -s https://api.epsx.io/health                              # Backend
curl -s -o /dev/null -w "%{http_code}" https://epsx.io           # Frontend (200)
curl -s -o /dev/null -w "%{http_code}" https://admin.epsx.io     # Admin (307 = OK)
```

---

## 5. Services & Ports

| Service | Container | Internal | Host |
|---------|-----------|----------|------|
| Frontend | epsx-prod-frontend | 3000 | 127.0.0.1:4700 |
| Admin | epsx-prod-admin | 3000 | 127.0.0.1:4701 |
| Backend | epsx-prod-backend | 8080 | 127.0.0.1:9180 |
| PostgreSQL | epsx-prod-postgres | 5432 | 5491 |
| Redis | epsx-prod-redis | 6379 | 6342 |
| Cloudflared | epsx-prod-cloudflared | — | — |

---

## Quick Reference: Deployment Order

```
1. Wallet    → Create/fund deployer wallet with BNB
2. Contract  → Deploy PaymentEscrow to BSC (testnet → mainnet)
3. Env       → Fill .env.prod with addresses, secrets, keys
4. Tunnel    → Set up Cloudflare Tunnel + DNS records
5. Build     → Docker build all 3 images
6. Deploy    → docker compose up -d --force-recreate
7. Verify    → curl health endpoints
```
