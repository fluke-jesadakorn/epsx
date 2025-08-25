#!/bin/bash

# Assign specific admin modules to a user
# Usage: ./assign-admin-modules.sh user@example.com "system_admin,user_management,analytics_access"

set -e

if [ $# -ne 2 ]; then
    echo "Usage: $0 <email> <comma-separated-modules>"
    echo "Example: $0 admin@example.com \"system_admin,user_management,analytics_access\""
    echo ""
    echo "Available modules:"
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
MODULES=$2

echo "🔧 Assigning admin modules to user: $EMAIL"
echo "📋 Modules: $MODULES"
echo "======================================================="

cd "$(dirname "$0")/../apps/backend"

# Check if user exists and build with CLI tools feature
echo "🏗️  Building CLI tool..."
cargo build --release --bin assign_iam --features cli-tools

echo "🚀 Running admin module assignment..."
cargo run --release --bin assign_iam --features cli-tools -- --email "$EMAIL" --modules "$MODULES"

echo ""
echo "✅ Admin module assignment completed!"
echo "💡 You can verify the assignment by checking the user_admin_roles table in the database."