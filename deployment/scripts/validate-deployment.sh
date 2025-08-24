#!/bin/bash

# EPSX Production Deployment Validation Script
# Comprehensive validation of security, performance, and functionality

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
ENVIRONMENT=${1:-production}
CONFIG_FILE="deployment/environments/${ENVIRONMENT}.env"
VALIDATION_TIMEOUT=300
MAX_RETRIES=5

# Load environment configuration
if [[ -f "$CONFIG_FILE" ]]; then
    source "$CONFIG_FILE"
else
    echo -e "${RED}❌ Environment configuration file not found: $CONFIG_FILE${NC}"
    exit 1
fi

echo -e "${BLUE}🔍 EPSX Deployment Validation - $ENVIRONMENT${NC}"
echo -e "${BLUE}========================================${NC}"

# Validation results tracking
VALIDATION_RESULTS=()
FAILED_CHECKS=0
TOTAL_CHECKS=0

# Function to log validation results
log_result() {
    local status=$1
    local check_name=$2
    local message=$3
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    if [[ "$status" == "PASS" ]]; then
        echo -e "${GREEN}✅ $check_name: $message${NC}"
        VALIDATION_RESULTS+=("PASS: $check_name - $message")
    else
        echo -e "${RED}❌ $check_name: $message${NC}"
        VALIDATION_RESULTS+=("FAIL: $check_name - $message")
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    fi
}

# Function to make HTTP requests with retry
http_request() {
    local url=$1
    local expected_status=${2:-200}
    local timeout=${3:-10}
    local retries=${4:-3}
    
    for i in $(seq 1 $retries); do
        if response=$(curl -s -o /dev/null -w "%{http_code}" --max-time $timeout "$url" 2>/dev/null); then
            if [[ "$response" == "$expected_status" ]]; then
                return 0
            fi
        fi
        sleep 2
    done
    return 1
}

# Function to check JSON API response
check_json_api() {
    local url=$1
    local timeout=${2:-10}
    local retries=${3:-3}
    
    for i in $(seq 1 $retries); do
        if response=$(curl -s --max-time $timeout "$url" 2>/dev/null); then
            if echo "$response" | jq . >/dev/null 2>&1; then
                echo "$response"
                return 0
            fi
        fi
        sleep 2
    done
    return 1
}

echo -e "\n${PURPLE}=== 1. INFRASTRUCTURE VALIDATION ===${NC}"

# 1.1 Service Health Checks
echo -e "\n${YELLOW}1.1 Service Health Checks${NC}"

# Backend health check
if http_request "$BACKEND_URL/health" 200 10; then
    log_result "PASS" "Backend Health" "API server is healthy at $BACKEND_URL"
else
    log_result "FAIL" "Backend Health" "API server health check failed at $BACKEND_URL"
fi

# Frontend health check
if http_request "$FRONTEND_URL" 200 10; then
    log_result "PASS" "Frontend Health" "Frontend is accessible at $FRONTEND_URL"
else
    log_result "FAIL" "Frontend Health" "Frontend health check failed at $FRONTEND_URL"
fi

# Admin frontend health check
if http_request "$ADMIN_FRONTEND_URL" 200 10; then
    log_result "PASS" "Admin Health" "Admin frontend is accessible at $ADMIN_FRONTEND_URL"
else
    log_result "FAIL" "Admin Health" "Admin frontend health check failed at $ADMIN_FRONTEND_URL"
fi

# 1.2 Database Connectivity
echo -e "\n${YELLOW}1.2 Database Connectivity${NC}"

# Database connection test via API
if db_status=$(check_json_api "$BACKEND_URL/api/v1/health/database" 10); then
    if echo "$db_status" | jq -e '.status == "healthy"' >/dev/null 2>&1; then
        log_result "PASS" "Database Connection" "Database is connected and healthy"
    else
        log_result "FAIL" "Database Connection" "Database reports unhealthy status"
    fi
else
    log_result "FAIL" "Database Connection" "Unable to check database status"
fi

# 1.3 Redis Connectivity
echo -e "\n${YELLOW}1.3 Redis Connectivity${NC}"

# Redis connection test via API
if redis_status=$(check_json_api "$BACKEND_URL/api/v1/health/redis" 10); then
    if echo "$redis_status" | jq -e '.status == "healthy"' >/dev/null 2>&1; then
        log_result "PASS" "Redis Connection" "Redis is connected and healthy"
    else
        log_result "FAIL" "Redis Connection" "Redis reports unhealthy status"
    fi
else
    log_result "FAIL" "Redis Connection" "Unable to check Redis status"
fi

echo -e "\n${PURPLE}=== 2. SECURITY VALIDATION ===${NC}"

# 2.1 HTTPS and TLS Configuration
echo -e "\n${YELLOW}2.1 HTTPS and TLS Configuration${NC}"

# Check HTTPS redirect
if http_response=$(curl -s -o /dev/null -w "%{http_code}:%{redirect_url}" "http://epsx.io" 2>/dev/null); then
    if [[ "$http_response" == *"https://epsx.io"* ]]; then
        log_result "PASS" "HTTPS Redirect" "HTTP properly redirects to HTTPS"
    else
        log_result "FAIL" "HTTPS Redirect" "HTTP redirect not working properly: $http_response"
    fi
else
    log_result "FAIL" "HTTPS Redirect" "Unable to test HTTP redirect"
fi

# Check TLS version
if tls_info=$(curl -s -w "%{ssl_version}" "$FRONTEND_URL" -o /dev/null 2>/dev/null); then
    if [[ "$tls_info" =~ TLSv1\.[2-3] ]]; then
        log_result "PASS" "TLS Version" "Using secure TLS version: $tls_info"
    else
        log_result "FAIL" "TLS Version" "Insecure TLS version detected: $tls_info"
    fi
else
    log_result "FAIL" "TLS Version" "Unable to check TLS version"
fi

# 2.2 Security Headers
echo -e "\n${YELLOW}2.2 Security Headers${NC}"

# Check security headers
if headers=$(curl -s -I "$FRONTEND_URL" 2>/dev/null); then
    # Check HSTS header
    if echo "$headers" | grep -i "strict-transport-security" >/dev/null; then
        log_result "PASS" "HSTS Header" "HSTS header is present"
    else
        log_result "FAIL" "HSTS Header" "HSTS header is missing"
    fi
    
    # Check X-Frame-Options
    if echo "$headers" | grep -i "x-frame-options" >/dev/null; then
        log_result "PASS" "X-Frame-Options" "X-Frame-Options header is present"
    else
        log_result "FAIL" "X-Frame-Options" "X-Frame-Options header is missing"
    fi
    
    # Check Content-Security-Policy
    if echo "$headers" | grep -i "content-security-policy" >/dev/null; then
        log_result "PASS" "CSP Header" "Content-Security-Policy header is present"
    else
        log_result "FAIL" "CSP Header" "Content-Security-Policy header is missing"
    fi
    
    # Check X-Content-Type-Options
    if echo "$headers" | grep -i "x-content-type-options" >/dev/null; then
        log_result "PASS" "Content-Type-Options" "X-Content-Type-Options header is present"
    else
        log_result "FAIL" "Content-Type-Options" "X-Content-Type-Options header is missing"
    fi
else
    log_result "FAIL" "Security Headers" "Unable to fetch headers for security check"
fi

# 2.3 Authentication System
echo -e "\n${YELLOW}2.3 Authentication System${NC}"

# Check OIDC discovery endpoint
if oidc_config=$(check_json_api "$BACKEND_URL/.well-known/openid_configuration" 10); then
    if echo "$oidc_config" | jq -e '.issuer' >/dev/null 2>&1; then
        log_result "PASS" "OIDC Discovery" "OIDC discovery endpoint is functional"
    else
        log_result "FAIL" "OIDC Discovery" "OIDC discovery endpoint returns invalid response"
    fi
else
    log_result "FAIL" "OIDC Discovery" "OIDC discovery endpoint is not accessible"
fi

# Check JWT public keys endpoint
if jwks=$(check_json_api "$BACKEND_URL/oauth/jwks" 10); then
    if echo "$jwks" | jq -e '.keys[]' >/dev/null 2>&1; then
        log_result "PASS" "JWKS Endpoint" "JWT public keys are available"
    else
        log_result "FAIL" "JWKS Endpoint" "JWKS endpoint returns no keys"
    fi
else
    log_result "FAIL" "JWKS Endpoint" "JWKS endpoint is not accessible"
fi

# 2.4 Permission System
echo -e "\n${YELLOW}2.4 Permission System${NC}"

# Check permission validation endpoint
if http_request "$BACKEND_URL/api/v1/admin/permissions/validate" 401 10; then
    log_result "PASS" "Permission Validation" "Permission system properly rejects unauthorized access"
else
    log_result "FAIL" "Permission Validation" "Permission system not properly enforcing access control"
fi

# Check rate limiting
echo -e "\n${YELLOW}2.5 Rate Limiting${NC}"

# Test rate limiting by making multiple requests
rate_limit_test() {
    local success_count=0
    local rate_limited=false
    
    for i in {1..15}; do
        if response=$(curl -s -o /dev/null -w "%{http_code}" "$BACKEND_URL/api/v1/health" 2>/dev/null); then
            if [[ "$response" == "200" ]]; then
                success_count=$((success_count + 1))
            elif [[ "$response" == "429" ]]; then
                rate_limited=true
                break
            fi
        fi
        sleep 0.1
    done
    
    if [[ "$rate_limited" == true ]] || [[ $success_count -lt 15 ]]; then
        log_result "PASS" "Rate Limiting" "Rate limiting is active (got limited after $success_count requests)"
    else
        log_result "FAIL" "Rate Limiting" "Rate limiting not working (processed all 15 requests)"
    fi
}

rate_limit_test

echo -e "\n${PURPLE}=== 3. PERFORMANCE VALIDATION ===${NC}"

# 3.1 Response Times
echo -e "\n${YELLOW}3.1 Response Times${NC}"

# Measure API response time
if response_time=$(curl -s -o /dev/null -w "%{time_total}" "$BACKEND_URL/health" 2>/dev/null); then
    response_time_ms=$(echo "$response_time * 1000" | bc)
    if (( $(echo "$response_time_ms < 100" | bc -l) )); then
        log_result "PASS" "API Response Time" "Fast response time: ${response_time_ms}ms"
    elif (( $(echo "$response_time_ms < 500" | bc -l) )); then
        log_result "PASS" "API Response Time" "Acceptable response time: ${response_time_ms}ms"
    else
        log_result "FAIL" "API Response Time" "Slow response time: ${response_time_ms}ms"
    fi
else
    log_result "FAIL" "API Response Time" "Unable to measure API response time"
fi

# 3.2 Database Performance
echo -e "\n${YELLOW}3.2 Database Performance${NC}"

# Check database metrics via API
if db_metrics=$(check_json_api "$BACKEND_URL/api/v1/admin/metrics/database" 10); then
    if avg_query_time=$(echo "$db_metrics" | jq -r '.avg_query_time_ms // empty' 2>/dev/null); then
        if [[ -n "$avg_query_time" ]] && (( $(echo "$avg_query_time < 10" | bc -l) )); then
            log_result "PASS" "Database Performance" "Fast average query time: ${avg_query_time}ms"
        elif [[ -n "$avg_query_time" ]] && (( $(echo "$avg_query_time < 50" | bc -l) )); then
            log_result "PASS" "Database Performance" "Acceptable average query time: ${avg_query_time}ms"
        else
            log_result "FAIL" "Database Performance" "Slow average query time: ${avg_query_time}ms"
        fi
    else
        log_result "FAIL" "Database Performance" "Unable to get database performance metrics"
    fi
else
    log_result "FAIL" "Database Performance" "Database metrics endpoint not accessible"
fi

# 3.3 Cache Performance
echo -e "\n${YELLOW}3.3 Cache Performance${NC}"

# Check Redis cache metrics
if cache_metrics=$(check_json_api "$BACKEND_URL/api/v1/admin/metrics/cache" 10); then
    if hit_rate=$(echo "$cache_metrics" | jq -r '.hit_rate // empty' 2>/dev/null); then
        if [[ -n "$hit_rate" ]] && (( $(echo "$hit_rate > 0.9" | bc -l) )); then
            log_result "PASS" "Cache Hit Rate" "Excellent cache hit rate: $(echo "$hit_rate * 100" | bc)%"
        elif [[ -n "$hit_rate" ]] && (( $(echo "$hit_rate > 0.7" | bc -l) )); then
            log_result "PASS" "Cache Hit Rate" "Good cache hit rate: $(echo "$hit_rate * 100" | bc)%"
        else
            log_result "FAIL" "Cache Hit Rate" "Low cache hit rate: $(echo "$hit_rate * 100" | bc)%"
        fi
    else
        log_result "FAIL" "Cache Hit Rate" "Unable to get cache hit rate metrics"
    fi
else
    log_result "FAIL" "Cache Performance" "Cache metrics endpoint not accessible"
fi

echo -e "\n${PURPLE}=== 4. FUNCTIONAL VALIDATION ===${NC}"

# 4.1 API Endpoints
echo -e "\n${YELLOW}4.1 Critical API Endpoints${NC}"

# Test public endpoints
if http_request "$BACKEND_URL/api/public/rankings" 200 10; then
    log_result "PASS" "Public API" "Public rankings endpoint is accessible"
else
    log_result "FAIL" "Public API" "Public rankings endpoint is not accessible"
fi

# Test protected endpoints (should return 401 without auth)
if http_request "$BACKEND_URL/api/v1/user/profile" 401 10; then
    log_result "PASS" "Protected API" "Protected endpoints properly require authentication"
else
    log_result "FAIL" "Protected API" "Protected endpoints not properly secured"
fi

# 4.2 Frontend Functionality
echo -e "\n${YELLOW}4.2 Frontend Functionality${NC}"

# Check if frontend loads critical resources
if http_request "$FRONTEND_URL/_next/static" 200 10 || http_request "$FRONTEND_URL/static" 200 10; then
    log_result "PASS" "Static Assets" "Static assets are accessible"
else
    log_result "FAIL" "Static Assets" "Static assets are not accessible"
fi

# 4.3 Cross-Origin Resource Sharing (CORS)
echo -e "\n${YELLOW}4.3 CORS Configuration${NC}"

# Test CORS headers
if cors_response=$(curl -s -H "Origin: https://epsx.io" -H "Access-Control-Request-Method: GET" -X OPTIONS "$BACKEND_URL/api/v1/health" -I 2>/dev/null); then
    if echo "$cors_response" | grep -i "access-control-allow-origin" >/dev/null; then
        log_result "PASS" "CORS Headers" "CORS headers are properly configured"
    else
        log_result "FAIL" "CORS Headers" "CORS headers are missing"
    fi
else
    log_result "FAIL" "CORS Configuration" "Unable to test CORS configuration"
fi

echo -e "\n${PURPLE}=== 5. COMPLIANCE VALIDATION ===${NC}"

# 5.1 Audit Logging
echo -e "\n${YELLOW}5.1 Audit Logging${NC}"

# Check if audit logs are being generated
if audit_status=$(check_json_api "$BACKEND_URL/api/v1/admin/audit/status" 10); then
    if echo "$audit_status" | jq -e '.logging_enabled == true' >/dev/null 2>&1; then
        log_result "PASS" "Audit Logging" "Audit logging is enabled and functional"
    else
        log_result "FAIL" "Audit Logging" "Audit logging is not properly enabled"
    fi
else
    log_result "FAIL" "Audit Logging" "Unable to check audit logging status"
fi

# 5.2 Data Encryption
echo -e "\n${YELLOW}5.2 Data Encryption${NC}"

# Check encryption status
if encryption_status=$(check_json_api "$BACKEND_URL/api/v1/admin/security/encryption" 10); then
    if echo "$encryption_status" | jq -e '.at_rest == true and .in_transit == true' >/dev/null 2>&1; then
        log_result "PASS" "Data Encryption" "Data encryption is enabled for both at-rest and in-transit"
    else
        log_result "FAIL" "Data Encryption" "Data encryption is not fully configured"
    fi
else
    log_result "FAIL" "Data Encryption" "Unable to check encryption status"
fi

echo -e "\n${PURPLE}=== 6. MONITORING VALIDATION ===${NC}"

# 6.1 Metrics Collection
echo -e "\n${YELLOW}6.1 Metrics Collection${NC}"

# Check if metrics are being collected
if metrics_status=$(check_json_api "$BACKEND_URL/api/v1/admin/metrics/status" 10); then
    if echo "$metrics_status" | jq -e '.collection_enabled == true' >/dev/null 2>&1; then
        log_result "PASS" "Metrics Collection" "Metrics collection is enabled"
    else
        log_result "FAIL" "Metrics Collection" "Metrics collection is not enabled"
    fi
else
    log_result "FAIL" "Metrics Collection" "Unable to check metrics collection status"
fi

# 6.2 Alert System
echo -e "\n${YELLOW}6.2 Alert System${NC}"

# Check alert system status
if alert_status=$(check_json_api "$BACKEND_URL/api/v1/admin/alerts/status" 10); then
    if echo "$alert_status" | jq -e '.alerting_enabled == true' >/dev/null 2>&1; then
        log_result "PASS" "Alert System" "Alert system is enabled and configured"
    else
        log_result "FAIL" "Alert System" "Alert system is not properly configured"
    fi
else
    log_result "FAIL" "Alert System" "Unable to check alert system status"
fi

echo -e "\n${PURPLE}=== VALIDATION SUMMARY ===${NC}"

# Calculate success rate
success_rate=$(echo "scale=2; ($TOTAL_CHECKS - $FAILED_CHECKS) * 100 / $TOTAL_CHECKS" | bc)

echo -e "\n${BLUE}Total Checks: $TOTAL_CHECKS${NC}"
echo -e "${GREEN}Passed: $(($TOTAL_CHECKS - $FAILED_CHECKS))${NC}"
echo -e "${RED}Failed: $FAILED_CHECKS${NC}"
echo -e "${YELLOW}Success Rate: $success_rate%${NC}"

# Determine overall status
if [[ $FAILED_CHECKS -eq 0 ]]; then
    echo -e "\n${GREEN}🎉 DEPLOYMENT VALIDATION PASSED${NC}"
    echo -e "${GREEN}All security, performance, and functionality checks passed successfully.${NC}"
    exit 0
elif [[ $success_rate > 90 ]]; then
    echo -e "\n${YELLOW}⚠️  DEPLOYMENT VALIDATION MOSTLY PASSED${NC}"
    echo -e "${YELLOW}Most checks passed, but some issues need attention.${NC}"
    exit 1
else
    echo -e "\n${RED}❌ DEPLOYMENT VALIDATION FAILED${NC}"
    echo -e "${RED}Critical issues detected that must be resolved before production.${NC}"
    exit 2
fi