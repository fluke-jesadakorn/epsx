#!/bin/bash

# EPSX Environment Manager
# Central environment detection and management system

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Script directory and root detection
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Environment precedence order (Next.js style)
# .env.local (highest priority - gitignored)
# .env.${NODE_ENV}
# .env.defaults (lowest priority)

# Function: Print usage
usage() {
    echo -e "${BLUE}EPSX Environment Manager${NC}"
    echo
    echo "Usage: $0 <command> [options]"
    echo
    echo "Commands:"
    echo -e "  ${GREEN}development${NC}  Switch to development environment"
    echo -e "  ${GREEN}staging${NC}      Switch to staging environment" 
    echo -e "  ${GREEN}production${NC}   Switch to production environment"
    echo -e "  ${GREEN}cloud-run${NC}    Switch to Cloud Run environment"
    echo -e "  ${GREEN}status${NC}       Show current environment status"
    echo -e "  ${GREEN}detect${NC}       Auto-detect current environment"
    echo -e "  ${GREEN}validate${NC}     Validate current environment"
    echo -e "  ${GREEN}list${NC}         List all available environments"
    echo
    echo "Options:"
    echo -e "  ${YELLOW}--app=APP${NC}    Apply to specific app (frontend, admin-frontend, backend)"
    echo -e "  ${YELLOW}--verbose${NC}    Verbose output"
    echo -e "  ${YELLOW}--dry-run${NC}    Show what would be done"
}

# Function: Detect current environment
detect_environment() {
    local detected="development"
    
    # Check CI/CD context
    if [[ "${GITHUB_ACTIONS:-}" == "true" ]] || [[ "${CI:-}" == "true" ]]; then
        if [[ "${GITHUB_REF:-}" == "refs/heads/main" ]] || [[ "${GITHUB_REF:-}" == "refs/heads/master" ]]; then
            detected="production"
        elif [[ "${GITHUB_REF:-}" == "refs/heads/staging" ]]; then
            detected="staging"
        else
            detected="staging"
        fi
    fi
    
    # Check environment variables
    if [[ "${BUILD_TARGET:-}" == "cloud-run" ]]; then
        detected="cloud-run"
    elif [[ "${ENV:-}" != "" ]]; then
        detected="${ENV}"
    elif [[ "${NODE_ENV:-}" == "production" ]]; then
        if [[ "${RUST_ENV:-}" == "staging" ]] || [[ "${ENV:-}" == "staging" ]]; then
            detected="staging"
        else
            detected="production"
        fi
    elif [[ "${NODE_ENV:-}" == "development" ]] || [[ "${NODE_ENV:-}" == "" ]]; then
        detected="development"
    fi
    
    # Check git branch (if available)
    if command -v git >/dev/null 2>&1 && git rev-parse --git-dir >/dev/null 2>&1; then
        local branch
        branch=$(git branch --show-current 2>/dev/null || echo "")
        case "$branch" in
            main|master)
                if [[ "$detected" == "development" ]]; then
                    detected="production"
                fi
                ;;
            staging)
                if [[ "$detected" == "development" ]]; then
                    detected="staging"
                fi
                ;;
        esac
    fi
    
    echo "$detected"
}

# Function: Load environment files in precedence order
load_environment() {
    local env_type="$1"
    local app_path="${2:-}"
    local verbose="${3:-false}"
    
    if [[ "$verbose" == "true" ]]; then
        echo -e "${BLUE}Loading environment: $env_type${NC}"
    fi
    
    # Load in reverse precedence order (lowest to highest)
    local files_to_load=()
    
    # 1. Defaults (lowest priority)
    if [[ -f "$ROOT_DIR/.env.defaults" ]]; then
        files_to_load+=("$ROOT_DIR/.env.defaults")
    fi
    
    # 2. Environment-specific
    if [[ -f "$ROOT_DIR/.env.$env_type" ]]; then
        files_to_load+=("$ROOT_DIR/.env.$env_type")
    fi
    
    # 3. Shared environment-specific
    if [[ -f "$ROOT_DIR/.env.shared.$env_type" ]]; then
        files_to_load+=("$ROOT_DIR/.env.shared.$env_type")
    fi
    
    # 4. Original shared (for backward compatibility)
    if [[ -f "$ROOT_DIR/.env.shared" ]]; then
        files_to_load+=("$ROOT_DIR/.env.shared")
    fi
    
    # 5. App-specific environment (if specified)
    if [[ -n "$app_path" ]] && [[ -f "$app_path/.env.$env_type" ]]; then
        files_to_load+=("$app_path/.env.$env_type")
    fi
    
    # 6. Local overrides (highest priority - gitignored)
    if [[ -f "$ROOT_DIR/.env.local" ]]; then
        files_to_load+=("$ROOT_DIR/.env.local")
    fi
    
    if [[ -n "$app_path" ]] && [[ -f "$app_path/.env.local" ]]; then
        files_to_load+=("$app_path/.env.local")
    fi
    
    # Export environment variables from all files
    for file in "${files_to_load[@]}"; do
        if [[ "$verbose" == "true" ]]; then
            echo -e "  Loading: ${file}"
        fi
        set -a
        source "$file"
        set +a
    done
    
    if [[ "$verbose" == "true" ]]; then
        echo -e "${GREEN}Environment loaded successfully${NC}"
    fi
}

# Function: Switch environment
switch_environment() {
    local target_env="$1"
    local app_filter="${2:-}"
    local dry_run="${3:-false}"
    local verbose="${4:-false}"
    
    echo -e "${BLUE}Switching to $target_env environment...${NC}"
    
    # Validate target environment
    if [[ ! -f "$ROOT_DIR/.env.$target_env" ]]; then
        echo -e "${RED}Error: Environment '$target_env' not found${NC}"
        echo -e "Available environments:"
        list_environments
        return 1
    fi
    
    # Create/update .env.local with environment override
    local env_local_content="# Auto-generated by env-manager.sh
# This file overrides environment detection
NODE_ENV=$([[ "$target_env" == "development" ]] && echo "development" || echo "production")
ENV=$target_env
EPSX_CURRENT_ENV=$target_env
"
    
    if [[ "$dry_run" == "true" ]]; then
        echo -e "${YELLOW}[DRY RUN] Would write to .env.local:${NC}"
        echo "$env_local_content"
        return 0
    fi
    
    echo "$env_local_content" > "$ROOT_DIR/.env.local"
    
    # Load the environment to test it
    load_environment "$target_env" "" "$verbose"
    
    echo -e "${GREEN}✅ Switched to $target_env environment${NC}"
    
    # Show current status
    show_status "$verbose"
}

# Function: Show current environment status
show_status() {
    local verbose="${1:-false}"
    local current_env
    current_env=$(detect_environment)
    
    echo -e "${BLUE}Current Environment Status:${NC}"
    echo -e "  Environment: ${GREEN}$current_env${NC}"
    echo -e "  NODE_ENV: ${YELLOW}${NODE_ENV:-'unset'}${NC}"
    echo -e "  ENV: ${YELLOW}${ENV:-'unset'}${NC}"
    echo -e "  RUST_ENV: ${YELLOW}${RUST_ENV:-'unset'}${NC}"
    
    if [[ "$verbose" == "true" ]]; then
        echo
        echo -e "${BLUE}Environment Details:${NC}"
        echo -e "  Database: ${YELLOW}${DATABASE_URL:-'unset'}${NC}"
        echo -e "  Frontend URL: ${YELLOW}${FRONTEND_URL:-'unset'}${NC}"
        echo -e "  Backend URL: ${YELLOW}${BACKEND_URL:-'unset'}${NC}"
        echo -e "  Admin URL: ${YELLOW}${ADMIN_FRONTEND_URL:-'unset'}${NC}"
    fi
}

# Function: List available environments
list_environments() {
    echo -e "${BLUE}Available Environments:${NC}"
    for env_file in "$ROOT_DIR"/.env.*; do
        if [[ -f "$env_file" ]] && [[ "$env_file" != *".shared"* ]] && [[ "$env_file" != *".local" ]] && [[ "$env_file" != *".defaults" ]]; then
            local env_name
            env_name=$(basename "$env_file" | sed 's/^\.env\.//')
            echo -e "  ${GREEN}$env_name${NC}"
        fi
    done
}

# Function: Validate environment
validate_environment() {
    local current_env
    current_env=$(detect_environment)
    
    echo -e "${BLUE}Validating $current_env environment...${NC}"
    
    # Load current environment
    load_environment "$current_env"
    
    # Check required variables
    local required_vars=("DATABASE_URL" "FRONTEND_URL" "BACKEND_URL" "NEXTAUTH_SECRET")
    local missing_vars=()
    
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            missing_vars+=("$var")
        fi
    done
    
    if [[ ${#missing_vars[@]} -gt 0 ]]; then
        echo -e "${RED}❌ Validation failed. Missing required variables:${NC}"
        for var in "${missing_vars[@]}"; do
            echo -e "  ${RED}- $var${NC}"
        done
        return 1
    else
        echo -e "${GREEN}✅ Environment validation passed${NC}"
    fi
}

# Main execution
main() {
    local command="${1:-}"
    local app_filter=""
    local verbose=false
    local dry_run=false
    
    # Parse arguments
    shift 2>/dev/null || true
    while [[ $# -gt 0 ]]; do
        case $1 in
            --app=*)
                app_filter="${1#*=}"
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --dry-run)
                dry_run=true
                shift
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                usage
                exit 1
                ;;
        esac
    done
    
    # Execute command
    case "$command" in
        development|staging|production|cloud-run)
            switch_environment "$command" "$app_filter" "$dry_run" "$verbose"
            ;;
        status)
            load_environment "$(detect_environment)" "" "$verbose"
            show_status "$verbose"
            ;;
        detect)
            detect_environment
            ;;
        validate)
            validate_environment
            ;;
        list)
            list_environments
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            if [[ -z "$command" ]]; then
                usage
            else
                echo -e "${RED}Unknown command: $command${NC}"
                usage
                exit 1
            fi
            ;;
    esac
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi