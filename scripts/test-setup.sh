#!/bin/bash

# Test script to validate the Docker + Traefik setup without requiring sudo

echo "🧪 Testing EPSX Docker + Traefik setup..."

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

# Test 1: Docker Compose Configuration
print_status "Testing Docker Compose configuration..."
if docker-compose config > /dev/null 2>&1; then
    print_success "Docker Compose configuration is valid ✓"
else
    print_error "Docker Compose configuration is invalid ✗"
    exit 1
fi

# Test 2: Check if setup script exists and is executable
print_status "Checking setup script..."
if [[ -x "./scripts/setup-domains.sh" ]]; then
    print_success "Domain setup script found and executable ✓"
else
    print_error "Domain setup script not found or not executable ✗"
    exit 1
fi

# Test 3: Check if cleanup script exists and is executable
print_status "Checking cleanup script..."
if [[ -x "./scripts/cleanup-domains.sh" ]]; then
    print_success "Domain cleanup script found and executable ✓"
else
    print_error "Domain cleanup script not found or not executable ✗"
    exit 1
fi

# Test 4: Validate environment files
print_status "Checking environment files..."
for app in frontend admin-frontend backend; do
    env_file="apps/$app/.env"
    if [[ -f "$env_file" ]] && grep -q "epsx.io" "$env_file"; then
        print_success "Environment file $env_file has epsx.io domains ✓"
    else
        print_warning "Environment file $env_file missing or incorrect domains ⚠"
    fi
done

# Test 5: Check package.json scripts
print_status "Checking package.json scripts..."
if grep -q "docker-compose up --build" package.json && grep -q "setup-domains.sh" package.json; then
    print_success "Package.json has correct Docker + Traefik scripts ✓"
else
    print_error "Package.json missing correct scripts ✗"
    exit 1
fi

# Test 6: Check if mkcert is available
print_status "Checking mkcert availability..."
if command -v mkcert &> /dev/null; then
    print_success "mkcert is installed ✓"
    mkcert_version=$(mkcert -version)
    print_status "mkcert version: $mkcert_version"
else
    print_warning "mkcert not installed - will be installed by setup script ⚠"
fi

# Test 7: Check Docker availability
print_status "Checking Docker availability..."
if docker info > /dev/null 2>&1; then
    print_success "Docker is running ✓"
    docker_version=$(docker --version)
    print_status "$docker_version"
else
    print_error "Docker is not running ✗"
    print_status "Please start Docker and try again"
    exit 1
fi

# Test 8: Create test Docker directories (simulate setup)
print_status "Testing directory creation..."
mkdir -p docker/traefik/certs
mkdir -p docker/traefik/dynamic
print_success "Created test Docker directories ✓"

# Test 9: Test SSL configuration template
print_status "Testing SSL configuration template..."
cat > docker/traefik/dynamic/test-ssl.yml << 'EOF'
tls:
  certificates:
    - certFile: /certs/epsx.crt
      keyFile: /certs/epsx.key
EOF
print_success "SSL configuration template created ✓"

# Test 10: Cleanup test files
print_status "Cleaning up test files..."
rm -rf docker/
print_success "Test cleanup complete ✓"

print_success "✅ All tests passed!"
echo
print_status "Setup is ready. Next steps:"
echo "  1. Run: pnpm dev (will setup domains and start services)"
echo "  2. Access: https://epsx.io, https://admin.epsx.io, https://api.epsx.io"
echo "  3. View Traefik: http://localhost:8080"
echo
print_status "The setup script will:"
echo "  • Modify /etc/hosts (requires sudo)"
echo "  • Install mkcert (if not present)"
echo "  • Generate SSL certificates"
echo "  • Start Docker services"