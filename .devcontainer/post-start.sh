#!/bin/bash
# EPSX Dev Container Post-Start Script
# Runs every time the dev container starts

set -e

echo "🔄 Starting EPSX development environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Wait for services to be ready
echo -e "${BLUE}⏳ Waiting for services to be ready...${NC}"

# Wait for database
for i in {1..30}; do
    if pg_isready -h database -p 5432 -U epsx_user 2>/dev/null; then
        echo -e "${GREEN}✅ Database ready${NC}"
        break
    fi
    if [[ $i -eq 30 ]]; then
        echo -e "${YELLOW}⚠️  Database not ready after 30 attempts${NC}"
    fi
    sleep 1
done

# Wait for Redis
for i in {1..30}; do
    if redis-cli -h redis ping 2>/dev/null | grep -q PONG; then
        echo -e "${GREEN}✅ Redis ready${NC}"
        break
    fi
    if [[ $i -eq 30 ]]; then
        echo -e "${YELLOW}⚠️  Redis not ready after 30 attempts${NC}"
    fi
    sleep 1
done

# Check if migrations need to be run
if command -v diesel &> /dev/null && [[ -d apps/backend/diesel_migrations ]]; then
    echo -e "${BLUE}🗄️  Checking database migrations...${NC}"
    cd apps/backend
    
    # Try to run pending migrations
    if diesel migration run --database-url="postgresql://epsx_user:epsx_password@database:5432/epsx_db" 2>/dev/null; then
        echo -e "${GREEN}✅ Database migrations up to date${NC}"
    else
        echo -e "${YELLOW}⚠️  Could not run migrations (may need manual setup)${NC}"
    fi
    cd ../..
fi

echo -e "${GREEN}🚀 Development environment ready!${NC}"
echo ""
echo -e "${BLUE}💡 Quick start:${NC}"
echo -e "  ${YELLOW}Terminal 1:${NC} cargo run             # Start backend"
echo -e "  ${YELLOW}Terminal 2:${NC} pnpm dev:frontend     # Start frontend"  
echo -e "  ${YELLOW}Terminal 3:${NC} pnpm dev:admin        # Start admin"
echo ""
echo -e "${BLUE}Or start everything:${NC}"
echo -e "  ${YELLOW}pnpm dev${NC}                          # All services with localhost"
echo ""
echo -e "${BLUE}🌐 Access URLs:${NC}"
echo -e "  ${YELLOW}Frontend:${NC}     http://localhost:3000"
echo -e "  ${YELLOW}Admin:${NC}        http://localhost:3001"
echo -e "  ${YELLOW}Backend:${NC}      http://localhost:8080"
echo -e "  ${YELLOW}Database:${NC}     postgresql://epsx_user:epsx_password@database:5432/epsx_db"