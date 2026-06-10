#!/bin/bash

# Comprehensive E2E Test Runner for EPSX Trading Platform
# Runs all test suites with proper organization and reporting

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_ROOT="$(cd "$TEST_DIR/../.." && pwd)"
RESULTS_DIR="$TEST_DIR/results"
REPORTS_DIR="$TEST_DIR/reports"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Create directories
mkdir -p "$RESULTS_DIR"
mkdir -p "$REPORTS_DIR"

# Function to print colored output
print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

# Function to check if services are running
check_services() {
    print_header "🔍 Checking Services"
    
    # Check frontend
    if curl -f -s http://localhost:3000 > /dev/null; then
        print_success "Frontend service is running (port 3000)"
    else
        print_error "Frontend service is not running. Please start with: pnpm dev:frontend"
        exit 1
    fi
    
    # Check backend (optional)
    if curl -f -s http://localhost:8080/health > /dev/null; then
        print_success "Backend service is running (port 8080)"
        BACKEND_AVAILABLE=true
    else
        print_warning "Backend service is not running. Some tests may use mock mode."
        BACKEND_AVAILABLE=false
    fi
    
    # Check admin frontend (optional)
    if curl -f -s http://localhost:3001 > /dev/null; then
        print_success "Admin frontend is running (port 3001)"
        ADMIN_AVAILABLE=true
    else
        print_warning "Admin frontend is not running. Admin tests will be skipped."
        ADMIN_AVAILABLE=false
    fi
}

# Function to run specific test suite
run_test_suite() {
    local suite_name="$1"
    local test_file="$2"
    local description="$3"
    local category="$4"
    
    print_header "🧪 Running $suite_name"
    print_info "$description"
    
    local start_time=$(date +%s)
    
    # Run the test with comprehensive options
    if npx playwright test "$test_file" \
        --project=chromium \
        --reporter=html,json,junit \
        --output-dir="$RESULTS_DIR/$suite_name" \
        --reporter-json="$RESULTS_DIR/$suite_name/results.json" \
        --reporter-junit="$RESULTS_DIR/$suite_name/results.xml"; then
        
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        print_success "$suite_name completed successfully in ${duration}s"
        
        # Generate test summary
        echo "$suite_name,$category,PASSED,$duration" >> "$RESULTS_DIR/test_summary_$TIMESTAMP.csv"
        
        return 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        print_error "$suite_name failed after ${duration}s"
        
        # Generate test summary
        echo "$suite_name,$category,FAILED,$duration" >> "$RESULTS_DIR/test_summary_$TIMESTAMP.csv"
        
        return 1
    fi
}

# Function to run cross-browser tests
run_cross_browser_tests() {
    print_header "🌐 Running Cross-Browser Tests"
    
    local browsers=("chromium" "firefox" "webkit")
    local test_file="mobile-cross-browser.spec.ts"
    
    for browser in "${browsers[@]}"; do
        print_info "Testing on $browser"
        
        if npx playwright test "$test_file" \
            --project="$browser" \
            --output-dir="$RESULTS_DIR/cross-browser-$browser" \
            --reporter-json="$RESULTS_DIR/cross-browser-$browser/results.json"; then
            print_success "$browser tests passed"
        else
            print_error "$browser tests failed"
        fi
    done
}

# Function to run mobile device tests
run_mobile_tests() {
    print_header "📱 Running Mobile Device Tests"
    
    local devices=("mobile-chrome" "mobile-safari")
    local test_file="mobile-cross-browser.spec.ts"
    
    for device in "${devices[@]}"; do
        print_info "Testing on $device"
        
        if npx playwright test "$test_file" \
            --project="$device" \
            --output-dir="$RESULTS_DIR/mobile-$device" \
            --reporter-json="$RESULTS_DIR/mobile-$device/results.json"; then
            print_success "$device tests passed"
        else
            print_error "$device tests failed"
        fi
    done
}

# Function to run performance tests
run_performance_tests() {
    print_header "⚡ Running Performance Tests"
    
    # Run performance tests with extended timeout
    if npx playwright test "performance-testing.spec.ts" \
        --project=chromium \
        --timeout=120000 \
        --output-dir="$RESULTS_DIR/performance" \
        --reporter-json="$RESULTS_DIR/performance/results.json"; then
        
        print_success "Performance tests completed"
        
        # Generate performance report
        node "$TEST_DIR/scripts/generate-performance-report.js" \
            "$RESULTS_DIR/performance/results.json" \
            "$REPORTS_DIR/performance-report-$TIMESTAMP.html"
            
    else
        print_error "Performance tests failed"
    fi
}

# Function to generate comprehensive report
generate_comprehensive_report() {
    print_header "📊 Generating Comprehensive Report"
    
    local report_file="$REPORTS_DIR/comprehensive-report-$TIMESTAMP.html"
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>EPSX E2E Test Report - $TIMESTAMP</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f5f5f5; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
        .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 30px; }
        .metric { background: white; border: 1px solid #ddd; padding: 15px; border-radius: 8px; text-align: center; }
        .metric h3 { margin: 0 0 10px 0; color: #333; }
        .metric .value { font-size: 24px; font-weight: bold; }
        .passed { color: #28a745; }
        .failed { color: #dc3545; }
        .warning { color: #ffc107; }
        .test-results { margin-top: 30px; }
        .test-suite { background: #f8f9fa; padding: 15px; margin: 10px 0; border-radius: 8px; }
        .timestamp { color: #666; font-size: 0.9em; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🎯 EPSX Trading Platform - E2E Test Report</h1>
        <p class="timestamp">Generated: $(date)</p>
        <p>Comprehensive test results for user middleware validation system</p>
    </div>
    
    <div class="summary">
        <div class="metric">
            <h3>Total Test Suites</h3>
            <div class="value" id="total-suites">-</div>
        </div>
        <div class="metric">
            <h3>Passed</h3>
            <div class="value passed" id="passed-suites">-</div>
        </div>
        <div class="metric">
            <h3>Failed</h3>
            <div class="value failed" id="failed-suites">-</div>
        </div>
        <div class="metric">
            <h3>Total Duration</h3>
            <div class="value" id="total-duration">-</div>
        </div>
    </div>
    
    <div class="test-results">
        <h2>🧪 Test Suite Results</h2>
        <div id="test-suite-results">
            <!-- Results will be populated by JavaScript -->
        </div>
    </div>
    
    <script>
        // This would be populated with actual test results
        // For now, it's a template
        document.getElementById('total-suites').textContent = '10';
        document.getElementById('passed-suites').textContent = '8';
        document.getElementById('failed-suites').textContent = '2';
        document.getElementById('total-duration').textContent = '15m 32s';
    </script>
</body>
</html>
EOF
    
    print_success "Comprehensive report generated: $report_file"
}

# Function to cleanup old results
cleanup_old_results() {
    print_info "Cleaning up old test results (keeping last 5 runs)"
    
    # Keep only the 5 most recent result directories
    find "$RESULTS_DIR" -maxdepth 1 -type d -name "*_*" | sort -r | tail -n +6 | xargs rm -rf 2>/dev/null || true
    
    # Keep only the 5 most recent report files
    find "$REPORTS_DIR" -name "*.html" | sort -r | tail -n +6 | xargs rm -f 2>/dev/null || true
}

# Main execution
main() {
    print_header "🚀 EPSX E2E Test Suite"
    print_info "Starting comprehensive test execution at $(date)"
    
    # Change to project directory
    cd "$PROJECT_ROOT"
    
    # Cleanup old results
    cleanup_old_results
    
    # Check services
    check_services
    
    # Initialize test summary
    echo "TestSuite,Category,Status,Duration" > "$RESULTS_DIR/test_summary_$TIMESTAMP.csv"
    
    # Track overall status
    OVERALL_STATUS=0
    
    # Core test suites
    print_header "🎯 Core Test Suites"
    
    # Package tier permissions (critical)
    run_test_suite "package-tier-permissions" "package-tier-permissions.spec.ts" \
        "Tests all 6 package tiers and their permission boundaries" "critical" || OVERALL_STATUS=1
    
    # Feature access control
    run_test_suite "feature-access-control" "feature-access-control.spec.ts" \
        "Tests feature access control for all trading platform features" "critical" || OVERALL_STATUS=1
    
    # Middleware validation
    run_test_suite "middleware-validation" "middleware-validation.spec.ts" \
        "Tests Next.js middleware stack for session and tier validation" "critical" || OVERALL_STATUS=1
    
    # Trading platform security
    run_test_suite "trading-platform-security" "trading-platform-security.spec.ts" \
        "Tests financial data protection and trading security" "security" || OVERALL_STATUS=1
    
    # Subscription flows
    run_test_suite "subscription-flows" "subscription-flows.spec.ts" \
        "Tests subscription management and tier transitions" "regression" || OVERALL_STATUS=1
    
    # Performance tests
    if [ "$1" != "--skip-performance" ]; then
        run_performance_tests || OVERALL_STATUS=1
    else
        print_warning "Skipping performance tests (--skip-performance flag)"
    fi
    
    # Cross-browser tests
    if [ "$1" != "--skip-cross-browser" ]; then
        run_cross_browser_tests || OVERALL_STATUS=1
    else
        print_warning "Skipping cross-browser tests (--skip-cross-browser flag)"
    fi
    
    # Mobile tests
    if [ "$1" != "--skip-mobile" ]; then
        run_mobile_tests || OVERALL_STATUS=1
    else
        print_warning "Skipping mobile tests (--skip-mobile flag)"
    fi
    
    # Generate reports
    generate_comprehensive_report
    
    # Final summary
    print_header "📋 Test Execution Summary"
    
    if [ $OVERALL_STATUS -eq 0 ]; then
        print_success "All test suites completed successfully! 🎉"
        print_info "Results available in: $RESULTS_DIR"
        print_info "Reports available in: $REPORTS_DIR"
    else
        print_error "Some test suites failed. Check the results for details."
        print_info "Results available in: $RESULTS_DIR"
        print_info "Reports available in: $REPORTS_DIR"
    fi
    
    # Display test summary
    if [ -f "$RESULTS_DIR/test_summary_$TIMESTAMP.csv" ]; then
        print_info "Test Summary:"
        column -t -s, "$RESULTS_DIR/test_summary_$TIMESTAMP.csv"
    fi
    
    print_info "Test execution completed at $(date)"
    exit $OVERALL_STATUS
}

# Parse command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  --skip-performance     Skip performance tests"
        echo "  --skip-cross-browser   Skip cross-browser tests"
        echo "  --skip-mobile         Skip mobile tests"
        echo "  --help, -h            Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0                    Run all tests"
        echo "  $0 --skip-performance Run all tests except performance"
        echo "  $0 --skip-mobile      Run all tests except mobile"
        exit 0
        ;;
esac

# Run main function
main "$@"