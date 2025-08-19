#!/bin/bash

# EPSX Environment Debugging Tool
# Comprehensive environment variable inspection and debugging

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory and root detection
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Function: Print usage
usage() {
    echo -e "${BLUE}EPSX Environment Debugger${NC}"
    echo
    echo "Usage: $0 [options]"
    echo
    echo "Options:"
    echo -e "  ${GREEN}--all${NC}        Show all environment variables"
    echo -e "  ${GREEN}--missing${NC}    Show missing required variables"
    echo -e "  ${GREEN}--compare${NC}    Compare current vs expected environment"
    echo -e "  ${GREEN}--secrets${NC}    Scan for potential secrets (be careful!)"
    echo -e "  ${GREEN}--apps${NC}       Show app-specific environment variables"
    echo -e "  ${GREEN}--files${NC}      Show which environment files exist"
    echo -e "  ${GREEN}--trace${NC}      Trace environment loading process"
    echo -e "  ${GREEN}--export${NC}     Export current environment to file"
    echo -e "  ${GREEN}--help${NC}       Show this help message"
}

# Function: Load environment for debugging
load_debug_environment() {
    local env_type="${1:-$(detect_current_env)}"
    
    # Source the env-manager to get environment loading functions
    source "$ROOT_DIR/scripts/env-manager.sh" 2>/dev/null || {
        echo -e "${RED}❌ Could not load env-manager.sh${NC}"
        return 1
    }
    
    load_environment "$env_type" "" true
}

# Function: Detect current environment
detect_current_env() {
    local detected="development"
    
    if [[ -n "${EPSX_CURRENT_ENV:-}" ]]; then
        detected="$EPSX_CURRENT_ENV"
    elif [[ -n "${ENV:-}" ]]; then
        detected="$ENV"
    elif [[ "${NODE_ENV:-}" == "production" ]]; then
        detected="production"
    fi
    
    echo "$detected"
}

# Function: Show environment summary
show_env_summary() {
    local current_env
    current_env=$(detect_current_env)
    
    echo -e "${BLUE}🔍 Environment Debug Summary${NC}"
    echo -e "  Current Environment: ${GREEN}$current_env${NC}"
    echo -e "  Timestamp: ${YELLOW}$(date)${NC}"
    echo -e "  Working Directory: ${YELLOW}$(pwd)${NC}"
    echo -e "  User: ${YELLOW}$(whoami)${NC}"
    echo

    # Load current environment
    load_debug_environment "$current_env" >/dev/null 2>&1

    # Core environment variables
    echo -e "${BLUE}Core Environment Variables:${NC}"
    local core_vars=("NODE_ENV" "ENV" "RUST_ENV" "NEXT_PUBLIC_BUILD_MODE")
    for var in "${core_vars[@]}"; do
        local value="${!var:-'❌ UNSET'}"
        if [[ "$value" == "❌ UNSET" ]]; then
            echo -e "  $var: ${RED}$value${NC}"
        else
            echo -e "  $var: ${GREEN}$value${NC}"
        fi
    done
    echo

    # URLs and endpoints
    echo -e "${BLUE}Service URLs:${NC}"
    local url_vars=("FRONTEND_URL" "ADMIN_FRONTEND_URL" "BACKEND_URL" "NEXT_PUBLIC_BACKEND_URL")
    for var in "${url_vars[@]}"; do
        local value="${!var:-'❌ UNSET'}"
        if [[ "$value" == "❌ UNSET" ]]; then
            echo -e "  $var: ${RED}$value${NC}"
        else
            echo -e "  $var: ${GREEN}$value${NC}"
        fi
    done
    echo

    # Database and external services
    echo -e "${BLUE}External Services:${NC}"
    local db_url="${DATABASE_URL:-'❌ UNSET'}"
    if [[ "$db_url" == "❌ UNSET" ]]; then
        echo -e "  DATABASE_URL: ${RED}$db_url${NC}"
    elif [[ "$db_url" == *"localhost"* ]]; then
        echo -e "  DATABASE_URL: ${YELLOW}Local Database${NC}"
    elif [[ "$db_url" == *"neon.tech"* ]]; then
        echo -e "  DATABASE_URL: ${GREEN}Neon PostgreSQL${NC}"
    else
        echo -e "  DATABASE_URL: ${BLUE}Remote Database${NC}"
    fi
    
    local redis_url="${REDIS_URL:-'❌ UNSET'}"
    if [[ "$redis_url" == "❌ UNSET" ]]; then
        echo -e "  REDIS_URL: ${RED}$redis_url${NC}"
    else
        echo -e "  REDIS_URL: ${GREEN}Configured${NC}"
    fi
    echo
}

# Function: Show all environment variables
show_all_env() {
    echo -e "${BLUE}📋 All Environment Variables${NC}"
    echo
    
    # Filter environment variables by EPSX-related prefixes
    local epsx_prefixes=("NEXT_PUBLIC_" "EPSX_" "DATABASE_" "REDIS_" "FIREBASE_" "MUSEPAY_" "OIDC_" "COOKIE_")
    
    for prefix in "${epsx_prefixes[@]}"; do
        local found_vars=()
        while IFS='=' read -r var value; do
            if [[ "$var" == "$prefix"* ]]; then
                found_vars+=("$var=$value")
            fi
        done < <(env | grep "^$prefix" | sort)
        
        if [[ ${#found_vars[@]} -gt 0 ]]; then
            echo -e "${PURPLE}$prefix* Variables:${NC}"
            for var_value in "${found_vars[@]}"; do
                local var="${var_value%%=*}"
                local value="${var_value#*=}"
                
                # Mask sensitive values
                if [[ "$var" == *"SECRET"* ]] || [[ "$var" == *"KEY"* ]] || [[ "$var" == *"PASSWORD"* ]]; then
                    echo -e "  ${var}: ${YELLOW}****MASKED****${NC}"
                elif [[ ${#value} -gt 100 ]]; then
                    echo -e "  ${var}: ${CYAN}${value:0:50}...${NC} (truncated)"
                else
                    echo -e "  ${var}: ${GREEN}${value}${NC}"
                fi
            done
            echo
        fi
    done
}

# Function: Check missing required variables
check_missing_vars() {
    echo -e "${BLUE}🚨 Missing Required Variables${NC}"
    echo
    
    local current_env
    current_env=$(detect_current_env)
    
    # Load environment
    load_debug_environment "$current_env" >/dev/null 2>&1
    
    # Define required variables per environment
    local required_vars=()
    case "$current_env" in
        development)
            required_vars=("NODE_ENV" "DATABASE_URL" "FRONTEND_URL" "BACKEND_URL" "NEXTAUTH_SECRET")
            ;;
        staging|production)
            required_vars=("NODE_ENV" "DATABASE_URL" "FRONTEND_URL" "BACKEND_URL" "ADMIN_FRONTEND_URL" "NEXTAUTH_SECRET" "FIREBASE_PRIVATE_KEY" "FIREBASE_CLIENT_EMAIL")
            ;;
        *)
            required_vars=("NODE_ENV" "DATABASE_URL" "FRONTEND_URL" "BACKEND_URL")
            ;;
    esac
    
    local missing_vars=()
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            missing_vars+=("$var")
        fi
    done
    
    if [[ ${#missing_vars[@]} -eq 0 ]]; then
        echo -e "${GREEN}✅ All required variables are set${NC}"
    else
        echo -e "${RED}❌ Missing ${#missing_vars[@]} required variables:${NC}"
        for var in "${missing_vars[@]}"; do
            echo -e "  ${RED}- $var${NC}"
        done
    fi
    echo
}

# Function: Show environment files status
show_env_files() {
    echo -e "${BLUE}📁 Environment Files Status${NC}"
    echo
    
    # Root environment files
    echo -e "${PURPLE}Root Environment Files:${NC}"
    local root_files=(".env.defaults" ".env.development" ".env.staging" ".env.production" ".env.cloud-run" ".env.local")
    for file in "${root_files[@]}"; do
        if [[ -f "$ROOT_DIR/$file" ]]; then
            local size=$(ls -lh "$ROOT_DIR/$file" | awk '{print $5}')
            echo -e "  ✅ $file ${CYAN}($size)${NC}"
        else
            echo -e "  ❌ $file ${RED}(missing)${NC}"
        fi
    done
    echo
    
    # Shared environment files  
    echo -e "${PURPLE}Shared Environment Files:${NC}"
    local shared_files=(".env.shared" ".env.shared.development" ".env.shared.staging" ".env.shared.production")
    for file in "${shared_files[@]}"; do
        if [[ -f "$ROOT_DIR/$file" ]]; then
            local size=$(ls -lh "$ROOT_DIR/$file" | awk '{print $5}')
            echo -e "  ✅ $file ${CYAN}($size)${NC}"
        else
            echo -e "  ❌ $file ${RED}(missing)${NC}"
        fi
    done
    echo
    
    # App-specific environment files
    local apps=("frontend" "admin-frontend" "backend")
    for app in "${apps[@]}"; do
        echo -e "${PURPLE}$app Environment Files:${NC}"
        local app_files=(".env.development" ".env.staging" ".env.production" ".env.local")
        for file in "${app_files[@]}"; do
            if [[ -f "$ROOT_DIR/apps/$app/$file" ]]; then
                local size=$(ls -lh "$ROOT_DIR/apps/$app/$file" | awk '{print $5}')
                echo -e "  ✅ $file ${CYAN}($size)${NC}"
            else
                echo -e "  ❌ $file ${RED}(missing)${NC}"
            fi
        done
        echo
    done
}

# Function: Export current environment
export_environment() {
    local output_file="$ROOT_DIR/.devtools/environment-export-$(date +%Y%m%d-%H%M%S).env"
    
    echo -e "${BLUE}📤 Exporting current environment...${NC}"
    
    # Create export file
    {
        echo "# EPSX Environment Export"
        echo "# Generated: $(date)"
        echo "# Current Environment: $(detect_current_env)"
        echo
        
        # Export EPSX-related variables only
        env | grep -E "^(NEXT_PUBLIC_|EPSX_|NODE_ENV|ENV|RUST_ENV|DATABASE_|REDIS_|FRONTEND_|BACKEND_|ADMIN_)" | sort
        
    } > "$output_file"
    
    echo -e "  ✅ Environment exported to: ${GREEN}$output_file${NC}"
    echo -e "  📝 Size: ${YELLOW}$(ls -lh "$output_file" | awk '{print $5}')${NC}"
    echo
}

# Main execution
main() {
    local show_all=false
    local show_missing=false
    local show_files=false
    local export_env=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --all)
                show_all=true
                shift
                ;;
            --missing)
                show_missing=true
                shift
                ;;
            --files)
                show_files=true
                shift
                ;;
            --export)
                export_env=true
                shift
                ;;
            --help|-h)
                usage
                exit 0
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                usage
                exit 1
                ;;
        esac
    done
    
    # Always show summary
    show_env_summary
    
    # Show requested information
    if [[ "$show_missing" == "true" ]]; then
        check_missing_vars
    fi
    
    if [[ "$show_files" == "true" ]]; then
        show_env_files
    fi
    
    if [[ "$show_all" == "true" ]]; then
        show_all_env
    fi
    
    if [[ "$export_env" == "true" ]]; then
        export_environment
    fi
    
    # Show help hint if no specific options
    if [[ "$show_all" == "false" && "$show_missing" == "false" && "$show_files" == "false" && "$export_env" == "false" ]]; then
        echo -e "${YELLOW}💡 Use --help to see available debugging options${NC}"
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi