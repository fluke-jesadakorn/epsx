#!/bin/bash

# EPSX Environment Setup Script
# Sets up and validates environment for development/staging/production

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory and root detection
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Function: Print usage
usage() {
    echo -e "${BLUE}EPSX Environment Setup${NC}"
    echo
    echo "Usage: $0 <environment> [options]"
    echo
    echo "Environments:"
    echo -e "  ${GREEN}dev${NC}          Setup development environment"
    echo -e "  ${GREEN}test${NC}         Setup test environment"
    echo -e "  ${GREEN}staging${NC}      Setup staging environment" 
    echo -e "  ${GREEN}prod${NC}         Setup production environment"
    echo
    echo "Options:"
    echo -e "  ${YELLOW}--validate${NC}   Validate environment after setup"
    echo -e "  ${YELLOW}--verbose${NC}    Verbose output"
    echo -e "  ${YELLOW}--force${NC}      Force overwrite existing configuration"
}

# Function: Check prerequisites
check_prerequisites() {
    local verbose="${1:-false}"
    
    echo -e "${BLUE}Checking prerequisites...${NC}"
    
    # Check for required commands
    local required_commands=("node" "pnpm" "git")
    local missing_commands=()
    
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing_commands+=("$cmd")
        fi
    done
    
    if [[ ${#missing_commands[@]} -gt 0 ]]; then
        echo -e "${RED}❌ Missing required commands:${NC}"
        for cmd in "${missing_commands[@]}"; do
            echo -e "  ${RED}- $cmd${NC}"
        done
        return 1
    fi
    
    # Check Node.js version
    local node_version
    node_version=$(node --version | sed 's/v//')
    local required_node_version="18.0.0"
    
    if ! command -v npx >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  npx not found, some features may not work${NC}"
    fi
    
    if [[ "$verbose" == "true" ]]; then
        echo -e "  Node.js: ${GREEN}$node_version${NC}"
        echo -e "  pnpm: ${GREEN}$(pnpm --version)${NC}"
        echo -e "  Git: ${GREEN}$(git --version | cut -d' ' -f3)${NC}"
    fi
    
    echo -e "${GREEN}✅ Prerequisites check passed${NC}"
}

# Function: Setup development environment
setup_development() {
    local verbose="${1:-false}"
    
    echo -e "${BLUE}Setting up development environment...${NC}"
    
    # Use env-manager to switch to development
    "$SCRIPT_DIR/env-manager.sh" development ${verbose:+--verbose}
    
    # Setup development database if needed
    if [[ "$verbose" == "true" ]]; then
        echo -e "${BLUE}Development database setup:${NC}"
        echo -e "  Expected: ${YELLOW}postgresql://localhost/epsx_dev${NC}"
        echo -e "  Current:  ${YELLOW}${DATABASE_URL:-'unset'}${NC}"
    fi
    
    echo -e "${GREEN}✅ Development environment ready${NC}"
}

# Function: Setup test environment
setup_test() {
    local verbose="${1:-false}"
    
    echo -e "${BLUE}Setting up test environment...${NC}"
    
    # Use env-manager to switch to development (tests run in dev-like environment)
    "$SCRIPT_DIR/env-manager.sh" development ${verbose:+--verbose}
    
    # Override with test-specific settings
    export NODE_ENV=test
    export DATABASE_URL="${DATABASE_URL:-postgresql://localhost/epsx_test}"
    
    echo -e "${GREEN}✅ Test environment ready${NC}"
}

# Function: Setup staging environment
setup_staging() {
    local verbose="${1:-false}"
    
    echo -e "${BLUE}Setting up staging environment...${NC}"
    
    # Use env-manager to switch to staging
    "$SCRIPT_DIR/env-manager.sh" staging ${verbose:+--verbose}
    
    # Validate staging-specific requirements
    if [[ -z "${DATABASE_URL:-}" ]]; then
        echo -e "${RED}❌ DATABASE_URL not set for staging${NC}"
        return 1
    fi
    
    if [[ "$verbose" == "true" ]]; then
        echo -e "${BLUE}Staging configuration:${NC}"
        echo -e "  Database: ${YELLOW}${DATABASE_URL}${NC}"
        echo -e "  Frontend: ${YELLOW}${FRONTEND_URL:-'unset'}${NC}"
        echo -e "  Backend:  ${YELLOW}${BACKEND_URL:-'unset'}${NC}"
    fi
    
    echo -e "${GREEN}✅ Staging environment ready${NC}"
}

# Function: Setup production environment
setup_production() {
    local verbose="${1:-false}"
    
    echo -e "${BLUE}Setting up production environment...${NC}"
    
    # Use env-manager to switch to production
    "$SCRIPT_DIR/env-manager.sh" production ${verbose:+--verbose}
    
    # Validate production-specific requirements
    local required_prod_vars=("DATABASE_URL" "NEXTAUTH_SECRET" "FIREBASE_PRIVATE_KEY")
    local missing_vars=()
    
    for var in "${required_prod_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            missing_vars+=("$var")
        fi
    done
    
    if [[ ${#missing_vars[@]} -gt 0 ]]; then
        echo -e "${RED}❌ Missing required production variables:${NC}"
        for var in "${missing_vars[@]}"; do
            echo -e "  ${RED}- $var${NC}"
        done
        return 1
    fi
    
    if [[ "$verbose" == "true" ]]; then
        echo -e "${BLUE}Production configuration:${NC}"
        echo -e "  Database: ${YELLOW}${DATABASE_URL}${NC}"
        echo -e "  Frontend: ${YELLOW}${FRONTEND_URL:-'unset'}${NC}"
        echo -e "  Backend:  ${YELLOW}${BACKEND_URL:-'unset'}${NC}"
    fi
    
    echo -e "${GREEN}✅ Production environment ready${NC}"
}

# Function: Validate environment
validate_environment() {
    local verbose="${1:-false}"
    
    echo -e "${BLUE}Validating environment...${NC}"
    
    # Use env-manager validation
    if "$SCRIPT_DIR/env-manager.sh" validate; then
        echo -e "${GREEN}✅ Environment validation passed${NC}"
    else
        echo -e "${RED}❌ Environment validation failed${NC}"
        return 1
    fi
}

# Function: Setup dependencies
setup_dependencies() {
    local verbose="${1:-false}"
    
    echo -e "${BLUE}Installing dependencies...${NC}"
    
    # Install root dependencies
    if [[ "$verbose" == "true" ]]; then
        pnpm install
    else
        pnpm install --silent
    fi
    
    echo -e "${GREEN}✅ Dependencies installed${NC}"
}

# Main execution
main() {
    local environment="${1:-}"
    local validate=false
    local verbose=false
    local force=false
    
    # Parse arguments
    shift 2>/dev/null || true
    while [[ $# -gt 0 ]]; do
        case $1 in
            --validate)
                validate=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --force)
                force=true
                shift
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                usage
                exit 1
                ;;
        esac
    done
    
    # Validate environment parameter
    case "$environment" in
        dev|development)
            environment="development"
            ;;
        prod|production)
            environment="production"
            ;;
        test)
            environment="test"
            ;;
        staging)
            environment="staging"
            ;;
        help|--help|-h|"")
            usage
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown environment: $environment${NC}"
            usage
            exit 1
            ;;
    esac
    
    echo -e "${BLUE}🚀 EPSX Environment Setup${NC}"
    echo -e "Environment: ${GREEN}$environment${NC}"
    echo
    
    # Check prerequisites
    check_prerequisites "$verbose"
    echo
    
    # Setup dependencies
    setup_dependencies "$verbose"
    echo
    
    # Setup environment
    case "$environment" in
        development)
            setup_development "$verbose"
            ;;
        test)
            setup_test "$verbose"
            ;;
        staging)
            setup_staging "$verbose"
            ;;
        production)
            setup_production "$verbose"
            ;;
    esac
    echo
    
    # Validate if requested
    if [[ "$validate" == "true" ]]; then
        validate_environment "$verbose"
        echo
    fi
    
    echo -e "${GREEN}🎉 Environment setup complete!${NC}"
    echo
    echo "Next steps:"
    case "$environment" in
        development)
            echo -e "  ${YELLOW}pnpm dev${NC}        # Start development servers"
            echo -e "  ${YELLOW}pnpm build:dev${NC}  # Build for development"
            ;;
        test)
            echo -e "  ${YELLOW}pnpm test${NC}       # Run tests"
            echo -e "  ${YELLOW}pnpm test:watch${NC} # Run tests in watch mode"
            ;;
        staging)
            echo -e "  ${YELLOW}pnpm build:staging${NC}    # Build for staging"
            echo -e "  ${YELLOW}pnpm deploy:staging${NC}   # Deploy to staging"
            ;;
        production)
            echo -e "  ${YELLOW}pnpm build:prod${NC}       # Build for production"
            echo -e "  ${YELLOW}pnpm deploy:prod${NC}      # Deploy to production"
            ;;
    esac
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi