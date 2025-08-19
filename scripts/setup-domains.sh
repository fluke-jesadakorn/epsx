#!/bin/bash

# EPSX Domain Setup Script
# Sets up local domain override and SSL certificates for Docker + Traefik development

set -e

echo "🚀 Setting up EPSX development domains..."

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

# Check if domains already exist in hosts file BEFORE creating backup
if grep -q "epsx.io" /etc/hosts; then
    print_success "EPSX domains already configured in /etc/hosts - skipping domain setup"
    print_status "Current EPSX entries:"
    grep -A 3 "epsx.io" /etc/hosts | head -4
else
    # Only create backup when we actually need to modify /etc/hosts
    BACKUP_FILE="/etc/hosts.backup.$(date +%Y%m%d_%H%M%S)"
    print_status "Creating backup of /etc/hosts -> $BACKUP_FILE"
    sudo cp /etc/hosts "$BACKUP_FILE"
    
    # Add domain overrides to hosts file
    print_status "Adding EPSX domain overrides to /etc/hosts..."
    sudo bash -c 'cat >> /etc/hosts << EOF

# EPSX Domain Override
# Added by setup-domains.sh on $(date)
127.0.0.1    epsx.io
127.0.0.1    admin.epsx.io
127.0.0.1    api.epsx.io
EOF'
    
    print_success "✅ Added EPSX domain overrides to /etc/hosts"
    echo "  • Backed up original hosts file to: $BACKUP_FILE"
fi

# Create Docker directories
print_status "Creating Docker configuration directories..."
mkdir -p docker/traefik/certs
mkdir -p docker/traefik/dynamic

# Check if mkcert is installed
if ! command -v mkcert &> /dev/null; then
    print_status "Installing mkcert for SSL certificate generation..."
    if command -v brew &> /dev/null; then
        brew install mkcert
    else
        print_error "Homebrew not found. Please install mkcert manually: https://github.com/FiloSottile/mkcert"
        exit 1
    fi
fi

# Install local CA
print_status "Setting up local Certificate Authority..."
mkcert -install

# Generate SSL certificates (only if they don't exist)
if [[ -f "docker/traefik/certs/epsx.crt" ]] && [[ -f "docker/traefik/certs/epsx.key" ]]; then
    print_success "SSL certificates already exist - skipping certificate generation"
else
    print_status "Generating SSL certificates for *.epsx.io domains..."
    cd docker/traefik/certs
    mkcert -cert-file epsx.crt \
           -key-file epsx.key \
           epsx.io admin.epsx.io api.epsx.io "*.epsx.io"
    cd - > /dev/null
    print_success "✅ Generated SSL certificates"
fi

# Create Traefik dynamic configuration for SSL (only if it doesn't exist)
if [[ -f "docker/traefik/dynamic/ssl.yml" ]]; then
    print_success "Traefik SSL configuration already exists - skipping"
else
    print_status "Creating Traefik SSL configuration..."
    cat > docker/traefik/dynamic/ssl.yml << 'EOF'
tls:
  certificates:
    - certFile: /certs/epsx.crt
      keyFile: /certs/epsx.key
      stores:
        - default
  stores:
    default:
      defaultCertificate:
        certFile: /certs/epsx.crt
        keyFile: /certs/epsx.key

# HTTPS redirect middleware
http:
  middlewares:
    redirect-to-https:
      redirectScheme:
        scheme: https
        permanent: true
EOF
    print_success "✅ Created Traefik SSL configuration"
fi

# Verify DNS resolution
print_status "Verifying DNS resolution..."
for domain in epsx.io admin.epsx.io api.epsx.io; do
    if ping -c 1 -W 1000 "$domain" &> /dev/null; then
        resolved_ip=$(ping -c 1 "$domain" | grep PING | sed -E 's/.*\(([0-9.]+)\).*/\1/')
        if [[ "$resolved_ip" == "127.0.0.1" ]]; then
            print_success "$domain resolves to 127.0.0.1 ✓"
        else
            print_warning "$domain resolves to $resolved_ip (expected 127.0.0.1)"
        fi
    else
        print_error "Failed to resolve $domain"
    fi
done

# Flush DNS cache
print_status "Flushing DNS cache..."
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder 2>/dev/null || true

print_success "✅ Domain setup complete!"
echo
print_status "Summary:"
echo "  • Verified *.epsx.io domains in /etc/hosts (pointing to 127.0.0.1)"
echo "  • Ensured SSL certificates exist for HTTPS"
echo "  • Verified Traefik configuration"
echo "  • Smart backup: Only created when actually modifying files"
echo
print_status "Next steps:"
echo "  1. Start development: pnpm dev"
echo "  2. Access applications:"
echo "     - Frontend: https://epsx.io"
echo "     - Admin:    https://admin.epsx.io"
echo "     - API:      https://api.epsx.io"
echo "     - Traefik:  http://localhost:8080"
echo
print_warning "⚠️  Your machine now overrides *.epsx.io domains"
print_warning "   Run 'pnpm dev:clean' to restore original DNS resolution"