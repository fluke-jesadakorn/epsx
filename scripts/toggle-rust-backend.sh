#!/bin/bash

# Script to toggle Rust backend on/off
# Usage: ./scripts/toggle-rust-backend.sh [on|off|status]

ACTION=${1:-status}

case $ACTION in
  off)
    echo "Disabling Rust backend..."
    cp docker-compose.dev.yml docker-compose.dev.yml.backup
    cp docker-compose.prod.yml docker-compose.prod.yml.backup
    
    # Comment out backend services in dev
    sed -i.bak '/backend-dev:/,/^[[:space:]]*networks:/s/^/# /' docker-compose.dev.yml
    
    # Comment out backend services in prod
    sed -i.bak '/backend-prod:/,/^[[:space:]]*networks:/s/^/# /' docker-compose.prod.yml
    
    echo "Rust backend has been temporarily disabled."
    echo "Original files backed up as *.backup"
    ;;
    
  on)
    echo "Re-enabling Rust backend..."
    if [ -f "docker-compose.dev.yml.backup" ]; then
      cp docker-compose.dev.yml.backup docker-compose.dev.yml
    else
      echo "No backup found for dev file. Manual restoration needed."
    fi
    
    if [ -f "docker-compose.prod.yml.backup" ]; then
      cp docker-compose.prod.yml.backup docker-compose.prod.yml
    else
      echo "No backup found for prod file. Manual restoration needed."
    fi
    
    echo "Rust backend has been re-enabled."
    ;;
    
  status)
    if grep -q "^  backend-dev:" docker-compose.dev.yml; then
      echo "Rust backend is currently ENABLED"
    else
      echo "Rust backend is currently DISABLED"
    fi
    ;;
    
  *)
    echo "Usage: $0 [on|off|status]"
    echo "  on     - Enable Rust backend"
    echo "  off    - Disable Rust backend"
    echo "  status - Check current status"
    exit 1
    ;;
esac
