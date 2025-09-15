#!/bin/bash
set -e

# EPSX - Deployment Status Script
# Check status of Cloud Run services and recent deployments

PROJECT_ID="epsx-469400"
REGION="us-central1"

echo "📊 EPSX Deployment Status"
echo "=========================="
echo ""

# Function to check service status
check_service_status() {
    local service_name=$1
    echo "🔍 $service_name Service:"
    
    # Check if service exists
    if gcloud run services describe "$service_name" --region="$REGION" --format="value(metadata.name)" >/dev/null 2>&1; then
        # Get service details
        local url=$(gcloud run services describe "$service_name" --region="$REGION" --format="value(status.url)")
        local ready=$(gcloud run services describe "$service_name" --region="$REGION" --format="value(status.conditions[0].status)")
        local traffic=$(gcloud run services describe "$service_name" --region="$REGION" --format="value(status.traffic[0].percent)")
        local revision=$(gcloud run services describe "$service_name" --region="$REGION" --format="value(status.traffic[0].revisionName)")
        local updated=$(gcloud run services describe "$service_name" --region="$REGION" --format="value(status.conditions[0].lastTransitionTime)")
        
        echo "   Status: $([ "$ready" = "True" ] && echo "🟢 Ready" || echo "🔴 Not Ready")"
        echo "   URL: $url"
        echo "   Traffic: ${traffic}% → $revision"
        echo "   Last Updated: $updated"
        
        # Check recent revisions
        echo "   Recent Revisions:"
        gcloud run revisions list \
            --service="$service_name" \
            --region="$REGION" \
            --limit=3 \
            --format="table(metadata.name:label='Revision',status.conditions[0].status:label='Ready',metadata.creationTimestamp:label='Created')" \
            --sort-by="~metadata.creationTimestamp" | sed 's/^/     /'
    else
        echo "   Status: 🔴 Service not found"
        echo "   Note: Service may not be deployed yet"
    fi
    echo ""
}

# Check specific service if provided
if [ $# -eq 1 ]; then
    SERVICE_NAME=$1
    echo "Checking status for: $SERVICE_NAME"
    echo ""
    check_service_status "$SERVICE_NAME"
else
    # Check all services
    echo "Checking all EPSX services..."
    echo ""
    
    check_service_status "frontend"
    check_service_status "admin"
    check_service_status "backend"
fi

# Show recent Cloud Build activity
echo "🔨 Recent Cloud Build Activity:"
echo "================================"
gcloud builds list \
    --filter="source.repoSource.repoName='epsx' OR buildTriggerId~'auto-revision'" \
    --limit=5 \
    --format="table(id:label='Build ID',status:label='Status',createTime:label='Started',duration:label='Duration',substitutions.TRIGGER_NAME:label='Trigger')" \
    --sort-by="~createTime" || echo "No recent builds found"

echo ""
echo "💡 Quick Commands:"
echo "   View logs: ./scripts/deploy/logs.sh [service]"
echo "   Push update: ./scripts/deploy/push-[service].sh"
echo "   Push all: ./scripts/deploy/push-all.sh"