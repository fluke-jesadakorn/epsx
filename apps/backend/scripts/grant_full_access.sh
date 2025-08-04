#!/bin/bash

# Script to grant full system access to a user
# Usage: ./scripts/grant_full_access.sh <email> [reason]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
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

# Check if email is provided
if [ -z "$1" ]; then
    print_error "Email address is required"
    echo "Usage: $0 <email> [reason]"
    echo "Example: $0 jesadakorn.kirtnu@gmail.com 'System administrator access'"
    exit 1
fi

EMAIL="$1"
REASON="${2:-Admin script promotion to full access}"

print_info "Granting full system access to: $EMAIL"
print_info "Reason: $REASON"

# Change to backend directory
cd "$(dirname "$0")/.."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Not in backend directory. Please run from apps/backend/"
    exit 1
fi

# Load environment variables if .env exists
if [ -f ".env" ]; then
    print_info "Loading environment variables from .env (skipping multiline values)"
    # Use a safer approach to avoid multiline key issues
    set -a
    source .env 2>/dev/null || print_warning "Some .env variables could not be loaded"
    set +a
fi

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    print_warning "DATABASE_URL not set, will use config values"
fi

print_info "Building promote_admin binary..."
cargo build --bin promote_admin --release

if [ $? -ne 0 ]; then
    print_error "Failed to build promote_admin binary"
    exit 1
fi

print_success "Binary built successfully"

print_info "Promoting user to SuperAdmin with ALL permissions..."

# Run the promotion with super admin flag for full access
./target/release/promote_admin \
    --email "$EMAIL" \
    --reason "$REASON" \
    --super-admin

if [ $? -eq 0 ]; then
    print_success "Successfully granted full system access to $EMAIL"
    print_info "The user now has:"
    echo "  🔑 SuperAdmin role"
    echo "  🌟 ALL system permissions including:"
    echo "    • READ_ALL - Full read access to all data"
    echo "    • WRITE_ALL - Full write access to all data"
    echo "    • DELETE_ALL - Full delete access"
    echo "    • MANAGE_USERS - User management capabilities"
    echo "    • DELETE_USERS - User deletion capabilities"
    echo "    • MANAGE_SYSTEM - System management"
    echo "    • MANAGE_ADMIN - Admin management"
    echo "    • MODERATE_CONTENT - Content moderation"
    echo "    • MODERATE_USERS - User moderation"
    echo "    • WRITE_CONTENT - Content creation"
    echo "    • ACCESS_PREMIUM - Premium feature access"
    echo "    • ACCESS_PREMIUM_FEATURES - Premium features"
    echo "    • READ_PREMIUM - Premium content read"
    echo "    • READ_ADVANCED_ANALYTICS - Advanced analytics"
    echo "    • READ_ALL_DATA - All data access"
    echo "    • WRITE_USER_DATA - User data modification"
    echo "    • READ_USER_REPORTS - User report access"
    print_warning "This user now has FULL SYSTEM ACCESS. Use with caution!"
else
    print_error "Failed to grant access. Check the error messages above."
    exit 1
fi