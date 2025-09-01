#!/bin/bash

echo "Fixing import syntax errors..."

# Fix files where imports were inserted into middle of existing use statements
files_to_fix=(
    "src/web/admin/analytics_handlers.rs"
    "src/web/admin/database_role_management.rs" 
    "src/auth/tokens.rs"
    "src/auth/cleanup.rs"
    "src/auth/refresh_tokens.rs"
    "src/auth/refresh_token_service.rs"
    "src/auth/session_security_service.rs"
    "src/web/middleware/error_handling.rs"
    "src/web/admin/handlers.rs"
    "src/web/health/mod.rs"
    "src/web/oidc/token.rs"
    "src/web/compliance/mod.rs"
    "src/web/realtime/handlers.rs"
    "src/web/analytics/eps/dto.rs"
    "src/web/analytics/eps/transform.rs"
    "src/dom/values/auth.rs"
    "src/dom/entities/auth.rs"
    "src/config/env.rs"
    "src/web/validation/middleware.rs"
    "src/web/middleware/rate_limiter.rs"
    "src/infra/services/market_data_service.rs"
    "src/infra/services/email_service.rs"
    "src/infra/db/postgres/mod.rs"
    "src/infra/db/diesel/repos/audit_repo.rs"
    "src/infra/container/services_module.rs"
    "src/app/ports/events.rs"
    "src/app/dtos/auth.rs"
)

for file in "${files_to_fix[@]}"; do
    if [ -f "$file" ]; then
        echo "Fixing $file"
        # Remove any imports that got added incorrectly in the middle of use statements
        sed -i '' '
        # Fix pattern: use statement with import inserted in middle
        /^use.*{$/{
            N
            s/use chrono::{DateTime, Utc};\n//
            s/use uuid::Uuid;\n//
            s/use std::net::IpAddr;\n//
        }
        ' "$file"
    fi
done

echo "Done fixing syntax errors!"