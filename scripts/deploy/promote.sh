#!/bin/bash
set -e

# Promote Cloud Run revision script
# Get script directory and load utilities
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/utils.sh"

SERVICE=$1
TAG=${2:-staging}
TRAFFIC=${3:-100}

if [ -z "$SERVICE" ]; then
    print_error "Usage: ./scripts/deploy/promote.sh <backend|frontend|admin> [tag] [traffic_percentage]"
    echo ""
    echo "Examples:"
    echo "  ./scripts/deploy/promote.sh backend staging 100        # Promote staging to 100%"
    echo "  ./scripts/deploy/promote.sh frontend development 50    # Canary deployment: 50%"
    echo "  ./scripts/deploy/promote.sh admin staging 10           # Gradual rollout: 10%"
    echo ""
    echo "Available operations:"
    echo "  - View current traffic: ./scripts/deploy/traffic.sh <service>"
    echo "  - List revisions: ./scripts/deploy/revisions.sh <service>"
    exit 1
fi

if [[ ! "$SERVICE" =~ ^(backend|frontend|admin)$ ]]; then
    print_error "Invalid service. Must be: backend, frontend, or admin"
    exit 1
fi

if [[ ! "$TRAFFIC" =~ ^[0-9]+$ ]] || [ "$TRAFFIC" -lt 0 ] || [ "$TRAFFIC" -gt 100 ]; then
    print_error "Traffic percentage must be a number between 0 and 100"
    exit 1
fi

# Set service name
case $SERVICE in
    "backend")
        SERVICE_NAME="epsx-backend"
        ;;
    "frontend")
        SERVICE_NAME="epsx-frontend"
        ;;
    "admin")
        SERVICE_NAME="epsx-admin"
        ;;
esac

# Load minimal environment for project configuration
ENV=${ENV:-production}
load_and_validate_env "$ENV"
get_env_config "$ENV"

print_info "Promoting $SERVICE revision '$TAG' to $TRAFFIC% traffic..."

# Check if the tagged revision exists
print_info "Checking if revision with tag '$TAG' exists..."
if ! gcloud run revisions list --service=$SERVICE_NAME --region=$REGION --format="value(metadata.name)" --filter="metadata.labels.serving-tag=$TAG" | grep -q .; then
    print_error "No revision found with tag '$TAG' for service $SERVICE_NAME"
    print_info "Available tagged revisions:"
    gcloud run revisions list --service=$SERVICE_NAME --region=$REGION --format="table(metadata.name,metadata.labels.serving-tag,status.conditions[0].status)" --filter="metadata.labels.serving-tag:*"
    exit 1
fi

# Safety check for production
if [ "$TRAFFIC" -gt 50 ]; then
    print_warning "You are about to promote $SERVICE ($TAG) to $TRAFFIC% traffic!"
    read -p "Are you sure? (yes/no): " -r
    if [[ ! $REPLY =~ ^yes$ ]]; then
        echo "Promotion cancelled"
        exit 1
    fi
fi

# Show current traffic distribution
print_info "Current traffic distribution:"
gcloud run services describe $SERVICE_NAME --region=$REGION --format="table(spec.traffic[].revisionName,spec.traffic[].tag,spec.traffic[].percent)"

# Perform traffic update
print_info "Updating traffic allocation..."

if [ "$TRAFFIC" = "100" ]; then
    # Full promotion - send all traffic to the tagged revision
    gcloud run services update-traffic $SERVICE_NAME \
        --region=$REGION \
        --to-revisions=$TAG=$TRAFFIC
else
    # Partial promotion - calculate remaining traffic for LATEST
    REMAINING_TRAFFIC=$((100 - TRAFFIC))
    gcloud run services update-traffic $SERVICE_NAME \
        --region=$REGION \
        --to-revisions=$TAG=$TRAFFIC,LATEST=$REMAINING_TRAFFIC
fi

print_status "Traffic promotion completed!"

# Show updated traffic distribution
print_info "Updated traffic distribution:"
gcloud run services describe $SERVICE_NAME --region=$REGION --format="table(spec.traffic[].revisionName,spec.traffic[].tag,spec.traffic[].percent)"

# Get service URL
SERVICE_URL=$(gcloud run services describe $SERVICE_NAME --region=$REGION --format="value(status.url)")
print_info "Service URL: $SERVICE_URL"

# Tagged revision URL
TAGGED_URL="https://$TAG---$SERVICE_NAME-$(echo $PROJECT_ID | tr -d '-').$REGION.run.app"
print_info "Tagged URL: $TAGGED_URL"

echo ""
echo "🎉 Promotion complete!"
echo "📊 Revision '$TAG' now receives $TRAFFIC% of traffic"

if [ "$TRAFFIC" -lt 100 ]; then
    echo ""
    echo "Next steps for gradual rollout:"
    echo "  - Monitor metrics and logs"
    echo "  - Increase traffic: ./scripts/deploy/promote.sh $SERVICE $TAG 50"
    echo "  - Full promotion: ./scripts/deploy/promote.sh $SERVICE $TAG 100"
    echo "  - Rollback: ./scripts/deploy/promote.sh $SERVICE LATEST 100"
fi