#!/bin/bash

# Promote user to full admin with all available modules
# Usage: ./promote-admin.sh user@example.com

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <email>"
    echo "Example: $0 admin@example.com"
    echo ""
    echo "This script will assign ALL available admin modules to the user:"
    echo "  - system_admin"
    echo "  - user_management"
    echo "  - analytics_access" 
    echo "  - security_management"
    echo "  - audit_logs"
    echo "  - financial_oversight"
    echo "  - content_management"
    echo "  - support_access"
    echo "  - database_management"
    echo "  - developer_portal"
    echo "  - module_management"
    exit 1
fi

EMAIL=$1

echo "👑 Promoting user to FULL ADMIN: $EMAIL"
echo "📋 This will assign ALL available admin modules"
echo "======================================================="

read -p "Are you sure you want to promote $EMAIL to full admin? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Operation cancelled."
    exit 1
fi

cd "$(dirname "$0")/../apps/backend"

# Build with CLI tools feature
echo "🏗️  Building CLI tool..."
cargo build --release --bin promote_admin --features cli-tools

echo "🚀 Running full admin promotion..."
cargo run --release --bin promote_admin --features cli-tools -- --email "$EMAIL"

echo ""
echo "👑 Full admin promotion completed!"
echo "💡 User $EMAIL now has access to all admin functionality."
echo "💡 You can verify by checking the user_admin_roles table in the database."