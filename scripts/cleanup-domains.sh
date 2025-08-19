#!/bin/bash

# EPSX Domain Cleanup Script
# Removes local domain overrides and restores original DNS resolution

set -e

echo "🧹 Cleaning up EPSX development domains..."

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

# Check if EPSX domains exist in hosts file
if ! grep -q "# EPSX Domain Override" /etc/hosts; then
    print_warning "No EPSX domain overrides found in /etc/hosts"
    print_status "Nothing to clean up in hosts file."
else
    # Create backup before cleanup
    BACKUP_FILE="/etc/hosts.cleanup.backup.$(date +%Y%m%d_%H%M%S)"
    print_status "Creating backup of /etc/hosts -> $BACKUP_FILE"
    sudo cp /etc/hosts "$BACKUP_FILE"

    # Remove EPSX domain overrides from hosts file
    print_status "Removing EPSX domain overrides from /etc/hosts..."
    sudo sed -i.tmp '/# EPSX Domain Override/,/api\.epsx\.io/d' /etc/hosts
    sudo rm -f /etc/hosts.tmp

    print_success "Removed EPSX domain overrides from /etc/hosts"
fi

# Stop Docker containers if running
print_status "Stopping Docker containers..."
if docker-compose ps -q 2>/dev/null | grep -q .; then
    docker-compose down
    print_success "Stopped Docker containers"
else
    print_status "No Docker containers were running"
fi

# Optional: Clean up Docker volumes
read -p "Do you want to remove Docker volumes? This will delete database data. [y/N]: " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Removing Docker volumes..."
    docker-compose down -v --remove-orphans
    print_success "Removed Docker volumes"
fi

# Optional: Clean up SSL certificates and Docker configs
read -p "Do you want to remove SSL certificates and Docker configs? [y/N]: " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Cleaning up Docker configuration files..."
    if [[ -d "docker" ]]; then
        rm -rf docker/
        print_success "Removed docker/ directory"
    fi
fi

# Flush DNS cache
print_status "Flushing DNS cache..."
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder 2>/dev/null || true

# Verify DNS resolution is restored
print_status "Verifying DNS resolution..."
for domain in epsx.io admin.epsx.io api.epsx.io; do
    if ping -c 1 -W 1000 "$domain" &> /dev/null; then
        resolved_ip=$(ping -c 1 "$domain" | grep PING | sed -E 's/.*\(([0-9.]+)\).*/\1/')
        if [[ "$resolved_ip" != "127.0.0.1" ]]; then
            print_success "$domain now resolves to $resolved_ip (restored) ✓"
        else
            print_warning "$domain still resolves to 127.0.0.1 (may need manual cleanup)"
        fi
    else
        print_warning "Unable to resolve $domain (this is normal if the real domain doesn't exist)"
    fi
done

print_success "✅ Domain cleanup complete!"
echo
print_status "Summary:"
echo "  • Removed *.epsx.io domain overrides from /etc/hosts"
echo "  • Stopped Docker development stack"
echo "  • Flushed DNS cache"
echo
print_status "Your system now resolves *.epsx.io domains normally"
echo "  • epsx.io will resolve to the real server (if it exists)"
echo "  • Local development is disabled"
echo
print_status "To resume development:"
echo "  • Run: pnpm dev (will automatically set up domains)"
echo "  • Or run: pnpm dev:localhost (use localhost:port URLs)"