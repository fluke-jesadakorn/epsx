#!/bin/bash

# EPSX Hosts Protection Script
# Makes /etc/hosts read-only to prevent accidental modifications
# Provides instructions for reverting if needed

set -e

echo "🔒 Making /etc/hosts read-only for protection..."

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

# Check current permissions
current_perms=$(ls -la /etc/hosts | awk '{print $1}')
print_status "Current /etc/hosts permissions: $current_perms"

# Check if already read-only
if [[ ! -w /etc/hosts ]]; then
    print_warning "/etc/hosts is already read-only"
    print_status "Current protection level: $(ls -la /etc/hosts | awk '{print $1}')"
    exit 0
fi

# Validate EPSX domains exist before making read-only
print_status "Validating EPSX domains before making read-only..."
if ! grep -q "127.0.0.1.*epsx.io" /etc/hosts; then
    print_error "EPSX domains not found in /etc/hosts!"
    print_warning "Run ./scripts/setup-domains.sh first to configure domains"
    exit 1
fi

# Create backup before making changes
BACKUP_FILE="/etc/hosts.readonly.backup.$(date +%Y%m%d_%H%M%S)"
print_status "Creating backup: $BACKUP_FILE"
sudo cp /etc/hosts "$BACKUP_FILE"

# Make hosts file read-only
print_status "Setting /etc/hosts to read-only..."
sudo chmod 444 /etc/hosts

# Verify the change
new_perms=$(ls -la /etc/hosts | awk '{print $1}')
if [[ ! -w /etc/hosts ]]; then
    print_success "✅ /etc/hosts is now read-only!"
    print_status "New permissions: $new_perms"
    print_status "Backup created: $BACKUP_FILE"
else
    print_error "Failed to make /etc/hosts read-only"
    exit 1
fi

echo
print_success "🎉 EPSX hosts file protection enabled!"
echo
print_status "Benefits:"
echo "  • No more password prompts during development"
echo "  • Protection against accidental modifications"
echo "  • Faster development startup"
echo
print_warning "⚠️  If you need to modify /etc/hosts in the future:"
echo
echo "1. Make it writable again:"
echo "   ${YELLOW}sudo chmod 644 /etc/hosts${NC}"
echo
echo "2. Make your changes"
echo
echo "3. Make it read-only again:"
echo "   ${YELLOW}sudo chmod 444 /etc/hosts${NC}"
echo
print_status "Or restore from backup:"
echo "   ${YELLOW}sudo cp $BACKUP_FILE /etc/hosts${NC}"
echo "   ${YELLOW}sudo chmod 644 /etc/hosts${NC}"
echo