#!/bin/bash

# EPSX Compliance and Audit Validation System
# Comprehensive compliance checking for SOX, PCI DSS, GDPR, and other regulations

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
COMPLIANCE_CONFIG="deployment/security/compliance/compliance-requirements.yaml"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="/tmp/epsx_compliance_validation_$(date +%Y%m%d_%H%M%S)"
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

# Compliance test results tracking
COMPLIANCE_TESTS_PASSED=0
COMPLIANCE_TESTS_FAILED=0
COMPLIANCE_TESTS_TOTAL=0
COMPLIANCE_VIOLATIONS=()
REGULATORY_FAILURES=()

echo -e "${BLUE}📋 EPSX Compliance & Audit Validation System${NC}"
echo -e "${BLUE}==============================================${NC}"
echo -e "Environment: $ENVIRONMENT"
echo -e "Results Directory: $RESULTS_DIR"
echo ""

# Logging function for compliance tests
log_compliance_test() {
    local status=$1
    local regulation=$2
    local requirement=$3
    local message=$4
    local severity=${5:-"MEDIUM"}
    
    COMPLIANCE_TESTS_TOTAL=$((COMPLIANCE_TESTS_TOTAL + 1))
    
    if [[ "$status" == "PASS" ]]; then
        echo -e "${GREEN}✅ [$regulation] $requirement: $message${NC}"
        COMPLIANCE_TESTS_PASSED=$((COMPLIANCE_TESTS_PASSED + 1))
        echo "PASS,$regulation,$requirement,$message,$severity" >> "$RESULTS_DIR/compliance_results.csv"
    else
        echo -e "${RED}❌ [$regulation] $requirement: $message${NC}"
        COMPLIANCE_TESTS_FAILED=$((COMPLIANCE_TESTS_FAILED + 1))
        echo "FAIL,$regulation,$requirement,$message,$severity" >> "$RESULTS_DIR/compliance_results.csv"
        
        COMPLIANCE_VIOLATIONS+=("$regulation - $requirement: $message")
        
        if [[ "$severity" == "CRITICAL" ]]; then
            REGULATORY_FAILURES+=("$regulation - $requirement: $message")
        fi
    fi
}

# Function to check API response and parse JSON
check_api_compliance() {
    local endpoint=$1
    local timeout=${2:-10}
    
    if response=$(curl -s --max-time $timeout "$BACKEND_URL$endpoint" 2>/dev/null); then
        if echo "$response" | jq . >/dev/null 2>&1; then
            echo "$response"
            return 0
        fi
    fi
    return 1
}

# SOX (Sarbanes-Oxley) Compliance Tests
test_sox_compliance() {
    echo -e "\n${PURPLE}=== SOX (Sarbanes-Oxley) Compliance Tests ===${NC}"
    
    # SOX Requirement: Audit trail for all financial data access
    if audit_config=$(check_api_compliance "/api/v1/admin/compliance/sox/audit-config"); then
        if echo "$audit_config" | jq -e '.financial_data_auditing == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "SOX" "Financial Data Audit Trail" "All financial data access is being audited" "CRITICAL"
        else
            log_compliance_test "FAIL" "SOX" "Financial Data Audit Trail" "Financial data auditing not enabled" "CRITICAL"
        fi
    else
        log_compliance_test "FAIL" "SOX" "Financial Data Audit Trail" "Unable to verify audit configuration" "CRITICAL"
    fi
    
    # SOX Requirement: Immutable audit logs
    if audit_status=$(check_api_compliance "/api/v1/admin/compliance/sox/audit-immutability"); then
        if echo "$audit_status" | jq -e '.immutable_logs == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "SOX" "Immutable Audit Logs" "Audit logs are immutable and tamper-proof" "CRITICAL"
        else
            log_compliance_test "FAIL" "SOX" "Immutable Audit Logs" "Audit logs are not immutable" "CRITICAL"
        fi
    else
        log_compliance_test "FAIL" "SOX" "Immutable Audit Logs" "Unable to verify audit log immutability" "CRITICAL"
    fi
    
    # SOX Requirement: Financial data retention (7 years)
    if retention_config=$(check_api_compliance "/api/v1/admin/compliance/sox/data-retention"); then
        local retention_days=$(echo "$retention_config" | jq -r '.financial_data_retention_days // 0')
        if [[ $retention_days -ge 2555 ]]; then  # 7 years = 2555 days
            log_compliance_test "PASS" "SOX" "Data Retention Policy" "Financial data retained for $retention_days days (≥7 years)" "HIGH"
        else
            log_compliance_test "FAIL" "SOX" "Data Retention Policy" "Financial data retention insufficient: $retention_days days" "CRITICAL"
        fi
    else
        log_compliance_test "FAIL" "SOX" "Data Retention Policy" "Unable to verify data retention configuration" "HIGH"
    fi
    
    # SOX Requirement: Segregation of duties
    if access_controls=$(check_api_compliance "/api/v1/admin/compliance/sox/access-controls"); then
        if echo "$access_controls" | jq -e '.segregation_of_duties == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "SOX" "Segregation of Duties" "Proper role separation implemented" "HIGH"
        else
            log_compliance_test "FAIL" "SOX" "Segregation of Duties" "Inadequate segregation of duties" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "SOX" "Segregation of Duties" "Unable to verify access controls" "HIGH"
    fi
    
    # SOX Requirement: Change management controls
    if change_controls=$(check_api_compliance "/api/v1/admin/compliance/sox/change-management"); then
        if echo "$change_controls" | jq -e '.documented_changes == true and .approval_required == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "SOX" "Change Management" "Proper change management controls in place" "HIGH"
        else
            log_compliance_test "FAIL" "SOX" "Change Management" "Inadequate change management controls" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "SOX" "Change Management" "Unable to verify change management controls" "HIGH"
    fi
}

# PCI DSS (Payment Card Industry Data Security Standard) Compliance Tests
test_pci_dss_compliance() {
    echo -e "\n${PURPLE}=== PCI DSS Compliance Tests ===${NC}"
    
    # PCI DSS Requirement 1: Install and maintain a firewall configuration
    if firewall_config=$(check_api_compliance "/api/v1/admin/compliance/pci/firewall-status"); then
        if echo "$firewall_config" | jq -e '.firewall_active == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "PCI DSS" "Firewall Configuration" "Network firewall is active and configured" "CRITICAL"
        else
            log_compliance_test "FAIL" "PCI DSS" "Firewall Configuration" "Network firewall not properly configured" "CRITICAL"
        fi
    else
        log_compliance_test "FAIL" "PCI DSS" "Firewall Configuration" "Unable to verify firewall status" "CRITICAL"
    fi
    
    # PCI DSS Requirement 2: Do not use vendor-supplied defaults for system passwords
    if default_creds=$(check_api_compliance "/api/v1/admin/compliance/pci/default-credentials"); then
        if echo "$default_creds" | jq -e '.default_credentials_removed == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "PCI DSS" "Default Credentials" "No default credentials in use" "CRITICAL"
        else
            log_compliance_test "FAIL" "PCI DSS" "Default Credentials" "Default credentials detected" "CRITICAL"
        fi
    else
        log_compliance_test "FAIL" "PCI DSS" "Default Credentials" "Unable to verify credential configuration" "CRITICAL"
    fi
    
    # PCI DSS Requirement 3: Protect stored cardholder data
    if data_encryption=$(check_api_compliance "/api/v1/admin/compliance/pci/data-encryption"); then
        if echo "$data_encryption" | jq -e '.cardholder_data_encrypted == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "PCI DSS" "Data Encryption" "Cardholder data properly encrypted" "CRITICAL"
        else
            log_compliance_test "FAIL" "PCI DSS" "Data Encryption" "Cardholder data encryption insufficient" "CRITICAL"
        fi
    else
        log_compliance_test "FAIL" "PCI DSS" "Data Encryption" "Unable to verify data encryption status" "CRITICAL"
    fi
    
    # PCI DSS Requirement 4: Encrypt transmission of cardholder data across networks
    if transmission_encryption=$(check_api_compliance "/api/v1/admin/compliance/pci/transmission-encryption"); then
        if echo "$transmission_encryption" | jq -e '.encrypted_transmission == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "PCI DSS" "Transmission Encryption" "Data transmission properly encrypted" "CRITICAL"
        else
            log_compliance_test "FAIL" "PCI DSS" "Transmission Encryption" "Data transmission not properly encrypted" "CRITICAL"
        fi
    else
        log_compliance_test "FAIL" "PCI DSS" "Transmission Encryption" "Unable to verify transmission encryption" "CRITICAL"
    fi
    
    # PCI DSS Requirement 8: Identify and authenticate access to system components
    if access_authentication=$(check_api_compliance "/api/v1/admin/compliance/pci/authentication"); then
        if echo "$access_authentication" | jq -e '.unique_user_ids == true and .strong_authentication == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "PCI DSS" "Access Authentication" "Proper user identification and authentication" "HIGH"
        else
            log_compliance_test "FAIL" "PCI DSS" "Access Authentication" "Inadequate authentication controls" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "PCI DSS" "Access Authentication" "Unable to verify authentication controls" "HIGH"
    fi
    
    # PCI DSS Requirement 10: Track and monitor all access to network resources
    if access_monitoring=$(check_api_compliance "/api/v1/admin/compliance/pci/access-monitoring"); then
        if echo "$access_monitoring" | jq -e '.access_logging == true and .log_monitoring == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "PCI DSS" "Access Monitoring" "Comprehensive access logging and monitoring" "HIGH"
        else
            log_compliance_test "FAIL" "PCI DSS" "Access Monitoring" "Inadequate access monitoring" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "PCI DSS" "Access Monitoring" "Unable to verify access monitoring" "HIGH"
    fi
}

# GDPR (General Data Protection Regulation) Compliance Tests
test_gdpr_compliance() {
    echo -e "\n${PURPLE}=== GDPR Compliance Tests ===${NC}"
    
    # GDPR Article 17: Right to erasure (right to be forgotten)
    if data_erasure=$(check_api_compliance "/api/v1/admin/compliance/gdpr/data-erasure"); then
        if echo "$data_erasure" | jq -e '.erasure_capability == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "GDPR" "Right to Erasure" "Data erasure capability implemented" "HIGH"
        else
            log_compliance_test "FAIL" "GDPR" "Right to Erasure" "Data erasure capability not implemented" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "GDPR" "Right to Erasure" "Unable to verify data erasure capability" "HIGH"
    fi
    
    # GDPR Article 20: Right to data portability
    if data_portability=$(check_api_compliance "/api/v1/admin/compliance/gdpr/data-portability"); then
        if echo "$data_portability" | jq -e '.data_export_available == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "GDPR" "Data Portability" "Data export functionality available" "MEDIUM"
        else
            log_compliance_test "FAIL" "GDPR" "Data Portability" "Data export functionality not available" "MEDIUM"
        fi
    else
        log_compliance_test "FAIL" "GDPR" "Data Portability" "Unable to verify data portability" "MEDIUM"
    fi
    
    # GDPR Article 32: Security of processing
    if processing_security=$(check_api_compliance "/api/v1/admin/compliance/gdpr/processing-security"); then
        if echo "$processing_security" | jq -e '.encryption_enabled == true and .access_controls == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "GDPR" "Processing Security" "Adequate security measures for data processing" "HIGH"
        else
            log_compliance_test "FAIL" "GDPR" "Processing Security" "Inadequate security measures for data processing" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "GDPR" "Processing Security" "Unable to verify processing security" "HIGH"
    fi
    
    # GDPR Article 33: Notification of a personal data breach
    if breach_notification=$(check_api_compliance "/api/v1/admin/compliance/gdpr/breach-notification"); then
        if echo "$breach_notification" | jq -e '.automated_notification == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "GDPR" "Breach Notification" "Automated breach notification system active" "HIGH"
        else
            log_compliance_test "FAIL" "GDPR" "Breach Notification" "Breach notification system not properly configured" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "GDPR" "Breach Notification" "Unable to verify breach notification system" "HIGH"
    fi
    
    # GDPR: Consent management
    if consent_management=$(check_api_compliance "/api/v1/admin/compliance/gdpr/consent-management"); then
        if echo "$consent_management" | jq -e '.consent_tracking == true and .withdrawal_capability == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "GDPR" "Consent Management" "Proper consent tracking and withdrawal capability" "HIGH"
        else
            log_compliance_test "FAIL" "GDPR" "Consent Management" "Inadequate consent management" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "GDPR" "Consent Management" "Unable to verify consent management" "HIGH"
    fi
}

# CCPA (California Consumer Privacy Act) Compliance Tests
test_ccpa_compliance() {
    echo -e "\n${PURPLE}=== CCPA Compliance Tests ===${NC}"
    
    # CCPA: Right to know
    if data_disclosure=$(check_api_compliance "/api/v1/admin/compliance/ccpa/data-disclosure"); then
        if echo "$data_disclosure" | jq -e '.disclosure_capability == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "CCPA" "Right to Know" "Data disclosure capability implemented" "MEDIUM"
        else
            log_compliance_test "FAIL" "CCPA" "Right to Know" "Data disclosure capability not implemented" "MEDIUM"
        fi
    else
        log_compliance_test "FAIL" "CCPA" "Right to Know" "Unable to verify data disclosure capability" "MEDIUM"
    fi
    
    # CCPA: Right to delete
    if data_deletion=$(check_api_compliance "/api/v1/admin/compliance/ccpa/data-deletion"); then
        if echo "$data_deletion" | jq -e '.deletion_capability == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "CCPA" "Right to Delete" "Data deletion capability implemented" "HIGH"
        else
            log_compliance_test "FAIL" "CCPA" "Right to Delete" "Data deletion capability not implemented" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "CCPA" "Right to Delete" "Unable to verify data deletion capability" "HIGH"
    fi
    
    # CCPA: Opt-out of sale
    if opt_out_mechanism=$(check_api_compliance "/api/v1/admin/compliance/ccpa/opt-out"); then
        if echo "$opt_out_mechanism" | jq -e '.opt_out_available == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "CCPA" "Opt-out of Sale" "Opt-out mechanism available" "MEDIUM"
        else
            log_compliance_test "FAIL" "CCPA" "Opt-out of Sale" "Opt-out mechanism not available" "MEDIUM"
        fi
    else
        log_compliance_test "FAIL" "CCPA" "Opt-out of Sale" "Unable to verify opt-out mechanism" "MEDIUM"
    fi
}

# General Audit and Compliance Controls
test_general_audit_controls() {
    echo -e "\n${PURPLE}=== General Audit Controls Tests ===${NC}"
    
    # Audit log integrity
    if audit_integrity=$(check_api_compliance "/api/v1/admin/audit/integrity-check"); then
        if echo "$audit_integrity" | jq -e '.logs_tampered == false and .checksum_valid == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "AUDIT" "Log Integrity" "Audit logs maintain integrity" "CRITICAL"
        else
            log_compliance_test "FAIL" "AUDIT" "Log Integrity" "Audit log integrity compromised" "CRITICAL"
        fi
    else
        log_compliance_test "FAIL" "AUDIT" "Log Integrity" "Unable to verify audit log integrity" "CRITICAL"
    fi
    
    # Access control matrix
    if access_matrix=$(check_api_compliance "/api/v1/admin/compliance/access-matrix"); then
        if echo "$access_matrix" | jq -e '.role_based_access == true and .least_privilege == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "AUDIT" "Access Control Matrix" "Proper role-based access controls" "HIGH"
        else
            log_compliance_test "FAIL" "AUDIT" "Access Control Matrix" "Inadequate access control implementation" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "AUDIT" "Access Control Matrix" "Unable to verify access controls" "HIGH"
    fi
    
    # Data classification and handling
    if data_classification=$(check_api_compliance "/api/v1/admin/compliance/data-classification"); then
        if echo "$data_classification" | jq -e '.classification_implemented == true and .handling_procedures == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "AUDIT" "Data Classification" "Proper data classification and handling" "HIGH"
        else
            log_compliance_test "FAIL" "AUDIT" "Data Classification" "Inadequate data classification" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "AUDIT" "Data Classification" "Unable to verify data classification" "HIGH"
    fi
    
    # Incident response procedures
    if incident_response=$(check_api_compliance "/api/v1/admin/compliance/incident-response"); then
        if echo "$incident_response" | jq -e '.procedures_documented == true and .response_tested == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "AUDIT" "Incident Response" "Incident response procedures properly implemented" "HIGH"
        else
            log_compliance_test "FAIL" "AUDIT" "Incident Response" "Incident response procedures inadequate" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "AUDIT" "Incident Response" "Unable to verify incident response procedures" "HIGH"
    fi
    
    # Business continuity and disaster recovery
    if business_continuity=$(check_api_compliance "/api/v1/admin/compliance/business-continuity"); then
        if echo "$business_continuity" | jq -e '.bcdr_plan_exists == true and .recovery_tested == true' >/dev/null 2>&1; then
            log_compliance_test "PASS" "AUDIT" "Business Continuity" "BCDR plans properly implemented and tested" "HIGH"
        else
            log_compliance_test "FAIL" "AUDIT" "Business Continuity" "BCDR plans inadequate or untested" "HIGH"
        fi
    else
        log_compliance_test "FAIL" "AUDIT" "Business Continuity" "Unable to verify BCDR procedures" "HIGH"
    fi
}

# Generate compliance report
generate_compliance_report() {
    echo -e "\n${PURPLE}=== Generating Compliance Report ===${NC}"
    
    local report_file="$RESULTS_DIR/compliance_validation_report.html"
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>EPSX Compliance Validation Report</title>
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
        .regulation { margin: 20px 0; padding: 15px; border-left: 5px solid #2196f3; background-color: #f9f9f9; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .metric { display: inline-block; margin: 10px; padding: 10px; background-color: #f9f9f9; border-radius: 5px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>EPSX Compliance Validation Report</h1>
        <p><strong>Environment:</strong> $ENVIRONMENT</p>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Validation Framework Version:</strong> 1.0.0</p>
    </div>
    
    <div class="summary">
        <h2>Compliance Summary</h2>
        <div class="metric">
            <strong>Total Tests:</strong> $COMPLIANCE_TESTS_TOTAL
        </div>
        <div class="metric">
            <strong>Passed:</strong> <span class="pass">$COMPLIANCE_TESTS_PASSED</span>
        </div>
        <div class="metric">
            <strong>Failed:</strong> <span class="fail">$COMPLIANCE_TESTS_FAILED</span>
        </div>
        <div class="metric">
            <strong>Compliance Rate:</strong> $(echo "scale=1; $COMPLIANCE_TESTS_PASSED * 100 / $COMPLIANCE_TESTS_TOTAL" | bc)%
        </div>
    </div>
EOF

    if [[ ${#REGULATORY_FAILURES[@]} -gt 0 ]]; then
        cat >> "$report_file" << EOF
    <div style="background-color: #ffebee; padding: 15px; margin: 20px 0; border-radius: 5px; border-left: 5px solid #f44336;">
        <h3 class="critical">Critical Regulatory Violations</h3>
        <ul>
EOF
        for violation in "${REGULATORY_FAILURES[@]}"; do
            echo "            <li class=\"critical\">$violation</li>" >> "$report_file"
        done
        cat >> "$report_file" << EOF
        </ul>
        <p><strong>WARNING:</strong> These critical violations must be addressed before production deployment.</p>
    </div>
EOF
    fi

    cat >> "$report_file" << EOF
    <h2>Regulatory Compliance Details</h2>
    
    <div class="regulation">
        <h3>SOX (Sarbanes-Oxley Act)</h3>
        <p>Financial reporting and internal controls for public companies.</p>
    </div>
    
    <div class="regulation">
        <h3>PCI DSS (Payment Card Industry Data Security Standard)</h3>
        <p>Security standards for organizations handling credit card information.</p>
    </div>
    
    <div class="regulation">
        <h3>GDPR (General Data Protection Regulation)</h3>
        <p>European Union data protection and privacy regulation.</p>
    </div>
    
    <div class="regulation">
        <h3>CCPA (California Consumer Privacy Act)</h3>
        <p>California state privacy law for consumer data protection.</p>
    </div>
    
    <h2>Detailed Compliance Test Results</h2>
    <table>
        <thead>
            <tr>
                <th>Status</th>
                <th>Regulation</th>
                <th>Requirement</th>
                <th>Details</th>
                <th>Severity</th>
            </tr>
        </thead>
        <tbody>
EOF

    if [[ -f "$RESULTS_DIR/compliance_results.csv" ]]; then
        while IFS=',' read -r status regulation requirement message severity; do
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
                <td>$regulation</td>
                <td>$requirement</td>
                <td>$message</td>
                <td class="$severity_class">$severity</td>
            </tr>
EOF
        done < "$RESULTS_DIR/compliance_results.csv"
    fi

    cat >> "$report_file" << EOF
        </tbody>
    </table>
    
    <div style="margin-top: 40px; padding: 20px; background-color: #f4f4f4; border-radius: 5px;">
        <h3>Compliance Recommendations</h3>
        <ul>
            <li>Address all critical regulatory violations immediately</li>
            <li>Implement remediation plans for failed compliance tests</li>
            <li>Schedule regular compliance audits and reviews</li>
            <li>Maintain comprehensive documentation for audit purposes</li>
            <li>Train staff on compliance requirements and procedures</li>
            <li>Monitor regulatory changes and update controls accordingly</li>
        </ul>
    </div>
    
    <div style="margin-top: 20px; font-size: 0.9em; color: #666;">
        <p>This report was generated by the EPSX Compliance Validation System.</p>
        <p>For compliance questions, contact the compliance team at compliance@epsx.io</p>
    </div>
</body>
</html>
EOF

    echo -e "${GREEN}Compliance report generated: $report_file${NC}"
}

# Main execution
main() {
    echo "CSV Header" > "$RESULTS_DIR/compliance_results.csv"
    echo "Status,Regulation,Requirement,Message,Severity" >> "$RESULTS_DIR/compliance_results.csv"
    
    test_sox_compliance
    test_pci_dss_compliance
    test_gdpr_compliance
    test_ccpa_compliance
    test_general_audit_controls
    
    generate_compliance_report
    
    echo -e "\n${PURPLE}=== Compliance Validation Summary ===${NC}"
    echo -e "Total Tests: $COMPLIANCE_TESTS_TOTAL"
    echo -e "${GREEN}Passed: $COMPLIANCE_TESTS_PASSED${NC}"
    echo -e "${RED}Failed: $COMPLIANCE_TESTS_FAILED${NC}"
    
    local compliance_rate=$(echo "scale=1; $COMPLIANCE_TESTS_PASSED * 100 / $COMPLIANCE_TESTS_TOTAL" | bc)
    echo -e "Compliance Rate: $compliance_rate%"
    
    if [[ ${#REGULATORY_FAILURES[@]} -gt 0 ]]; then
        echo -e "\n${RED}❌ CRITICAL REGULATORY VIOLATIONS DETECTED:${NC}"
        for violation in "${REGULATORY_FAILURES[@]}"; do
            echo -e "${RED}  • $violation${NC}"
        done
        echo -e "\n${RED}Production deployment must be blocked until violations are resolved.${NC}"
        exit 2
    elif [[ $COMPLIANCE_TESTS_FAILED -gt 0 ]]; then
        echo -e "\n${YELLOW}⚠️  Some compliance tests failed. Review and remediation required.${NC}"
        exit 1
    else
        echo -e "\n${GREEN}✅ All compliance validation tests passed!${NC}"
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