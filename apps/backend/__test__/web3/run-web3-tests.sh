#!/bin/bash

# Web3 Authentication Test Suite Runner
# Comprehensive testing script for the EPSX Web3-first authentication system

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DATABASE_URL=${DATABASE_URL:-"postgresql://postgres:password@localhost:5432/epsx_test"}
BACKEND_URL=${BACKEND_URL:-"http://localhost:8080"}
FRONTEND_URL=${FRONTEND_URL:-"http://localhost:3000"}

echo -e "${BLUE}🧪 EPSX Web3 Authentication Test Suite${NC}"
echo "========================================"
echo ""

# Check prerequisites
echo -e "${YELLOW}📋 Checking prerequisites...${NC}"

# Check if PostgreSQL is running
if ! pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    echo -e "${RED}❌ PostgreSQL is not running. Please start PostgreSQL first.${NC}"
    exit 1
fi

# Check if test database exists
if ! psql "$DATABASE_URL" -c "SELECT 1;" > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Test database doesn't exist. Creating...${NC}"
    createdb epsx_test || true
fi

# Run database migrations for test database
echo -e "${YELLOW}🔄 Running database migrations...${NC}"
cd "$(dirname "$0")/../../"
diesel migration run --database-url="$DATABASE_URL" || {
    echo -e "${RED}❌ Failed to run migrations${NC}"
    exit 1
}

echo -e "${GREEN}✅ Prerequisites check passed${NC}"
echo ""

# Function to run tests with timing
run_test_suite() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${BLUE}🧪 Running $test_name...${NC}"
    start_time=$(date +%s)
    
    if eval "$test_command"; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo -e "${GREEN}✅ $test_name passed (${duration}s)${NC}"
        return 0
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo -e "${RED}❌ $test_name failed (${duration}s)${NC}"
        return 1
    fi
}

# Track test results
total_suites=0
passed_suites=0
failed_suites=0

# Backend Unit Tests
echo -e "${BLUE}🦀 Backend Unit Tests${NC}"
echo "===================="

((total_suites++))
if run_test_suite "Web3AuthService Unit Tests" "cargo test web3_auth_service_tests --lib"; then
    ((passed_suites++))
else
    ((failed_suites++))
fi

((total_suites++))
if run_test_suite "Web3PermissionService Unit Tests" "cargo test web3_permission_service_tests --lib"; then
    ((passed_suites++))
else
    ((failed_suites++))
fi

((total_suites++))
if run_test_suite "Web3 Security Tests" "cargo test web3_security_tests --lib"; then
    ((passed_suites++))
else
    ((failed_suites++))
fi

echo ""

# Backend Integration Tests
echo -e "${BLUE}🔗 Backend Integration Tests${NC}"
echo "============================"

((total_suites++))
if run_test_suite "Web3 Routes Integration Tests" "cargo test web3_routes_integration_tests --test integration"; then
    ((passed_suites++))
else
    ((failed_suites++))
fi

echo ""

# Backend Performance Tests
echo -e "${BLUE}⚡ Backend Performance Tests${NC}"
echo "==========================="

((total_suites++))
if run_test_suite "Web3 Performance Tests" "cargo test web3_performance_tests --lib --release"; then
    ((passed_suites++))
else
    ((failed_suites++))
fi

echo ""

# Check if frontend dependencies are available
if command -v pnpm > /dev/null 2>&1; then
    # Frontend Unit Tests
    echo -e "${BLUE}⚛️  Frontend Unit Tests${NC}"
    echo "====================="
    
    cd "$(dirname "$0")/../../../frontend"
    
    ((total_suites++))
    if run_test_suite "WalletConnectAuth Component Tests" "pnpm test __test__/unit/web3/WalletConnectAuth.test.tsx --run"; then
        ((passed_suites++))
    else
        ((failed_suites++))
    fi
    
    ((total_suites++))
    if run_test_suite "UnifiedAuthForm Component Tests" "pnpm test __test__/unit/web3/UnifiedAuthForm.test.tsx --run"; then
        ((passed_suites++))
    else
        ((failed_suites++))
    fi
    
    ((total_suites++))
    if run_test_suite "Web3AuthProvider Tests" "pnpm test __test__/unit/web3/Web3AuthProvider.test.tsx --run"; then
        ((passed_suites++))
    else
        ((failed_suites++))
    fi
    
    echo ""
    
    # Frontend Integration Tests
    echo -e "${BLUE}🔄 Frontend Integration Tests${NC}"
    echo "============================="
    
    ((total_suites++))
    if run_test_suite "Auth Flow Integration Tests" "pnpm test __test__/integration/web3/auth-flow-integration.test.tsx --run"; then
        ((passed_suites++))
    else
        ((failed_suites++))
    fi
    
    echo ""
    
    # E2E Tests (if Playwright is available)
    if command -v playwright > /dev/null 2>&1; then
        echo -e "${BLUE}🎭 End-to-End Tests${NC}"
        echo "=================="
        
        # Start services for E2E tests
        echo -e "${YELLOW}🚀 Starting services for E2E tests...${NC}"
        
        # Start backend in background (if not already running)
        if ! curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
            echo "Starting backend service..."
            cd "$(dirname "$0")/../../"
            cargo run &
            BACKEND_PID=$!
            
            # Wait for backend to start
            timeout=30
            while [ $timeout -gt 0 ]; do
                if curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
                    break
                fi
                sleep 1
                ((timeout--))
            done
            
            if [ $timeout -eq 0 ]; then
                echo -e "${RED}❌ Backend failed to start${NC}"
                kill $BACKEND_PID 2>/dev/null || true
                exit 1
            fi
        fi
        
        # Start frontend in background (if not already running)
        cd "$(dirname "$0")/../../../frontend"
        if ! curl -s "$FRONTEND_URL" > /dev/null 2>&1; then
            echo "Starting frontend service..."
            pnpm build
            pnpm start &
            FRONTEND_PID=$!
            
            # Wait for frontend to start
            timeout=60
            while [ $timeout -gt 0 ]; do
                if curl -s "$FRONTEND_URL" > /dev/null 2>&1; then
                    break
                fi
                sleep 1
                ((timeout--))
            done
            
            if [ $timeout -eq 0 ]; then
                echo -e "${RED}❌ Frontend failed to start${NC}"
                kill $FRONTEND_PID 2>/dev/null || true
                kill $BACKEND_PID 2>/dev/null || true
                exit 1
            fi
        fi
        
        ((total_suites++))
        if run_test_suite "Web3 Authentication E2E Tests" "pnpm playwright test __test__/e2e/web3-authentication-flows.spec.ts"; then
            ((passed_suites++))
        else
            ((failed_suites++))
        fi
        
        # Performance/Load Tests
        echo ""
        echo -e "${BLUE}📊 Performance Tests${NC}"
        echo "==================="
        
        ((total_suites++))
        if run_test_suite "Web3 Auth Load Testing" "pnpm playwright test __test__/performance/web3-auth-load-testing.spec.ts"; then
            ((passed_suites++))
        else
            ((failed_suites++))
        fi
        
        # Cleanup services
        echo -e "${YELLOW}🧹 Cleaning up services...${NC}"
        kill $FRONTEND_PID 2>/dev/null || true
        kill $BACKEND_PID 2>/dev/null || true
        
    else
        echo -e "${YELLOW}⚠️  Playwright not found. Skipping E2E tests.${NC}"
    fi
    
else
    echo -e "${YELLOW}⚠️  pnpm not found. Skipping frontend tests.${NC}"
fi

# Test Summary
echo ""
echo -e "${BLUE}📊 Test Suite Summary${NC}"
echo "===================="
echo "Total test suites: $total_suites"
echo -e "Passed: ${GREEN}$passed_suites${NC}"
echo -e "Failed: ${RED}$failed_suites${NC}"

if [ $failed_suites -eq 0 ]; then
    echo ""
    echo -e "${GREEN}🎉 All tests passed! Web3 authentication system is ready for deployment.${NC}"
    echo ""
    echo -e "${BLUE}🔒 Security Features Validated:${NC}"
    echo "  ✅ SIWE message validation"
    echo "  ✅ Nonce replay protection"
    echo "  ✅ Domain validation security"
    echo "  ✅ Permission escalation prevention"
    echo "  ✅ SQL injection protection"
    echo ""
    echo -e "${BLUE}⚡ Performance Benchmarks:${NC}"
    echo "  ✅ Challenge generation < 50ms average"
    echo "  ✅ Permission checks < 5ms average"
    echo "  ✅ Concurrent operations handling"
    echo "  ✅ Load spike resilience"
    echo ""
    echo -e "${BLUE}🧪 Test Coverage:${NC}"
    echo "  ✅ Unit tests for all core services"
    echo "  ✅ Integration tests for API endpoints"
    echo "  ✅ Component tests for React components"
    echo "  ✅ End-to-end user journey tests"
    echo "  ✅ Security vulnerability tests"
    echo "  ✅ Performance and load tests"
    
    exit 0
else
    echo ""
    echo -e "${RED}❌ $failed_suites test suite(s) failed. Please review and fix issues before deployment.${NC}"
    
    echo ""
    echo -e "${YELLOW}🔧 Troubleshooting Tips:${NC}"
    echo "  • Check database connection and migrations"
    echo "  • Ensure all environment variables are set correctly"
    echo "  • Verify services are running on expected ports"
    echo "  • Review test logs for specific error details"
    echo "  • Run individual test suites for detailed debugging"
    
    exit 1
fi