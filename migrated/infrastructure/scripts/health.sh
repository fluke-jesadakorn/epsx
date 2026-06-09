#!/bin/bash
# ============================================================================
# EPSX Microservices - Health Check
# ============================================================================

services=(
    "gateway:8080"
    "identity:8101"
    "wallet:8102"
    "payment:8103"
    "subscription:8104"
    "content:8105"
    "notification:8106"
    "analytics:8107"
    "indexer:8108"
    "bff-frontend:3000"
    "bff-admin:3001"
    "bff-pay:3002"
    "bff-preview:3003"
)

echo "EPSX Service Health Check"
echo "========================"
echo ""

healthy=0
unhealthy=0

for svc in "${services[@]}"; do
    name="${svc%%:*}"
    port="${svc##*:}"
    
    if curl -sf "http://localhost:$port/health" > /dev/null 2>&1; then
        echo "  ✓ $name (port $port) - healthy"
        ((healthy++))
    else
        echo "  ✗ $name (port $port) - unhealthy"
        ((unhealthy++))
    fi
done

echo ""
echo "Result: $healthy healthy, $unhealthy unhealthy"
echo ""

if [ $unhealthy -gt 0 ]; then
    exit 1
fi
