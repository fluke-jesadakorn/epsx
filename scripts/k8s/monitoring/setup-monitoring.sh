#!/bin/bash

# EPSX Monitoring Setup Script
# Sets up Prometheus, Grafana, and monitoring dashboards
# Usage: ./scripts/k8s/monitoring/setup-monitoring.sh

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1"
}

success() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] SUCCESS:${NC} $1"
}

log "Setting up EPSX monitoring stack"

# Check prerequisites
command -v kubectl >/dev/null 2>&1 || { error "kubectl is required but not installed"; exit 1; }
command -v helm >/dev/null 2>&1 || { error "helm is required but not installed"; exit 1; }

# Check if monitoring namespace exists
kubectl create namespace monitoring --dry-run=client -o yaml | kubectl apply -f -

# Check if kube-prometheus-stack is already installed
if helm list -n monitoring | grep -q "kube-prometheus-stack"; then
    warn "kube-prometheus-stack is already installed. Upgrading..."
    UPGRADE_FLAG="--upgrade"
else
    log "Installing kube-prometheus-stack..."
    UPGRADE_FLAG=""
fi

# Install/Upgrade kube-prometheus-stack
helm $UPGRADE_FLAG install kube-prometheus-stack prometheus-community/kube-prometheus-stack \
    --namespace monitoring \
    --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.resources.requests.storage=50Gi \
    --set prometheus.prometheusSpec.resources.requests.cpu=500m \
    --set prometheus.prometheusSpec.resources.requests.memory=1Gi \
    --set prometheus.prometheusSpec.resources.limits.cpu=1000m \
    --set prometheus.prometheusSpec.resources.limits.memory=2Gi \
    --set prometheus.prometheusSpec.serviceMonitor.enabled=true \
    --set prometheus.prometheusSpec.ruleSelector.matchLabels.app.kubernetes.io.name=epsx \
    --set grafana.persistence.enabled=true \
    --set grafana.persistence.size=10Gi \
    --set grafana.resources.requests.cpu=200m \
    --set grafana.resources.requests.memory=256Mi \
    --set grafana.adminPassword=admin123 \
    --set grafana.sidecar.datasources.enabled=true \
    --set grafana.sidecar.dashboards.enabled=true \
    --set grafana.sidecar.dashboards.searchNamespace=monitoring \
    --wait

success "kube-prometheus-stack deployed successfully"

# Apply custom Prometheus rules
log "Applying custom Prometheus rules..."
kubectl apply -f "$SCRIPT_DIR/prometheus-rules.yaml"

# Create Grafana dashboard configmap
log "Creating Grafana dashboard..."
kubectl create configmap epsx-dashboard \
    --namespace=monitoring \
    --from-file="$SCRIPT_DIR/grafana-dashboard.json" \
    --dry-run=client -o yaml | kubectl apply -f -

# Label the configmap for Grafana sidecar
kubectl label configmap epsx-dashboard \
    --namespace=monitoring \
    grafana_dashboard=1 \
    --overwrite

# Create ServiceMonitors for EPSX services
log "Creating ServiceMonitors..."

# Backend ServiceMonitor
cat <<EOF | kubectl apply -f -
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: backend-monitor
  namespace: epsx
  labels:
    app.kubernetes.io/name: epsx
    app.kubernetes.io/component: backend
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: epsx
      app.kubernetes.io/component: backend
  endpoints:
  - port: http
    path: /metrics
    interval: 30s
    scrapeTimeout: 10s
EOF

# Frontend ServiceMonitor
cat <<EOF | kubectl apply -f -
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: frontend-monitor
  namespace: epsx
  labels:
    app.kubernetes.io/name: epsx
    app.kubernetes.io/component: frontend
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: epsx
      app.kubernetes.io/component: frontend
  endpoints:
  - port: http
    interval: 30s
    scrapeTimeout: 10s
EOF

# PostgreSQL ServiceMonitor (if using postgres_exporter)
cat <<EOF | kubectl apply -f -
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: postgres-monitor
  namespace: epsx
  labels:
    app.kubernetes.io/name: epsx
    app.kubernetes.io/component: postgres
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: epsx
      app.kubernetes.io/component: postgres
  endpoints:
  - port: metrics
    interval: 30s
    scrapeTimeout: 10s
EOF

# Redis ServiceMonitor (if using redis_exporter)
cat <<EOF | kubectl apply -f -
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: redis-monitor
  namespace: epsx
  labels:
    app.kubernetes.io/name: epsx
    app.kubernetes.io/component: redis
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: epsx
      app.kubernetes.io/component: redis
  endpoints:
  - port: metrics
    interval: 30s
    scrapeTimeout: 10s
EOF

# Create blackbox exporter for URL monitoring
log "Setting up blackbox exporter..."

helm upgrade --install blackbox-exporter prometheus-community/prometheus-blackbox-exporter \
    --namespace monitoring \
    --set config.modules.http_2xx.prober=http \
    --set config.modules.http_2xx.timeout=5s \
    --set serviceMonitor.enabled=true \
    --set serviceMonitor.targets='{"epsx.io":"https://epsx.io","api.epsx.io":"https://api.epsx.io","admin.epsx.io":"https://admin.epsx.io"}' \
    --wait

# Set up port forwarding helper script
cat <<EOF > "$PROJECT_ROOT/scripts/k8s/monitoring/access-monitoring.sh"
#!/bin/bash

# Access Monitoring Stack
# Usage: ./access-monitoring.sh [service]

SERVICE=\${1:-grafana}

case \$SERVICE in
    grafana)
        echo "Forwarding Grafana (http://localhost:3000, admin/admin123)..."
        kubectl port-forward svc/kube-prometheus-stack-grafana 3000:80 -n monitoring
        ;;
    prometheus)
        echo "Forwarding Prometheus (http://localhost:9090)..."
        kubectl port-forward svc/kube-prometheus-stack-prometheus 9090:9090 -n monitoring
        ;;
    alertmanager)
        echo "Forwarding AlertManager (http://localhost:9093)..."
        kubectl port-forward svc/kube-prometheus-stack-alertmanager 9093:9093 -n monitoring
        ;;
    *)
        echo "Usage: \$0 [grafana|prometheus|alertmanager]"
        exit 1
        ;;
esac
EOF

chmod +x "$PROJECT_ROOT/scripts/k8s/monitoring/access-monitoring.sh"

success "Monitoring setup completed successfully!"

echo ""
success "EPSX monitoring is now ready"
echo ""
log "Access the monitoring stack:"
echo "  Grafana:    ./scripts/k8s/monitoring/access-monitoring.sh grafana"
echo "  Prometheus: ./scripts/k8s/monitoring/access-monitoring.sh prometheus"
echo "  AlertManager: ./scripts/k8s/monitoring/access-monitoring.sh alertmanager"
echo ""
log "Grafana credentials:"
echo "  Username: admin"
echo "  Password: admin123"
echo ""
log "Dashboard URL after port-forward: http://localhost:3000"
log "Prometheus URL after port-forward: http://localhost:9090"

# Show monitoring status
echo ""
log "Monitoring components status:"
kubectl get pods --namespace=monitoring -l release=kube-prometheus-stack
kubectl get servicemonitors --all-namespaces | grep epsx