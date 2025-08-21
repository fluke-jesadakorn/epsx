#!/bin/bash

# EPSX Hosts Validation Script
# Validates that required *.epsx.io domain entries exist in /etc/hosts
# NO MODIFICATIONS - read-only validation only

set -e

echo "🔍 Validating EPSX development domains..."

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

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This script is designed for macOS. Please adapt for your OS."
    exit 1
fi

# Required domains
REQUIRED_DOMAINS=("epsx.io" "admin.epsx.io" "api.epsx.io")
MISSING_DOMAINS=()

print_status "Checking /etc/hosts for required *.epsx.io domains..."

# Check each required domain
for domain in "${REQUIRED_DOMAINS[@]}"; do
    if grep -q "127.0.0.1.*$domain" /etc/hosts; then
        print_success "✅ $domain found in /etc/hosts"
    else
        print_error "❌ $domain NOT found in /etc/hosts"
        MISSING_DOMAINS+=("$domain")
    fi
done

# If any domains are missing, show setup instructions and exit
if [ ${#MISSING_DOMAINS[@]} -ne 0 ]; then
    echo
    print_error "🚫 EPSX domains not properly configured!"
    echo
    print_status "Missing domains: ${MISSING_DOMAINS[*]}"
    echo
    print_warning "⚠️  One-time setup required:"
    echo
    echo "1. Run the setup script to configure your hosts file:"
    echo "   ${YELLOW}./scripts/setup-domains.sh${NC}"
    echo
    echo "2. OR manually add these lines to /etc/hosts:"
    echo "   ${YELLOW}sudo nano /etc/hosts${NC}"
    echo
    echo "   Add these lines:"
    for domain in "${MISSING_DOMAINS[@]}"; do
        echo "   ${GREEN}127.0.0.1    $domain${NC}"
    done
    echo
    echo "3. OR use localhost development mode instead:"
    echo "   ${YELLOW}pnpm dev:localhost${NC}"
    echo
    print_warning "💡 After setup, /etc/hosts can be made read-only with:"
    echo "   ${YELLOW}./scripts/make-hosts-readonly.sh${NC}"
    echo
    exit 1
fi

# Validate IP addresses point to localhost
print_status "Validating IP addresses..."
for domain in "${REQUIRED_DOMAINS[@]}"; do
    resolved_ip=$(grep "127.0.0.1.*$domain" /etc/hosts | head -1 | awk '{print $1}')
    if [[ "$resolved_ip" == "127.0.0.1" ]]; then
        print_success "✅ $domain correctly points to 127.0.0.1"
    else
        print_warning "⚠️  $domain points to $resolved_ip (expected 127.0.0.1)"
    fi
done

# Check if hosts file is writable (optional warning)
if [[ -w /etc/hosts ]]; then
    print_warning "💡 /etc/hosts is writable. Consider making it read-only:"
    echo "   ${YELLOW}./scripts/make-hosts-readonly.sh${NC}"
else
    print_success "✅ /etc/hosts is read-only (protected)"
fi

print_success "🎉 All EPSX domains validated successfully!"
echo
print_status "Development domains ready:"
echo "  • Frontend: https://epsx.io"
echo "  • Admin:    https://admin.epsx.io"  
echo "  • API:      https://api.epsx.io"
echo
print_status "Starting Docker containers..."