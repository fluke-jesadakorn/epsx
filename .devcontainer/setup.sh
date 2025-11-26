#!/bin/bash
# EPSX Dev Container Setup Script - Production Docker Configuration
# Runs after dev container is created with multi-service architecture

set -e

echo "🚀 Setting up EPSX development environment with production Docker..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check if we're in the frontend container
echo -e "${BLUE}📋 Checking container environment...${NC}"
if [[ "$PRODUCTION_DOCKER" == "true" ]]; then
    echo -e "${GREEN}✅ Production Docker multi-service setup detected${NC}"
    echo -e "${BLUE}   Frontend Container: ${FRONTEND_CONTAINER}${NC}"
    echo -e "${BLUE}   Backend Container: ${BACKEND_CONTAINER}${NC}"
    echo -e "${BLUE}   Database Container: ${DATABASE_CONTAINER}${NC}"
else
    echo -e "${YELLOW}⚠️  Legacy dev container setup${NC}"
fi

# Install pnpm globally (in frontend container)
echo -e "${BLUE}📦 Setting up Node.js environment...${NC}"
if command -v pnpm &> /dev/null; then
    echo -e "${GREEN}✅ pnpm already available (version: $(pnpm --version))${NC}"
else
    echo -e "${YELLOW}⚠️  pnpm not found${NC}"
fi

# Check backend tools via docker compose exec
echo -e "${BLUE}🦀 Checking Rust environment in backend container...${NC}"
if docker compose exec backend cargo --version &> /dev/null; then
    echo -e "${GREEN}✅ Rust available in backend container${NC}"
    echo -e "${GREEN}✅ Using SQLx for database operations${NC}"
else
    echo -e "${YELLOW}⚠️  Backend container not available yet${NC}"
fi

# Install project dependencies
echo -e "${BLUE}📦 Installing project dependencies...${NC}"
pnpm install

# Set up Git configuration if not already configured
if [[ -z "$(git config --global user.name)" ]]; then
    echo -e "${YELLOW}⚙️  Setting up Git configuration...${NC}"
    echo "Please configure Git:"
    echo "  git config --global user.name 'Your Name'"
    echo "  git config --global user.email 'your.email@example.com'"
fi

# Create environment file from template if it doesn't exist
if [[ ! -f .env.local ]]; then
    echo -e "${BLUE}📋 Creating .env.local from template...${NC}"
    if [[ -f .env.example ]]; then
        cp .env.example .env.local
        echo -e "${GREEN}✅ Created .env.local${NC}"
        echo -e "${YELLOW}⚠️  Please update .env.local with your actual values${NC}"
    else
        echo -e "${YELLOW}⚠️  No .env.example found${NC}"
    fi
fi

# Set up database if it's available
echo -e "${BLUE}🗄️  Setting up database...${NC}"
if pg_isready -h database -p 5432 -U epsx_user 2>/dev/null; then
    echo -e "${GREEN}✅ Database is available${NC}"
    
    # Run migrations via backend container using SQLx
    if [[ -d apps/backend/migrations ]]; then
        echo -e "${BLUE}🔄 Running database migrations via backend container...${NC}"
        docker compose exec backend bash -c "
            cd /workspace/apps/backend && 
            DATABASE_URL='postgresql://epsx_user:epsx_password@database:5432/epsx_db' cargo run --bin migrate up
        " || {
            echo -e "${YELLOW}⚠️  Migrations failed (expected on first run)${NC}"
        }
    fi
else
    echo -e "${YELLOW}⚠️  Database not available yet${NC}"
fi

# Set up Rust environment in backend container
echo -e "${BLUE}🦀 Setting up Rust environment in backend container...${NC}"
docker compose exec backend bash -c "
    rustup component add rustfmt clippy rust-analyzer 2>/dev/null || true
    echo 'Rust components updated'
" || echo -e "${YELLOW}⚠️  Backend container not ready${NC}"

# Build Rust project to cache dependencies in backend container
echo -e "${BLUE}🔨 Building backend in production container...${NC}"
docker compose exec backend bash -c "
    cd /workspace/apps/backend && 
    cargo check
" || echo -e "${YELLOW}⚠️  Backend build failed (expected without complete setup)${NC}"

# Create helpful aliases for multi-service Docker development
echo -e "${BLUE}⚙️  Setting up production Docker development aliases...${NC}"
cat >> ~/.bashrc << 'EOF'

# EPSX Production Docker Development Aliases
alias pdev="pnpm dev"
alias pdev-frontend="pnpm dev:frontend"
alias pdev-admin="pnpm dev:admin"
alias pbuild="pnpm build"
alias ptest="pnpm test"

# Multi-container Backend aliases  
alias rs-check="docker compose exec backend bash -c 'cd /workspace/apps/backend && cargo check'"
alias rs-run="docker compose exec backend bash -c 'cd /workspace/apps/backend && cargo run'"
alias rs-test="docker compose exec backend bash -c 'cd /workspace/apps/backend && cargo test'"
alias rs-fmt="docker compose exec backend bash -c 'cd /workspace/apps/backend && cargo fmt'"
alias rs-watch="docker compose exec backend bash -c 'cd /workspace/apps/backend && cargo watch -x run'"

# Database aliases for container environment
alias db-migrate="docker compose exec backend bash -c 'cd /workspace/apps/backend && DATABASE_URL=\"postgresql://epsx_user:epsx_password@database:5432/epsx_db\" cargo run --bin migrate up'"
alias db-status="docker compose exec backend bash -c 'cd /workspace/apps/backend && DATABASE_URL=\"postgresql://epsx_user:epsx_password@database:5432/epsx_db\" cargo run --bin migrate status'"
alias psql-dev="docker compose exec database psql postgresql://epsx_user:epsx_password@localhost:5432/epsx_db"

# Docker compose aliases for multi-service development
alias dc="docker compose"
alias dc-up="docker compose up -d"
alias dc-down="docker compose down"
alias dc-logs="docker compose logs -f"
alias dc-restart="docker compose restart"
alias dc-rebuild="docker compose up -d --build"

# Container execution aliases
alias exec-frontend="docker compose exec frontend bash"
alias exec-backend="docker compose exec backend bash"
alias exec-db="docker compose exec database bash"

# Service-specific log viewing
alias logs-frontend="docker compose logs -f frontend"
alias logs-backend="docker compose logs -f backend"
alias logs-db="docker compose logs -f database"
alias logs-proxy="docker compose logs -f proxy"

# Development workflow aliases
alias dev-start="docker compose up -d && echo 'All services started'"
alias dev-stop="docker compose down && echo 'All services stopped'"
alias dev-rebuild="docker compose down && docker compose up -d --build"

# Navigation aliases (same as before)
alias cdbe="cd /workspace/apps/backend"
alias cdfe="cd /workspace/apps/frontend" 
alias cdad="cd /workspace/apps/admin-frontend"
alias cdroot="cd /workspace"
EOF

# Source the new aliases
source ~/.bashrc 2>/dev/null || true

echo ""
echo -e "${GREEN}🎉 Production Docker dev container setup complete!${NC}"
echo ""
echo -e "${BLUE}🐳 Multi-Service Architecture:${NC}"
echo -e "  ${GREEN}✅ Frontend Container${NC}    - Node.js 22-alpine (production base)"
echo -e "  ${GREEN}✅ Backend Container${NC}     - Rust 1.85-slim (production base)"
echo -e "  ${GREEN}✅ Database Container${NC}    - PostgreSQL 16-alpine"
echo -e "  ${GREEN}✅ Redis Container${NC}       - Redis 7.2-alpine"
echo -e "  ${GREEN}✅ Proxy Container${NC}       - Caddy reverse proxy"
echo ""
echo -e "${BLUE}📚 Available commands:${NC}"
echo -e "  ${YELLOW}pnpm dev${NC}                 - Start frontend/admin dev servers"
echo -e "  ${YELLOW}rs-run${NC}                   - Run backend in Rust container"
echo -e "  ${YELLOW}rs-watch${NC}                 - Watch mode for backend development"
echo -e "  ${YELLOW}dev-start${NC}                - Start all services"
echo -e "  ${YELLOW}dev-stop${NC}                 - Stop all services"
echo -e "  ${YELLOW}dev-rebuild${NC}              - Rebuild and restart all services"
echo ""
echo -e "${BLUE}🌐 Production domains (with proxy):${NC}"
echo -e "  ${YELLOW}https://epsx.io${NC}          - Frontend (port 3000)"
echo -e "  ${YELLOW}https://admin.epsx.io${NC}    - Admin (port 3001)" 
echo -e "  ${YELLOW}https://api.epsx.io${NC}      - Backend API (port 8080)"
echo ""
echo -e "${BLUE}🐳 Container access:${NC}"
echo -e "  ${YELLOW}exec-frontend${NC}            - Shell into frontend container"
echo -e "  ${YELLOW}exec-backend${NC}             - Shell into backend container"
echo -e "  ${YELLOW}exec-db${NC}                  - Shell into database container"
echo ""
echo -e "${BLUE}📁 Quick navigation:${NC}"
echo -e "  ${YELLOW}cdbe${NC}                     - Go to backend"
echo -e "  ${YELLOW}cdfe${NC}                     - Go to frontend"
echo -e "  ${YELLOW}cdad${NC}                     - Go to admin"
echo ""
echo -e "${BLUE}🔍 Monitoring:${NC}"
echo -e "  ${YELLOW}dc-logs${NC}                  - View all service logs"
echo -e "  ${YELLOW}logs-backend${NC}             - Backend-specific logs"
echo -e "  ${YELLOW}logs-frontend${NC}            - Frontend-specific logs"
echo ""
echo -e "${YELLOW}⚠️  Production Docker Benefits:${NC}"
echo -e "  1. Same base images as production (Node.js 22-alpine, Rust 1.85-slim)"
echo -e "  2. Multi-stage optimization patterns"
echo -e "  3. Container isolation and security"
echo -e "  4. Production-ready build caching"
echo ""
echo -e "${YELLOW}⚠️  Don't forget to:${NC}"
echo -e "  1. Update .env.local with your actual values"
echo -e "  2. Configure Git if needed"
echo -e "  3. Set up /etc/hosts on host machine for domain development"