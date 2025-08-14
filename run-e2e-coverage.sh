#!/bin/bash

# Complete E2E Test Coverage Runner
# Runs comprehensive E2E tests across all applications

set -e

echo "🚀 Starting Complete E2E Test Coverage Suite"
echo "============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if applications are running
check_app_status() {
    local app_name=$1
    local port=$2
    local url="http://localhost:${port}"
    
    print_status "Checking ${app_name} on port ${port}..."
    
    if curl -s -f "${url}" > /dev/null 2>&1; then
        print_success "${app_name} is running on port ${port}"
        return 0
    else
        print_warning "${app_name} is not running on port ${port}"
        return 1
    fi
}

# Check if backend is running
check_backend_status() {
    print_status "Checking backend on port 8080..."
    
    if curl -s -f "http://localhost:8080/health" > /dev/null 2>&1 || \
       curl -s -f "http://localhost:8080/" > /dev/null 2>&1; then
        print_success "Backend is running on port 8080"
        return 0
    else
        print_warning "Backend is not running on port 8080"
        return 1
    fi
}

# Create test results directories
create_test_dirs() {
    print_status "Creating test result directories..."
    
    mkdir -p apps/frontend/test-results
    mkdir -p apps/frontend/playwright-report
    mkdir -p apps/admin-frontend/test-results
    mkdir -p apps/admin-frontend/playwright-report
    
    print_success "Test directories created"
}

# Run frontend E2E tests
run_frontend_tests() {
    print_status "Running Frontend E2E Tests..."
    echo "======================================"
    
    cd apps/frontend
    
    # Run different test suites
    echo "🧪 Running Core Frontend Tests..."
    if npx playwright test --project=core; then
        print_success "Core frontend tests passed"
    else
        print_error "Core frontend tests failed"
    fi
    
    echo "🧪 Running Complete Coverage Tests..."
    if npx playwright test --project=coverage; then
        print_success "Complete coverage tests passed"
    else
        print_error "Complete coverage tests failed"
    fi
    
    echo "🧪 Running Enhanced Auth Flow Tests..."
    if npx playwright test --project=auth-enhanced; then
        print_success "Enhanced auth tests passed"
    else
        print_error "Enhanced auth tests failed"
    fi
    
    echo "🧪 Running User Journey Tests..."
    if npx playwright test --project=journeys; then
        print_success "User journey tests passed"
    else
        print_error "User journey tests failed"
    fi
    
    echo "🧪 Running Cross-Browser Tests..."
    if npx playwright test --project=firefox --project=webkit; then
        print_success "Cross-browser tests passed"
    else
        print_warning "Some cross-browser tests failed"
    fi
    
    echo "📱 Running Mobile Tests..."
    if npx playwright test --project=mobile-chrome --project=mobile-safari; then
        print_success "Mobile tests passed"
    else
        print_warning "Some mobile tests failed"
    fi
    
    cd ../..
}

# Run admin frontend E2E tests
run_admin_tests() {
    print_status "Running Admin Frontend E2E Tests..."
    echo "========================================="
    
    cd apps/admin-frontend
    
    echo "🧪 Running Core Admin Tests..."
    if npx playwright test --project=admin-core; then
        print_success "Core admin tests passed"
    else
        print_error "Core admin tests failed"
    fi
    
    echo "🧪 Running Complete Admin Coverage Tests..."
    if npx playwright test --project=admin-coverage; then
        print_success "Complete admin coverage tests passed"
    else
        print_error "Complete admin coverage tests failed"
    fi
    
    echo "👥 Running User Management Tests..."
    if npx playwright test --project=user-management; then
        print_success "User management tests passed"
    else
        print_error "User management tests failed"
    fi
    
    echo "🔐 Running Permission Management Tests..."
    if npx playwright test --project=permission-management; then
        print_success "Permission management tests passed"
    else
        print_error "Permission management tests failed"
    fi
    
    echo "🛠️ Running System Admin Tests..."
    if npx playwright test --project=system-admin; then
        print_success "System admin tests passed"
    else
        print_error "System admin tests failed"
    fi
    
    echo "🧪 Running Cross-Browser Admin Tests..."
    if npx playwright test --project=admin-firefox --project=admin-webkit; then
        print_success "Cross-browser admin tests passed"
    else
        print_warning "Some cross-browser admin tests failed"
    fi
    
    echo "📱 Running Mobile Admin Tests..."
    if npx playwright test --project=admin-mobile-chrome --project=admin-mobile-safari; then
        print_success "Mobile admin tests passed"
    else
        print_warning "Some mobile admin tests failed"
    fi
    
    cd ../..
}

# Generate coverage reports
generate_reports() {
    print_status "Generating Test Coverage Reports..."
    echo "====================================="
    
    # Frontend reports
    if [ -f "apps/frontend/test-results/results.json" ]; then
        print_success "Frontend test results generated"
        echo "📊 Frontend Report: apps/frontend/playwright-report/index.html"
    fi
    
    # Admin reports
    if [ -f "apps/admin-frontend/test-results/results.json" ]; then
        print_success "Admin frontend test results generated"
        echo "📊 Admin Report: apps/admin-frontend/playwright-report/index.html"
    fi
    
    # Combined summary
    echo ""
    echo "📋 Test Summary:"
    echo "=================="
    
    # Count test files
    frontend_tests=$(find apps/frontend/__test__/e2e -name "*.spec.ts" | wc -l)
    admin_tests=$(find apps/admin-frontend/__test__/e2e -name "*.spec.ts" | wc -l)
    total_tests=$((frontend_tests + admin_tests))
    
    echo "📁 Frontend test files: ${frontend_tests}"
    echo "📁 Admin test files: ${admin_tests}"
    echo "📁 Total test files: ${total_tests}"
    
    # Count pages covered
    frontend_pages=$(grep -r "path:" apps/frontend/__test__/e2e/ | wc -l)
    admin_pages=$(grep -r "path:" apps/admin-frontend/__test__/e2e/ | wc -l)
    total_pages=$((frontend_pages + admin_pages))
    
    echo "📄 Frontend pages tested: ~${frontend_pages}"
    echo "📄 Admin pages tested: ~${admin_pages}"
    echo "📄 Total pages covered: ~${total_pages}"
}

# Main execution
main() {
    echo "🎯 E2E Test Coverage Analysis Starting..."
    echo "Current directory: $(pwd)"
    echo "Timestamp: $(date)"
    echo ""
    
    # Create test directories
    create_test_dirs
    
    # Check application status
    echo "🔍 Checking Application Status..."
    echo "=================================="
    
    FRONTEND_RUNNING=0
    ADMIN_RUNNING=0
    BACKEND_RUNNING=0
    
    if check_app_status "Frontend" "3000"; then
        FRONTEND_RUNNING=1
    fi
    
    if check_app_status "Admin Frontend" "3001"; then
        ADMIN_RUNNING=1
    fi
    
    if check_backend_status; then
        BACKEND_RUNNING=1
    fi
    
    echo ""
    
    # Run tests based on what's available
    if [ $BACKEND_RUNNING -eq 0 ]; then
        print_error "Backend is not running. E2E tests require backend for OAuth."
        print_warning "Please start the backend with: pnpm dev:backend"
        exit 1
    fi
    
    if [ $FRONTEND_RUNNING -eq 1 ]; then
        run_frontend_tests
    else
        print_warning "Frontend not running - skipping frontend tests"
        print_warning "Start frontend with: pnpm dev:frontend"
    fi
    
    echo ""
    
    if [ $ADMIN_RUNNING -eq 1 ]; then
        run_admin_tests
    else
        print_warning "Admin frontend not running - skipping admin tests"
        print_warning "Start admin with: pnpm dev:admin"
    fi
    
    echo ""
    
    # Generate reports
    generate_reports
    
    echo ""
    print_success "🎉 E2E Test Coverage Suite Completed!"
    echo ""
    echo "📊 View Reports:"
    echo "  Frontend: file://$(pwd)/apps/frontend/playwright-report/index.html"
    echo "  Admin:    file://$(pwd)/apps/admin-frontend/playwright-report/index.html"
    echo ""
    echo "🚀 To run specific test suites:"
    echo "  Frontend Coverage: cd apps/frontend && npx playwright test --project=coverage"
    echo "  Admin Coverage:    cd apps/admin-frontend && npx playwright test --project=admin-coverage"
    echo "  Mobile Tests:      npx playwright test --project=mobile-chrome"
    echo "  Cross-Browser:     npx playwright test --project=firefox --project=webkit"
}

# Handle script interruption
trap 'print_error "Test run interrupted by user"; exit 1' INT

# Run main function
main "$@"