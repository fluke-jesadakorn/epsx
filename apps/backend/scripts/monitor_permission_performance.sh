#!/bin/bash

# ================================================================================================
# PERFORMANCE MONITORING SCRIPT - UNIFIED PERMISSION SYSTEM
# ================================================================================================
# This script provides real-time performance monitoring for the unified permission system.
# It tracks key metrics, generates reports, and provides alerts for performance issues.
#
# Features:
# - Real-time performance monitoring
# - Automated performance testing
# - Performance trend analysis
# - Alert generation for degradation
# - Integration with monitoring systems
# ================================================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
BACKEND_URL="${BACKEND_URL:-https://backend-307278481624.us-central1.run.app}"
TEST_WALLET="0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
TEST_PERMISSIONS=(
    "admin:users:manage"
    "admin:permissions:view"
    "trading:orders:execute"
    "trading:portfolio:view"
    "notifications:send:email"
)

# Performance thresholds
LATENCY_WARNING=5      # ms
LATENCY_CRITICAL=10    # ms
CACHE_HIT_WARNING=70   # %
CACHE_HIT_CRITICAL=50  # %
THROUGHPUT_WARNING=100 # requests/sec
THROUGHPUT_CRITICAL=50 # requests/sec

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_perf() {
    echo -e "${PURPLE}[PERF]${NC} $1"
}

# Performance testing function
test_permission_validation() {
    local permission="$1"
    local iterations="${2:-100}"

    log_perf "Testing permission validation: $permission ($iterations iterations)"

    local total_time=0
    local success_count=0
    local error_count=0

    for ((i=1; i<=iterations; i++)); do
        local start_time=$(date +%s%N)

        local response=$(curl -s -w "%{http_code}" -X POST "$BACKEND_URL/api/v1/permissions/validate" \
            -H "Content-Type: application/json" \
            -d "{\"wallet_address\":\"$TEST_WALLET\",\"permission\":\"$permission\"}" 2>/dev/null)

        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 )) # Convert to ms

        local http_code="${response: -3}"
        local body="${response%???}"

        if [[ "$http_code" == "200" ]]; then
            ((success_count++))
            total_time=$((total_time + duration))
        else
            ((error_count++))
            log_error "Request failed with status $http_code"
        fi

        # Progress indicator
        if (( i % 20 == 0 )); then
            echo -n "."
        fi
    done

    echo "" # New line after progress indicator

    local avg_latency=$((total_time / success_count))
    local success_rate=$(( (success_count * 100) / iterations ))

    echo "  - Average Latency: ${avg_latency}ms"
    echo "  - Success Rate: ${success_rate}%"
    echo "  - Error Count: $error_count"

    # Performance evaluation
    if (( avg_latency > LATENCY_CRITICAL )); then
        log_error "❌ CRITICAL: Latency exceeds threshold (${avg_latency}ms > ${LATENCY_CRITICAL}ms)"
        return 2
    elif (( avg_latency > LATENCY_WARNING )); then
        log_warning "⚠️ WARNING: Latency elevated (${avg_latency}ms > ${LATENCY_WARNING}ms)"
        return 1
    else
        log_success "✅ OK: Latency within acceptable range (${avg_latency}ms)"
        return 0
    fi
}

# Cache hit rate testing
test_cache_performance() {
    log_perf "Testing cache performance..."

    # First request (cache miss)
    local start_time=$(date +%s%N)
    curl -s -X POST "$BACKEND_URL/api/v1/permissions/validate" \
        -H "Content-Type: application/json" \
        -d "{\"wallet_address\":\"$TEST_WALLET\",\"permission\":\"${TEST_PERMISSIONS[0]}\"}" > /dev/null
    local first_request_time=$(( ( $(date +%s%N) - start_time) / 1000000 ))

    # Subsequent requests (cache hits)
    local total_cache_time=0
    local cache_tests=10

    for ((i=1; i<=cache_tests; i++)); do
        start_time=$(date +%s%N)
        curl -s -X POST "$BACKEND_URL/api/v1/permissions/validate" \
            -H "Content-Type: application/json" \
            -d "{\"wallet_address\":\"$TEST_WALLET\",\"permission\":\"${TEST_PERMISSIONS[0]}\"}" > /dev/null
        total_cache_time=$((total_cache_time + ( ( $(date +%s%N) - start_time) / 1000000 )))
    done

    local avg_cache_time=$((total_cache_time / cache_tests))
    local cache_improvement=$(( ( (first_request_time - avg_cache_time) * 100 ) / first_request_time ))

    echo "  - First Request: ${first_request_time}ms (cache miss)"
    echo "  - Cached Request Average: ${avg_cache_time}ms"
    echo "  - Cache Improvement: ${cache_improvement}%"

    if (( cache_improvement < 20 )); then
        log_warning "⚠️ WARNING: Cache improvement low (${cache_improvement}%)"
        return 1
    else
        log_success "✅ OK: Cache performance good (${cache_improvement}% improvement)"
        return 0
    fi
}

# Throughput testing
test_throughput() {
    local duration="${1:-30}" # seconds
    log_perf "Testing throughput for ${duration} seconds..."

    local start_timestamp=$(date +%s)
    local end_timestamp=$((start_timestamp + duration))
    local total_requests=0
    local successful_requests=0

    while (( $(date +%s) < end_timestamp )); do
        for permission in "${TEST_PERMISSIONS[@]}"; do
            curl -s -X POST "$BACKEND_URL/api/v1/permissions/validate" \
                -H "Content-Type: application/json" \
                -d "{\"wallet_address\":\"$TEST_WALLET\",\"permission\":\"$permission\"}" > /dev/null &
            ((total_requests++))
        done
        wait # Wait for background requests
    done

    local actual_duration=$(( $(date +%s) - start_timestamp ))
    local throughput=$((total_requests / actual_duration))

    echo "  - Total Requests: $total_requests"
    echo "  - Duration: ${actual_duration}s"
    echo "  - Throughput: ${throughput} requests/sec"

    if (( throughput < THROUGHPUT_CRITICAL )); then
        log_error "❌ CRITICAL: Throughput too low (${throughput} < ${THROUGHPUT_CRITICAL})"
        return 2
    elif (( throughput < THROUGHPUT_WARNING )); then
        log_warning "⚠️ WARNING: Throughput low (${throughput} < ${THROUGHPUT_WARNING})"
        return 1
    else
        log_success "✅ OK: Throughput acceptable (${throughput} req/sec)"
        return 0
    fi
}

# Get system metrics
get_system_metrics() {
    log_perf "Retrieving system metrics..."

    # Health check
    local health_status=$(curl -s "$BACKEND_URL/health" | jq -r '.status // "unknown"' 2>/dev/null || echo "failed")
    echo "  - Health Status: $health_status"

    # Metrics from backend (if available)
    if curl -s "$BACKEND_URL/metrics" > /dev/null 2>&1; then
        echo "  - Metrics Endpoint: Available"

        # Extract key metrics
        local validation_latency=$(curl -s "$BACKEND_URL/metrics" | grep "permission_validation_duration_seconds" | head -1 | awk '{print $2}' || echo "N/A")
        local cache_hit_rate=$(curl -s "$BACKEND_URL/metrics" | grep "permission_cache_hit_rate" | awk '{print $2}' || echo "N/A")

        if [[ "$validation_latency" != "N/A" ]]; then
            echo "  - Reported Latency: ${validation_latency}s"
        fi

        if [[ "$cache_hit_rate" != "N/A" ]]; then
            local cache_percentage=$((cache_hit_rate * 100))
            echo "  - Reported Cache Hit Rate: ${cache_percentage}%"
        fi
    else
        echo "  - Metrics Endpoint: Not available"
    fi
}

# Generate performance report
generate_performance_report() {
    local report_file="performance_report_$(date +%Y%m%d_%H%M%S).json"

    log_info "Generating performance report: $report_file"

    local report=$(cat << EOF
{
  "timestamp": "$(date -Iseconds)",
  "backend_url": "$BACKEND_URL",
  "test_wallet": "$TEST_WALLET",
  "performance_tests": {
    "permission_validation": {
      "latency_warning_ms": $LATENCY_WARNING,
      "latency_critical_ms": $LATENCY_CRITICAL,
      "tested_permissions": $(printf '%s\n' "${TEST_PERMISSIONS[@]}" | jq -R . | jq -s .)
    },
    "cache_performance": {
      "improvement_threshold_percent": 20
    },
    "throughput": {
      "warning_threshold_rps": $THROUGHPUT_WARNING,
      "critical_threshold_rps": $THROUGHPUT_CRITICAL
    }
  },
  "system_health": {
    "backend_url": "$BACKEND_URL",
    "health_endpoint": "$BACKEND_URL/health",
    "metrics_endpoint": "$BACKEND_URL/metrics"
  }
}
EOF
)

    echo "$report" > "$report_file"
    log_success "✅ Performance report saved to: $report_file"
}

# Continuous monitoring mode
continuous_monitoring() {
    local interval="${1:-60}" # seconds

    log_info "Starting continuous monitoring (interval: ${interval}s)"
    log_info "Press Ctrl+C to stop monitoring"

    while true; do
        echo ""
        log_perf "=== Performance Check - $(date) ==="

        # Health check
        local health_status=$(curl -s "$BACKEND_URL/health" | jq -r '.status // "unknown"' 2>/dev/null || echo "failed")
        if [[ "$health_status" != "healthy" ]]; then
            log_error "❌ Backend health check failed (status: $health_status)"
        else
            log_success "✅ Backend healthy"
        fi

        # Quick performance test
        test_permission_validation "${TEST_PERMISSIONS[0]}" 10

        sleep "$interval"
    done
}

# Performance alerting
check_performance_alerts() {
    log_perf "Checking for performance alerts..."

    local alerts_count=0

    # Test critical permission
    test_permission_validation "${TEST_PERMISSIONS[0]}" 50
    local latency_test_result=$?

    if (( latency_test_result >= 2 )); then
        echo "🚨 ALERT: Critical latency detected"
        ((alerts_count++))
    fi

    # Check cache performance
    test_cache_performance
    local cache_test_result=$?

    if (( cache_test_result >= 1 )); then
        echo "⚠️ ALERT: Cache performance degraded"
        ((alerts_count++))
    fi

    if (( alerts_count == 0 )); then
        log_success "✅ No performance alerts"
    else
        log_warning "⚠️ $alerts_count performance alert(s) detected"
    fi

    return $alerts_count
}

# Main function
main() {
    local mode="${1:-test}"

    log_info "=== EPSX PERMISSION SYSTEM PERFORMANCE MONITORING ==="
    log_info "Backend URL: $BACKEND_URL"
    log_info "Test Wallet: $TEST_WALLET"
    log_info "Mode: $mode"

    case "$mode" in
        "test")
            log_info "Running comprehensive performance tests..."

            echo ""
            test_permission_validation "${TEST_PERMISSIONS[0]}" 100
            echo ""

            test_cache_performance
            echo ""

            test_throughput 30
            echo ""

            get_system_metrics
            echo ""

            generate_performance_report
            ;;

        "quick")
            log_info "Running quick performance check..."
            test_permission_validation "${TEST_PERMISSIONS[0]}" 50
            check_performance_alerts
            ;;

        "monitor")
            local interval="${2:-60}"
            continuous_monitoring "$interval"
            ;;

        "report")
            generate_performance_report
            ;;

        "help")
            echo "Usage: $0 {test|quick|monitor|report|help} [interval]"
            echo ""
            echo "Modes:"
            echo "  test     - Run comprehensive performance tests"
            echo "  quick    - Run quick performance check with alerts"
            echo "  monitor  - Start continuous monitoring mode"
            echo "  report   - Generate performance report only"
            echo "  help     - Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 test                    # Full performance test suite"
            echo "  $0 quick                   # Quick check with alerts"
            echo "  $0 monitor 30              # Monitor every 30 seconds"
            echo "  $0 report                  # Generate report only"
            ;;

        *)
            log_error "Unknown mode: $mode"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
}

# Execute main function with all arguments
main "$@"