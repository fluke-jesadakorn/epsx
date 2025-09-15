#!/bin/bash
set -e

# EPSX - Deployment Logs Script
# View Cloud Run service logs and Cloud Build deployment logs

PROJECT_ID="epsx-469400"
REGION="us-central1"

echo "📋 EPSX Deployment Logs"
echo "======================="
echo ""

# Function to show service logs
show_service_logs() {
    local service_name=$1
    local lines=${2:-50}
    
    echo "📊 $service_name Service Logs (last $lines lines):"
    echo "=================================================="
    
    # Check if service exists
    if gcloud run services describe "$service_name" --region="$REGION" --format="value(metadata.name)" >/dev/null 2>&1; then
        gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=$service_name" \
            --limit="$lines" \
            --order="desc" \
            --format="table(timestamp,severity,textPayload)" \
            --project="$PROJECT_ID" || echo "No logs found for $service_name"
    else
        echo "❌ Service '$service_name' not found"
    fi
    echo ""
}

# Function to show Cloud Build logs
show_build_logs() {
    local limit=${1:-10}
    
    echo "🔨 Recent Cloud Build Logs (last $limit builds):"
    echo "=============================================="
    
    # Show recent builds related to auto-revision
    gcloud builds list \
        --filter="buildTriggerId~'auto-revision'" \
        --limit="$limit" \
        --format="table(id:label='Build ID',status:label='Status',createTime:label='Started',duration:label='Duration',substitutions._IMAGE_NAME:label='Image')" \
        --sort-by="~createTime" || echo "No build logs found"
    echo ""
}

# Function to show detailed build log
show_detailed_build_log() {
    local build_id=$1
    
    echo "🔍 Detailed Build Log for: $build_id"
    echo "===================================="
    gcloud logging read "resource.type=build AND resource.labels.build_id=$build_id" \
        --order="asc" \
        --format="value(textPayload)" \
        --project="$PROJECT_ID"
    echo ""
}

# Parse command line arguments
case "${1:-}" in
    "frontend"|"admin"|"backend")
        SERVICE_NAME=$1
        LINES=${2:-50}
        echo "Showing logs for: $SERVICE_NAME"
        echo ""
        show_service_logs "$SERVICE_NAME" "$LINES"
        ;;
    "builds")
        LIMIT=${2:-10}
        show_build_logs "$LIMIT"
        ;;
    "build")
        if [ -z "${2:-}" ]; then
            echo "❌ Error: Build ID required"
            echo "Usage: $0 build <build-id>"
            exit 1
        fi
        show_detailed_build_log "$2"
        ;;
    "")
        # Show overview of all services
        echo "Showing overview logs for all services..."
        echo ""
        
        show_service_logs "frontend" 20
        show_service_logs "admin" 20
        show_service_logs "backend" 20
        show_build_logs 5
        ;;
    *)
        echo "❌ Unknown option: $1"
        echo ""
        echo "Usage: $0 [service|builds|build] [options]"
        echo ""
        echo "Examples:"
        echo "   $0                    # Show overview of all services"
        echo "   $0 frontend          # Show frontend logs (50 lines)"
        echo "   $0 frontend 100      # Show frontend logs (100 lines)"
        echo "   $0 admin             # Show admin logs"
        echo "   $0 backend           # Show backend logs"
        echo "   $0 builds            # Show recent build logs"
        echo "   $0 builds 20         # Show last 20 builds"
        echo "   $0 build <build-id>  # Show detailed build log"
        exit 1
        ;;
esac

echo "💡 Quick Commands:"
echo "   View status: ./scripts/deploy/status.sh [service]"
echo "   Follow logs: gcloud logging tail \"resource.type=cloud_run_revision AND resource.labels.service_name=[service]\""
echo "   Push update: ./scripts/deploy/push-[service].sh"