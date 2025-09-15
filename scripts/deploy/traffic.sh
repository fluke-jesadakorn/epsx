#!/bin/bash
set -e

# View Cloud Run traffic distribution script
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/utils.sh"

SERVICE=$1

if [ -z "$SERVICE" ]; then
    print_error "Usage: ./scripts/deploy/traffic.sh <backend|frontend|admin>"
    echo ""
    echo "Examples:"
    echo "  ./scripts/deploy/traffic.sh backend     # Show backend traffic distribution"
    echo "  ./scripts/deploy/traffic.sh frontend    # Show frontend traffic distribution"
    echo "  ./scripts/deploy/traffic.sh admin       # Show admin traffic distribution"
    exit 1
fi

if [[ ! "$SERVICE" =~ ^(backend|frontend|admin)$ ]]; then
    print_error "Invalid service. Must be: backend, frontend, or admin"
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

print_info "Current traffic distribution for $SERVICE_NAME:"
echo ""

# Show traffic allocation with nice formatting
gcloud run services describe $SERVICE_NAME --region=$REGION \
    --format="table(
        spec.traffic[].revisionName:label='REVISION',
        spec.traffic[].tag:label='TAG',
        spec.traffic[].percent:label='TRAFFIC_%'
    )"

echo ""

# Show service URL
SERVICE_URL=$(gcloud run services describe $SERVICE_NAME --region=$REGION --format="value(status.url)")
print_info "Main Service URL: $SERVICE_URL"

# Show tagged URLs if any exist
print_info "Tagged revision URLs:"
gcloud run revisions list --service=$SERVICE_NAME --region=$REGION \
    --format="table(metadata.name,metadata.labels.serving-tag)" \
    --filter="metadata.labels.serving-tag:*" | while read -r revision tag; do
    if [ "$tag" != "serving-tag" ] && [ -n "$tag" ]; then
        TAGGED_URL="https://$tag---$SERVICE_NAME-$(echo $PROJECT_ID | tr -d '-').$REGION.run.app"
        echo "  $tag: $TAGGED_URL"
    fi
done

echo ""
print_info "Quick actions:"
echo "  - Promote revision: ./scripts/deploy/promote.sh $SERVICE <tag> <percentage>"
echo "  - List all revisions: ./scripts/deploy/revisions.sh $SERVICE"
echo "  - Deploy new revision: ./scripts/deploy/service.sh $SERVICE"