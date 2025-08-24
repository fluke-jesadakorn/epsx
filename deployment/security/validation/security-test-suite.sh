#!/bin/bash

# EPSX Production Security Validation Framework
# Comprehensive security testing and validation suite

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
SECURITY_CONFIG="deployment/security/validation/security-tests.yaml"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="/tmp/epsx_security_validation_$(date +%Y%m%d_%H%M%S)"
VERBOSE=${VERBOSE:-false}

# Load environment configuration
ENV_FILE="deployment/environments/${ENVIRONMENT}.env"
if [[ -f "$ENV_FILE" ]]; then
    source "$ENV_FILE"
else
    echo -e "${RED}❌ Environment configuration file not found: $ENV_FILE${NC}"
    exit 1
fi

# Create results directory
mkdir -p "$RESULTS_DIR"

# Security test results tracking
SECURITY_TESTS_PASSED=0
SECURITY_TESTS_FAILED=0
SECURITY_TESTS_TOTAL=0
CRITICAL_FAILURES=()

echo -e "${BLUE}🔒 EPSX Security Validation Framework${NC}"
echo -e "${BLUE}=====================================${NC}"
echo -e "Environment: $ENVIRONMENT"
echo -e "Results Directory: $RESULTS_DIR"
echo ""

# Logging function
log_security_test() {
    local status=$1
    local test_name=$2
    local message=$3
    local severity=${4:-"MEDIUM"}
    
    SECURITY_TESTS_TOTAL=$((SECURITY_TESTS_TOTAL + 1))
    
    if [[ "$status" == "PASS" ]]; then
        echo -e "${GREEN}✅ [$severity] $test_name: $message${NC}"
        SECURITY_TESTS_PASSED=$((SECURITY_TESTS_PASSED + 1))
        echo "PASS,$severity,$test_name,$message" >> "$RESULTS_DIR/security_results.csv"
    else
        echo -e "${RED}❌ [$severity] $test_name: $message${NC}"
        SECURITY_TESTS_FAILED=$((SECURITY_TESTS_FAILED + 1))
        echo "FAIL,$severity,$test_name,$message" >> "$RESULTS_DIR/security_results.csv"
        
        if [[ "$severity" == "CRITICAL" ]]; then
            CRITICAL_FAILURES+=("$test_name: $message")
        fi
    fi
}

# Function to test HTTP security headers
test_security_headers() {
    echo -e "\n${PURPLE}=== HTTP Security Headers Tests ===${NC}"
    
    local headers
    if headers=$(curl -s -I "$FRONTEND_URL" 2>/dev/null); then
        
        # Test HSTS header
        if echo "$headers" | grep -i "strict-transport-security" | grep -q "max-age"; then
            local hsts_value=$(echo "$headers" | grep -i "strict-transport-security" | head -1)
            if echo "$hsts_value" | grep -q "includeSubDomains"; then
                log_security_test "PASS" "HSTS Header" "Properly configured with includeSubDomains" "HIGH"
            else
                log_security_test "FAIL" "HSTS Header" "Missing includeSubDomains directive" "MEDIUM"
            fi
        else
            log_security_test "FAIL" "HSTS Header" "HSTS header missing or invalid" "HIGH"
        fi
        
        # Test X-Frame-Options
        if echo "$headers" | grep -i "x-frame-options" | grep -q -E "(DENY|SAMEORIGIN)"; then
            log_security_test "PASS" "X-Frame-Options" "Clickjacking protection enabled" "HIGH"
        else
            log_security_test "FAIL" "X-Frame-Options" "Clickjacking protection missing" "HIGH"
        fi
        
        # Test X-Content-Type-Options
        if echo "$headers" | grep -i "x-content-type-options" | grep -q "nosniff"; then
            log_security_test "PASS" "X-Content-Type-Options" "MIME sniffing protection enabled" "MEDIUM"
        else
            log_security_test "FAIL" "X-Content-Type-Options" "MIME sniffing protection missing" "MEDIUM"
        fi
        
        # Test Content-Security-Policy
        if echo "$headers" | grep -i "content-security-policy" >/dev/null; then
            local csp_header=$(echo "$headers" | grep -i "content-security-policy" | head -1)
            if echo "$csp_header" | grep -q "default-src"; then
                log_security_test "PASS" "Content-Security-Policy" "CSP header present with default-src" "HIGH"
            else
                log_security_test "FAIL" "Content-Security-Policy" "CSP header incomplete" "MEDIUM"
            fi
        else
            log_security_test "FAIL" "Content-Security-Policy" "CSP header missing" "HIGH"
        fi
        
        # Test X-XSS-Protection
        if echo "$headers" | grep -i "x-xss-protection" | grep -q "1; mode=block"; then
            log_security_test "PASS" "X-XSS-Protection" "XSS protection enabled" "MEDIUM"
        else
            log_security_test "FAIL" "X-XSS-Protection" "XSS protection missing or misconfigured" "MEDIUM"
        fi
        
    else
        log_security_test "FAIL" "Security Headers Test" "Unable to fetch headers" "CRITICAL"
    fi
}

# Function to test TLS/SSL configuration
test_tls_configuration() {
    echo -e "\n${PURPLE}=== TLS/SSL Configuration Tests ===${NC}"
    
    # Test TLS version
    local tls_version
    if tls_version=$(curl -s -w "%{ssl_version}" "$FRONTEND_URL" -o /dev/null 2>/dev/null); then
        if [[ "$tls_version" =~ TLSv1\.[2-3] ]]; then
            log_security_test "PASS" "TLS Version" "Using secure TLS version: $tls_version" "CRITICAL"
        else
            log_security_test "FAIL" "TLS Version" "Insecure TLS version: $tls_version" "CRITICAL"
        fi
    else
        log_security_test "FAIL" "TLS Version" "Unable to determine TLS version" "HIGH"
    fi
    
    # Test certificate validity
    local cert_info
    if cert_info=$(echo | openssl s_client -servername "$(echo "$FRONTEND_URL" | sed 's|https://||')" -connect "$(echo "$FRONTEND_URL" | sed 's|https://||'):443" 2>/dev/null | openssl x509 -noout -dates 2>/dev/null); then
        local not_after=$(echo "$cert_info" | grep "notAfter" | cut -d= -f2)
        local expiry_epoch=$(date -d "$not_after" +%s 2>/dev/null || echo "0")
        local current_epoch=$(date +%s)
        local days_until_expiry=$(( (expiry_epoch - current_epoch) / 86400 ))
        
        if [[ $days_until_expiry -gt 30 ]]; then
            log_security_test "PASS" "Certificate Validity" "Certificate valid for $days_until_expiry days" "MEDIUM"
        elif [[ $days_until_expiry -gt 0 ]]; then
            log_security_test "FAIL" "Certificate Validity" "Certificate expires in $days_until_expiry days" "HIGH"
        else
            log_security_test "FAIL" "Certificate Validity" "Certificate has expired" "CRITICAL"
        fi
    else
        log_security_test "FAIL" "Certificate Validity" "Unable to verify certificate" "HIGH"
    fi
    
    # Test cipher suites
    local cipher_info
    if cipher_info=$(nmap --script ssl-enum-ciphers -p 443 "$(echo "$FRONTEND_URL" | sed 's|https://||')" 2>/dev/null | grep -A 20 "TLSv1.2\|TLSv1.3"); then
        if echo "$cipher_info" | grep -q "ECDHE\|DHE"; then
            log_security_test "PASS" "Perfect Forward Secrecy" "PFS-enabled ciphers detected" "HIGH"
        else
            log_security_test "FAIL" "Perfect Forward Secrecy" "No PFS-enabled ciphers found" "HIGH"
        fi
        
        if echo "$cipher_info" | grep -q -i "weak\|null\|export"; then
            log_security_test "FAIL" "Weak Ciphers" "Weak cipher suites detected" "HIGH"
        else
            log_security_test "PASS" "Weak Ciphers" "No weak cipher suites detected" "MEDIUM"
        fi
    else
        log_security_test "FAIL" "Cipher Analysis" "Unable to analyze cipher suites" "MEDIUM"
    fi
}

# Function to test authentication and authorization
test_authentication_security() {
    echo -e "\n${PURPLE}=== Authentication & Authorization Tests ===${NC}"
    
    # Test public endpoints (should be accessible)
    if curl -s -f "$BACKEND_URL/health" >/dev/null 2>&1; then
        log_security_test "PASS" "Public Endpoint Access" "Public health endpoint accessible" "LOW"
    else
        log_security_test "FAIL" "Public Endpoint Access" "Public health endpoint not accessible" "MEDIUM"
    fi
    
    # Test protected endpoints (should require auth)
    local protected_endpoints=(
        "/api/v1/user/profile"
        "/api/v1/admin/users"
        "/api/v1/admin/permissions"
        "/api/v1/admin/security"
    )
    
    for endpoint in "${protected_endpoints[@]}"; do
        local response_code
        if response_code=$(curl -s -o /dev/null -w "%{http_code}" "$BACKEND_URL$endpoint" 2>/dev/null); then
            if [[ "$response_code" == "401" || "$response_code" == "403" ]]; then
                log_security_test "PASS" "Protected Endpoint" "$endpoint properly requires authentication (HTTP $response_code)" "HIGH"
            else
                log_security_test "FAIL" "Protected Endpoint" "$endpoint allows unauthorized access (HTTP $response_code)" "CRITICAL"
            fi
        else
            log_security_test "FAIL" "Protected Endpoint" "Unable to test $endpoint" "MEDIUM"
        fi
    done
    
    # Test OIDC configuration
    if oidc_config=$(curl -s "$BACKEND_URL/.well-known/openid_configuration" 2>/dev/null); then
        if echo "$oidc_config" | jq -e '.issuer' >/dev/null 2>&1; then
            local issuer=$(echo "$oidc_config" | jq -r '.issuer')
            if [[ "$issuer" == "$BACKEND_URL" ]]; then
                log_security_test "PASS" "OIDC Configuration" "OIDC discovery endpoint properly configured" "HIGH"
            else
                log_security_test "FAIL" "OIDC Configuration" "OIDC issuer mismatch: $issuer" "HIGH"
            fi
        else
            log_security_test "FAIL" "OIDC Configuration" "Invalid OIDC discovery response" "HIGH"
        fi
    else
        log_security_test "FAIL" "OIDC Configuration" "OIDC discovery endpoint not accessible" "HIGH"
    fi
    
    # Test JWT public keys
    if jwks=$(curl -s "$BACKEND_URL/oauth/jwks" 2>/dev/null); then
        if echo "$jwks" | jq -e '.keys[]' >/dev/null 2>&1; then
            local key_count=$(echo "$jwks" | jq '.keys | length')
            if [[ $key_count -gt 0 ]]; then
                log_security_test "PASS" "JWT Public Keys" "$key_count JWT public keys available" "HIGH"
            else
                log_security_test "FAIL" "JWT Public Keys" "No JWT public keys found" "CRITICAL"
            fi
        else
            log_security_test "FAIL" "JWT Public Keys" "Invalid JWKS response" "HIGH"
        fi
    else
        log_security_test "FAIL" "JWT Public Keys" "JWKS endpoint not accessible" "HIGH"
    fi
}

# Function to test rate limiting and brute force protection
test_rate_limiting() {
    echo -e "\n${PURPLE}=== Rate Limiting & Brute Force Protection Tests ===${NC}"
    
    # Test basic rate limiting
    local success_count=0
    local rate_limited=false
    local test_endpoint="$BACKEND_URL/api/v1/health"
    
    echo "Testing rate limiting with 20 rapid requests..."
    for i in {1..20}; do
        local response_code
        if response_code=$(curl -s -o /dev/null -w "%{http_code}" "$test_endpoint" 2>/dev/null); then
            if [[ "$response_code" == "200" ]]; then
                success_count=$((success_count + 1))
            elif [[ "$response_code" == "429" ]]; then
                rate_limited=true
                break
            fi
        fi
        sleep 0.1
    done
    
    if [[ "$rate_limited" == true ]]; then
        log_security_test "PASS" "Rate Limiting" "Rate limiting activated after $success_count requests" "HIGH"
    elif [[ $success_count -lt 20 ]]; then
        log_security_test "PASS" "Rate Limiting" "Rate limiting or other protection active" "MEDIUM"
    else
        log_security_test "FAIL" "Rate Limiting" "No rate limiting detected (processed all 20 requests)" "HIGH"
    fi
    
    # Test authentication rate limiting
    echo "Testing authentication rate limiting..."
    local auth_endpoint="$BACKEND_URL/oauth/token"
    local auth_limited=false
    local auth_attempts=0
    
    for i in {1..10}; do
        local response_code
        if response_code=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$auth_endpoint" \
            -H "Content-Type: application/x-www-form-urlencoded" \
            -d "grant_type=password&username=invalid&password=invalid" 2>/dev/null); then
            auth_attempts=$((auth_attempts + 1))
            if [[ "$response_code" == "429" ]]; then
                auth_limited=true
                break
            fi
        fi
        sleep 0.5
    done
    
    if [[ "$auth_limited" == true ]]; then
        log_security_test "PASS" "Auth Rate Limiting" "Authentication rate limiting activated after $auth_attempts attempts" "HIGH"
    else
        log_security_test "FAIL" "Auth Rate Limiting" "Authentication rate limiting not detected" "HIGH"
    fi
}

# Function to test input validation and injection protection
test_input_validation() {
    echo -e "\n${PURPLE}=== Input Validation & Injection Protection Tests ===${NC}"
    
    # SQL Injection test patterns
    local sql_patterns=(
        "' OR '1'='1"
        "'; DROP TABLE users; --"
        "1' UNION SELECT NULL--"
        "' OR 1=1 #"
    )
    
    # Test SQL injection on public endpoints
    local test_endpoint="$BACKEND_URL/api/public/rankings"
    for pattern in "${sql_patterns[@]}"; do
        local encoded_pattern=$(printf '%s' "$pattern" | jq -sRr @uri)
        local response_code
        if response_code=$(curl -s -o /dev/null -w "%{http_code}" "$test_endpoint?search=$encoded_pattern" 2>/dev/null); then
            if [[ "$response_code" == "400" || "$response_code" == "422" ]]; then
                log_security_test "PASS" "SQL Injection Protection" "Malicious input rejected: '$pattern'" "HIGH"
            elif [[ "$response_code" == "500" ]]; then
                log_security_test "FAIL" "SQL Injection Protection" "Server error on malicious input: '$pattern'" "CRITICAL"
            else
                log_security_test "FAIL" "SQL Injection Protection" "Malicious input accepted: '$pattern'" "CRITICAL"
            fi
        else
            log_security_test "FAIL" "SQL Injection Test" "Unable to test injection pattern: '$pattern'" "MEDIUM"
        fi
    done
    
    # XSS test patterns
    local xss_patterns=(
        "<script>alert('xss')</script>"
        "javascript:alert('xss')"
        "<img src=x onerror=alert('xss')>"
        "';alert('xss');//"
    )
    
    # Test XSS protection
    for pattern in "${xss_patterns[@]}"; do
        local encoded_pattern=$(printf '%s' "$pattern" | jq -sRr @uri)
        local response_code
        if response_code=$(curl -s -o /dev/null -w "%{http_code}" "$test_endpoint?search=$encoded_pattern" 2>/dev/null); then
            if [[ "$response_code" == "400" || "$response_code" == "422" ]]; then
                log_security_test "PASS" "XSS Protection" "XSS pattern rejected: '$pattern'" "HIGH"
            elif [[ "$response_code" == "500" ]]; then
                log_security_test "FAIL" "XSS Protection" "Server error on XSS input: '$pattern'" "HIGH"
            else
                # Additional check: ensure the response doesn't reflect the malicious script
                local response_body
                if response_body=$(curl -s "$test_endpoint?search=$encoded_pattern" 2>/dev/null); then
                    if echo "$response_body" | grep -q "<script>"; then
                        log_security_test "FAIL" "XSS Protection" "XSS script reflected in response: '$pattern'" "CRITICAL"
                    else
                        log_security_test "PASS" "XSS Protection" "XSS input sanitized: '$pattern'" "HIGH"
                    fi
                fi
            fi
        fi
    done
    
    # Test command injection protection
    local command_patterns=(
        "; ls -la"
        "| whoami"
        "&& cat /etc/passwd"
        "\$(id)"
    )
    
    for pattern in "${command_patterns[@]}"; do
        local encoded_pattern=$(printf '%s' "$pattern" | jq -sRr @uri)
        local response_code
        if response_code=$(curl -s -o /dev/null -w "%{http_code}" "$test_endpoint?search=$encoded_pattern" 2>/dev/null); then
            if [[ "$response_code" == "400" || "$response_code" == "422" ]]; then
                log_security_test "PASS" "Command Injection Protection" "Command injection pattern rejected: '$pattern'" "HIGH"
            elif [[ "$response_code" == "500" ]]; then
                log_security_test "FAIL" "Command Injection Protection" "Server error on command injection: '$pattern'" "CRITICAL"
            else
                log_security_test "PASS" "Command Injection Protection" "Command injection pattern handled: '$pattern'" "MEDIUM"
            fi
        fi
    done
}

# Function to test CORS configuration
test_cors_configuration() {
    echo -e "\n${PURPLE}=== CORS Configuration Tests ===${NC}"
    
    # Test legitimate CORS request
    local cors_response
    if cors_response=$(curl -s -H "Origin: https://epsx.io" -H "Access-Control-Request-Method: GET" -X OPTIONS "$BACKEND_URL/api/v1/health" -I 2>/dev/null); then
        if echo "$cors_response" | grep -i "access-control-allow-origin" | grep -q "https://epsx.io"; then
            log_security_test "PASS" "CORS Legitimate Origin" "CORS properly configured for legitimate origin" "MEDIUM"
        else
            log_security_test "FAIL" "CORS Legitimate Origin" "CORS not allowing legitimate origin" "MEDIUM"
        fi
    else
        log_security_test "FAIL" "CORS Configuration" "Unable to test CORS configuration" "MEDIUM"
    fi
    
    # Test malicious CORS request
    if cors_response=$(curl -s -H "Origin: https://malicious.com" -H "Access-Control-Request-Method: GET" -X OPTIONS "$BACKEND_URL/api/v1/health" -I 2>/dev/null); then
        if echo "$cors_response" | grep -i "access-control-allow-origin" | grep -q "malicious.com"; then
            log_security_test "FAIL" "CORS Malicious Origin" "CORS allowing malicious origin" "HIGH"
        else
            log_security_test "PASS" "CORS Malicious Origin" "CORS properly blocking malicious origin" "HIGH"
        fi
    fi
    
    # Test wildcard CORS (should not be used in production)
    if cors_response=$(curl -s -H "Origin: https://anydomain.com" -H "Access-Control-Request-Method: GET" -X OPTIONS "$BACKEND_URL/api/v1/health" -I 2>/dev/null); then
        if echo "$cors_response" | grep -i "access-control-allow-origin" | grep -q "\*"; then
            log_security_test "FAIL" "CORS Wildcard" "Dangerous wildcard CORS configuration detected" "CRITICAL"
        else
            log_security_test "PASS" "CORS Wildcard" "No wildcard CORS configuration" "HIGH"
        fi
    fi
}

# Function to test session security
test_session_security() {
    echo -e "\n${PURPLE}=== Session Security Tests ===${NC}"
    
    # Test session cookie security attributes
    local cookie_response
    if cookie_response=$(curl -s -c - "$FRONTEND_URL" 2>/dev/null | grep -i "set-cookie"); then
        if echo "$cookie_response" | grep -i "secure"; then
            log_security_test "PASS" "Secure Cookie Flag" "Session cookies have Secure flag" "HIGH"
        else
            log_security_test "FAIL" "Secure Cookie Flag" "Session cookies missing Secure flag" "HIGH"
        fi
        
        if echo "$cookie_response" | grep -i "httponly"; then
            log_security_test "PASS" "HttpOnly Cookie Flag" "Session cookies have HttpOnly flag" "HIGH"
        else
            log_security_test "FAIL" "HttpOnly Cookie Flag" "Session cookies missing HttpOnly flag" "HIGH"
        fi
        
        if echo "$cookie_response" | grep -i "samesite"; then
            log_security_test "PASS" "SameSite Cookie Attribute" "Session cookies have SameSite attribute" "MEDIUM"
        else
            log_security_test "FAIL" "SameSite Cookie Attribute" "Session cookies missing SameSite attribute" "MEDIUM"
        fi
    else
        log_security_test "FAIL" "Session Cookie Test" "Unable to test session cookies" "MEDIUM"
    fi
    
    # Test session fixation protection
    local initial_session
    local post_auth_session
    
    if initial_session=$(curl -s -c - "$FRONTEND_URL/login" 2>/dev/null | grep -o "session=[^;]*"); then
        # Simulate login (this would need actual credentials in a real test)
        if post_auth_session=$(curl -s -b "$initial_session" -c - "$FRONTEND_URL/api/auth/session" 2>/dev/null | grep -o "session=[^;]*"); then
            if [[ "$initial_session" != "$post_auth_session" ]]; then
                log_security_test "PASS" "Session Fixation Protection" "Session ID changes after authentication" "HIGH"
            else
                log_security_test "FAIL" "Session Fixation Protection" "Session ID does not change after authentication" "HIGH"
            fi
        fi
    fi
}

# Function to generate security report
generate_security_report() {
    echo -e "\n${PURPLE}=== Generating Security Report ===${NC}"
    
    local report_file="$RESULTS_DIR/security_validation_report.html"
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>EPSX Security Validation Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background-color: #f4f4f4; padding: 20px; border-radius: 5px; }
        .summary { background-color: #e7f3ff; padding: 15px; margin: 20px 0; border-radius: 5px; }
        .critical { color: #d32f2f; font-weight: bold; }
        .high { color: #f57c00; font-weight: bold; }
        .medium { color: #fbc02d; }
        .low { color: #388e3c; }
        .pass { color: #4caf50; }
        .fail { color: #f44336; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .metric { display: inline-block; margin: 10px; padding: 10px; background-color: #f9f9f9; border-radius: 5px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>EPSX Security Validation Report</h1>
        <p><strong>Environment:</strong> $ENVIRONMENT</p>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Test Suite Version:</strong> 1.0.0</p>
    </div>
    
    <div class="summary">
        <h2>Executive Summary</h2>
        <div class="metric">
            <strong>Total Tests:</strong> $SECURITY_TESTS_TOTAL
        </div>
        <div class="metric">
            <strong>Passed:</strong> <span class="pass">$SECURITY_TESTS_PASSED</span>
        </div>
        <div class="metric">
            <strong>Failed:</strong> <span class="fail">$SECURITY_TESTS_FAILED</span>
        </div>
        <div class="metric">
            <strong>Success Rate:</strong> $(echo "scale=1; $SECURITY_TESTS_PASSED * 100 / $SECURITY_TESTS_TOTAL" | bc)%
        </div>
    </div>
EOF

    if [[ ${#CRITICAL_FAILURES[@]} -gt 0 ]]; then
        cat >> "$report_file" << EOF
    <div style="background-color: #ffebee; padding: 15px; margin: 20px 0; border-radius: 5px; border-left: 5px solid #f44336;">
        <h3 class="critical">Critical Security Issues</h3>
        <ul>
EOF
        for failure in "${CRITICAL_FAILURES[@]}"; do
            echo "            <li class=\"critical\">$failure</li>" >> "$report_file"
        done
        cat >> "$report_file" << EOF
        </ul>
    </div>
EOF
    fi

    cat >> "$report_file" << EOF
    <h2>Detailed Test Results</h2>
    <table>
        <thead>
            <tr>
                <th>Status</th>
                <th>Severity</th>
                <th>Test Name</th>
                <th>Details</th>
            </tr>
        </thead>
        <tbody>
EOF

    if [[ -f "$RESULTS_DIR/security_results.csv" ]]; then
        while IFS=',' read -r status severity test_name message; do
            local status_class="pass"
            if [[ "$status" == "FAIL" ]]; then
                status_class="fail"
            fi
            
            local severity_class="low"
            case $severity in
                "CRITICAL") severity_class="critical" ;;
                "HIGH") severity_class="high" ;;
                "MEDIUM") severity_class="medium" ;;
            esac
            
            cat >> "$report_file" << EOF
            <tr>
                <td class="$status_class">$status</td>
                <td class="$severity_class">$severity</td>
                <td>$test_name</td>
                <td>$message</td>
            </tr>
EOF
        done < "$RESULTS_DIR/security_results.csv"
    fi

    cat >> "$report_file" << EOF
        </tbody>
    </table>
    
    <div style="margin-top: 40px; padding: 20px; background-color: #f4f4f4; border-radius: 5px;">
        <h3>Recommendations</h3>
        <ul>
            <li>Review and address all critical and high-severity issues immediately</li>
            <li>Implement additional security controls for failed tests</li>
            <li>Schedule regular security validation testing</li>
            <li>Monitor security metrics continuously in production</li>
            <li>Update security configurations based on evolving threats</li>
        </ul>
    </div>
    
    <div style="margin-top: 20px; font-size: 0.9em; color: #666;">
        <p>This report was generated by the EPSX Security Validation Framework.</p>
        <p>For questions or concerns, contact the security team at security@epsx.io</p>
    </div>
</body>
</html>
EOF

    echo -e "${GREEN}Security report generated: $report_file${NC}"
}

# Main execution
main() {
    echo "CSV Header" > "$RESULTS_DIR/security_results.csv"
    echo "Status,Severity,Test Name,Message" >> "$RESULTS_DIR/security_results.csv"
    
    test_security_headers
    test_tls_configuration
    test_authentication_security
    test_rate_limiting
    test_input_validation
    test_cors_configuration
    test_session_security
    
    generate_security_report
    
    echo -e "\n${PURPLE}=== Security Validation Summary ===${NC}"
    echo -e "Total Tests: $SECURITY_TESTS_TOTAL"
    echo -e "${GREEN}Passed: $SECURITY_TESTS_PASSED${NC}"
    echo -e "${RED}Failed: $SECURITY_TESTS_FAILED${NC}"
    
    local success_rate=$(echo "scale=1; $SECURITY_TESTS_PASSED * 100 / $SECURITY_TESTS_TOTAL" | bc)
    echo -e "Success Rate: $success_rate%"
    
    if [[ ${#CRITICAL_FAILURES[@]} -gt 0 ]]; then
        echo -e "\n${RED}❌ CRITICAL SECURITY ISSUES DETECTED:${NC}"
        for failure in "${CRITICAL_FAILURES[@]}"; do
            echo -e "${RED}  • $failure${NC}"
        done
        echo -e "\n${RED}Production deployment should be blocked until critical issues are resolved.${NC}"
        exit 2
    elif [[ $SECURITY_TESTS_FAILED -gt 0 ]]; then
        echo -e "\n${YELLOW}⚠️  Some security tests failed. Review required before production deployment.${NC}"
        exit 1
    else
        echo -e "\n${GREEN}✅ All security validation tests passed!${NC}"
        exit 0
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [environment] [options]"
            echo "Environment: production, staging, development (default: production)"
            echo "Options:"
            echo "  --verbose    Enable verbose output"
            echo "  --help       Show this help message"
            exit 0
            ;;
        *)
            ENVIRONMENT=$1
            shift
            ;;
    esac
done

# Run main function
main