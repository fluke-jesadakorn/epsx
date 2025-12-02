#!/bin/bash

# Detect IP (macOS/Linux)
# Tries to find the first non-loopback IPv4 address
IP=$(ifconfig | grep "inet " | grep -v 127.0.0.1 | grep -v 169.254 | head -n 1 | awk '{print $2}')

if [ -z "$IP" ]; then
  echo "Could not detect IP. Defaulting to 127.0.0.1"
  IP="127.0.0.1"
fi

echo "Configuring project for Local IP: $IP"

# Helper function to update env vars
# Works on macOS (BSD sed) and Linux (GNU sed)
function update_env {
    local file=$1
    local key=$2
    local value=$3
    
    # Ensure file exists
    if [ ! -f "$file" ]; then
        touch "$file"
    fi

    # Check if key exists
    if grep -q "^$key=" "$file"; then
        # Update existing key using perl for better cross-platform compatibility than sed
        perl -i -pe "s|^$key=.*|$key=$value|" "$file"
    else
        # Append new key
        # Ensure there is a newline before appending if the file ends with text
        if [ -s "$file" ] && [ "$(tail -c1 "$file" | wc -l)" -eq 0 ]; then
            echo "" >> "$file"
        fi
        echo "$key=$value" >> "$file"
    fi
}

# --- Root Configuration ---
ROOT_ENV=".env"
echo "Updating root $ROOT_ENV..."
update_env "$ROOT_ENV" "BACKEND_URL" "http://$IP:8080"
update_env "$ROOT_ENV" "FRONTEND_URL" "http://$IP:3000"
update_env "$ROOT_ENV" "ADMIN_FRONTEND_URL" "http://$IP:3001"
update_env "$ROOT_ENV" "NEXT_PUBLIC_BACKEND_URL" "http://$IP:8080"
update_env "$ROOT_ENV" "NEXT_PUBLIC_APP_URL" "http://$IP:3000"
update_env "$ROOT_ENV" "NEXT_PUBLIC_ADMIN_URL" "http://$IP:3001"

# --- Backend Configuration ---
BACKEND_ENV="apps/backend/.env"
echo "Updating $BACKEND_ENV..."
if [ ! -f "$BACKEND_ENV" ] && [ -f "apps/backend/.env.example" ]; then
    cp apps/backend/.env.example "$BACKEND_ENV"
fi

update_env "$BACKEND_ENV" "HOST" "0.0.0.0"
update_env "$BACKEND_ENV" "PORT" "8080"
update_env "$BACKEND_ENV" "BACKEND_URL" "http://$IP:8080"
update_env "$BACKEND_ENV" "FRONTEND_URL" "http://$IP:3000"
update_env "$BACKEND_ENV" "ADMIN_FRONTEND_URL" "http://$IP:3001"

# --- Frontend Configuration ---
FRONTEND_ENV="apps/frontend/.env.local"
echo "Updating $FRONTEND_ENV..."
# We don't blindly copy root .env.example because it might contain backend secrets not needed here, 
# but for development convenience, if it's empty, we might want base values.
# For now, just set the critical connection URLs.

update_env "$FRONTEND_ENV" "NEXT_PUBLIC_BACKEND_URL" "http://$IP:8080"
update_env "$FRONTEND_ENV" "NEXT_PUBLIC_APP_URL" "http://$IP:3000"
update_env "$FRONTEND_ENV" "NEXT_PUBLIC_ADMIN_URL" "http://$IP:3001"

# --- Admin Frontend Configuration ---
ADMIN_ENV="apps/admin-frontend/.env.local"
echo "Updating $ADMIN_ENV..."

update_env "$ADMIN_ENV" "NEXT_PUBLIC_BACKEND_URL" "http://$IP:8080"
update_env "$ADMIN_ENV" "NEXT_PUBLIC_APP_URL" "http://$IP:3001"

echo "---------------------------------------------------"
echo "✅ Network configuration complete!"
echo "   Backend listening on: 0.0.0.0:8080 (Access via http://$IP:8080)"
echo "   Frontend configured for: http://$IP:3000"
echo "   Admin configured for: http://$IP:3001"
echo ""
echo "You can now start the services:"
echo "   Backend: cd apps/backend && cargo run"
echo "   Frontend: cd apps/frontend && npm run dev"
echo "   Admin: cd apps/admin-frontend && npm run dev"
echo "---------------------------------------------------"
