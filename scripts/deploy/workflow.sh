#!/bin/bash
set -e

# EPSX Revision-Based Deployment Workflow
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/utils.sh"

ACTION=${1:-help}
SERVICE=$2
TAG=${3:-staging}

show_help() {
    echo "🚀 EPSX Revision-Based Deployment Workflow"
    echo ""
    echo "Usage: ./scripts/deploy/workflow.sh <action> [service] [tag]"
    echo ""
    echo "Actions:"
    echo "  deploy <service> <tag>    Deploy service with tag (no traffic)"
    echo "  promote <service> <tag>   Promote tagged revision to 100% traffic"
    echo "  canary <service> <tag>    Start canary deployment (10% traffic)"
    echo "  rollback <service>        Rollback to previous LATEST revision"
    echo "  status <service>          Show service status and traffic"
    echo "  revisions <service>       List all revisions"
    echo "  traffic <service>         Show traffic distribution"
    echo "  help                      Show this help"
    echo ""
    echo "Services: backend, frontend, admin"
    echo "Tags: development, staging, hotfix, feature-xyz, etc."
    echo ""
    echo "Example Workflows:"
    echo ""
    echo "  1. Deploy and test staging:"
    echo "     ./scripts/deploy/workflow.sh deploy backend staging"
    echo "     ./scripts/deploy/workflow.sh status backend"
    echo "     # Test at tagged URL"
    echo "     ./scripts/deploy/workflow.sh promote backend staging"
    echo ""
    echo "  2. Canary deployment:"
    echo "     ./scripts/deploy/workflow.sh deploy frontend hotfix"
    echo "     ./scripts/deploy/workflow.sh canary frontend hotfix"
    echo "     # Monitor metrics"
    echo "     ./scripts/deploy/workflow.sh promote frontend hotfix"
    echo ""
    echo "  3. Emergency rollback:"
    echo "     ./scripts/deploy/workflow.sh rollback backend"
    echo ""
}

validate_service() {
    if [[ ! "$SERVICE" =~ ^(backend|frontend|admin)$ ]]; then
        print_error "Invalid service. Must be: backend, frontend, or admin"
        exit 1
    fi
}

case $ACTION in
    "deploy")
        if [ -z "$SERVICE" ] || [ -z "$TAG" ]; then
            print_error "Usage: ./scripts/deploy/workflow.sh deploy <service> <tag>"
            exit 1
        fi
        validate_service
        
        print_info "🚀 Deploying $SERVICE with tag '$TAG' (no traffic)"
        ENV=$TAG "$SCRIPT_DIR/service.sh" $SERVICE
        ;;
        
    "promote")
        if [ -z "$SERVICE" ] || [ -z "$TAG" ]; then
            print_error "Usage: ./scripts/deploy/workflow.sh promote <service> <tag>"
            exit 1
        fi
        validate_service
        
        print_info "📈 Promoting $SERVICE revision '$TAG' to 100% traffic"
        "$SCRIPT_DIR/promote.sh" $SERVICE $TAG 100
        ;;
        
    "canary")
        if [ -z "$SERVICE" ] || [ -z "$TAG" ]; then
            print_error "Usage: ./scripts/deploy/workflow.sh canary <service> <tag>"
            exit 1
        fi
        validate_service
        
        print_info "🐤 Starting canary deployment: $SERVICE revision '$TAG' to 10% traffic"
        "$SCRIPT_DIR/promote.sh" $SERVICE $TAG 10
        ;;
        
    "rollback")
        if [ -z "$SERVICE" ]; then
            print_error "Usage: ./scripts/deploy/workflow.sh rollback <service>"
            exit 1
        fi
        validate_service
        
        print_warning "🔙 Rolling back $SERVICE to LATEST revision"
        "$SCRIPT_DIR/promote.sh" $SERVICE LATEST 100
        ;;
        
    "status")
        if [ -z "$SERVICE" ]; then
            print_error "Usage: ./scripts/deploy/workflow.sh status <service>"
            exit 1
        fi
        validate_service
        
        print_info "📊 Status for $SERVICE"
        "$SCRIPT_DIR/traffic.sh" $SERVICE
        ;;
        
    "revisions")
        if [ -z "$SERVICE" ]; then
            print_error "Usage: ./scripts/deploy/workflow.sh revisions <service>"
            exit 1
        fi
        validate_service
        
        "$SCRIPT_DIR/revisions.sh" $SERVICE
        ;;
        
    "traffic")
        if [ -z "$SERVICE" ]; then
            print_error "Usage: ./scripts/deploy/workflow.sh traffic <service>"
            exit 1
        fi
        validate_service
        
        "$SCRIPT_DIR/traffic.sh" $SERVICE
        ;;
        
    "help"|"")
        show_help
        ;;
        
    *)
        print_error "Unknown action: $ACTION"
        echo ""
        show_help
        exit 1
        ;;
esac