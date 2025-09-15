#!/bin/bash
set -e

# View Cloud Run revisions script
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/utils.sh"

SERVICE=$1

if [ -z "$SERVICE" ]; then
    print_error "Usage: ./scripts/deploy/revisions.sh <backend|frontend|admin>"
    echo ""
    echo "Examples:"
    echo "  ./scripts/deploy/revisions.sh backend     # List backend revisions"
    echo "  ./scripts/deploy/revisions.sh frontend    # List frontend revisions"
    echo "  ./scripts/deploy/revisions.sh admin       # List admin revisions"
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

print_info "Revisions for $SERVICE_NAME:"
echo ""

# Show all revisions with detailed information
gcloud run revisions list --service=$SERVICE_NAME --region=$REGION \
    --format="table(
        metadata.name:label='REVISION',
        metadata.labels.serving-tag:label='TAG',
        status.conditions[0].status:label='STATUS',
        metadata.creationTimestamp.date(format='%Y-%m-%d %H:%M'):label='CREATED',
        spec.containers[0].image.split('/').slice(-1:):label='IMAGE'
    )"

echo ""

# Show current traffic allocation
print_info "Current traffic allocation:"
gcloud run services describe $SERVICE_NAME --region=$REGION \
    --format="table(
        spec.traffic[].revisionName:label='REVISION',
        spec.traffic[].tag:label='TAG',
        spec.traffic[].percent:label='TRAFFIC_%'
    )"

echo ""
print_info "Quick actions:"
echo "  - Promote revision: ./scripts/deploy/promote.sh $SERVICE <tag> <percentage>"
echo "  - View traffic only: ./scripts/deploy/traffic.sh $SERVICE"
echo "  - Deploy new revision: ./scripts/deploy/service.sh $SERVICE"