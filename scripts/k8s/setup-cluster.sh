#!/bin/bash

# EPSX Kubernetes Cluster Setup Script
# Sets up on-prem Kubernetes cluster with required components
# Usage: ./scripts/k8s/setup-cluster.sh [options]

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Default values
CLUSTER_NAME="epsx-cluster"
NODE_COUNT=3
INSTALL_INGRESS=true
INSTALL_CERTMANAGER=true
INSTALL_METALLB=true
INSTALL_MONITORING=true
INSTALL_STORAGE=true
METALLB_IP_RANGE="192.168.1.200-192.168.1.250"
STORAGE_CLASS="fast-ssd"

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

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --cluster-name)
            CLUSTER_NAME="$2"
            shift 2
            ;;
        --node-count)
            NODE_COUNT="$2"
            shift 2
            ;;
        --no-ingress)
            INSTALL_INGRESS=false
            shift
            ;;
        --no-certmanager)
            INSTALL_CERTMANAGER=false
            shift
            ;;
        --no-metallb)
            INSTALL_METALLB=false
            shift
            ;;
        --no-monitoring)
            INSTALL_MONITORING=false
            shift
            ;;
        --no-storage)
            INSTALL_STORAGE=false
            shift
            ;;
        --metallb-ip-range)
            METALLB_IP_RANGE="$2"
            shift 2
            ;;
        --storage-class)
            STORAGE_CLASS="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --cluster-name NAME        Cluster name (default: epsx-cluster)"
            echo "  --node-count COUNT         Number of nodes (default: 3)"
            echo "  --no-ingress              Skip NGINX Ingress Controller"
            echo "  --no-certmanager          Skip cert-manager"
            echo "  --no-metallb              Skip MetalLB"
            echo "  --no-monitoring           Skip monitoring stack"
            echo "  --no-storage              Skip storage classes"
            echo "  --metallb-ip-range RANGE  MetalLB IP range (default: 192.168.1.200-192.168.1.250)"
            echo "  --storage-class NAME      Storage class name (default: fast-ssd)"
            echo "  --help                    Show this help message"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            exit 1
            ;;
    esac
done

log "Setting up EPSX Kubernetes cluster"
log "Cluster name: $CLUSTER_NAME"
log "Node count: $NODE_COUNT"

# Check prerequisites
command -v kubectl >/dev/null 2>&1 || { error "kubectl is required but not installed"; exit 1; }
command -v helm >/dev/null 2>&1 || { error "helm is required but not installed"; exit 1; }

# Check if cluster exists
if ! kubectl cluster-info >/dev/null 2>&1; then
    error "Cannot connect to Kubernetes cluster"
    error "Please ensure your cluster is running and kubectl is configured"
    exit 1
fi

# Add required Helm repositories
log "Adding Helm repositories..."

helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
helm repo add jetstack https://charts.jetstack.io
helm repo add metallb https://metallb.github.io/metallb
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

success "Helm repositories added successfully"

# Install NGINX Ingress Controller
if [[ "$INSTALL_INGRESS" == "true" ]]; then
    log "Installing NGINX Ingress Controller..."

    helm upgrade --install ingress-nginx ingress-nginx/ingress-nginx \
        --namespace ingress-nginx \
        --create-namespace \
        --set controller.replicaCount=2 \
        --set controller.resources.requests.cpu=200m \
        --set controller.resources.requests.memory=256Mi \
        --set controller.resources.limits.cpu=500m \
        --set controller.resources.limits.memory=512Mi \
        --set controller.service.type=LoadBalancer \
        --set controller.admissionWebhooks.enabled=true \
        --set controller.metrics.enabled=true \
        --set controller.metrics.serviceMonitor.enabled=true \
        --wait

    success "NGINX Ingress Controller installed successfully"
fi

# Install cert-manager
if [[ "$INSTALL_CERTMANAGER" == "true" ]]; then
    log "Installing cert-manager..."

    # Install cert-manager CRDs
    kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.2/cert-manager.crds.yaml

    helm upgrade --install cert-manager jetstack/cert-manager \
        --namespace cert-manager \
        --create-namespace \
        --set installCRDs=true \
        --set resources.requests.cpu=100m \
        --set resources.requests.memory=128Mi \
        --set resources.limits.cpu=500m \
        --set resources.limits.memory=512Mi \
        --wait

    # Create Let's Encrypt ClusterIssuer
    log "Creating Let's Encrypt ClusterIssuer..."
    cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@epsx.io
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF

    success "cert-manager installed successfully"
fi

# Install MetalLB
if [[ "$INSTALL_METALLB" == "true" ]]; then
    log "Installing MetalLB..."

    helm upgrade --install metallb metallb/metallb \
        --namespace metallb-system \
        --create-namespace \
        --set controller.resources.requests.cpu=100m \
        --set controller.resources.requests.memory=100Mi \
        --set speaker.resources.requests.cpu=100m \
        --set speaker.resources.requests.memory=100Mi \
        --wait

    # Create MetalLB address pool
    cat <<EOF | kubectl apply -f -
apiVersion: metallb.io/v1beta1
kind: IPAddressPool
metadata:
  name: epsx-ip-pool
  namespace: metallb-system
spec:
  addresses:
  - $METALLB_IP_RANGE
---
apiVersion: metallb.io/v1beta1
kind: L2Advertisement
metadata:
  name: epsx-l2-advertisement
  namespace: metallb-system
spec:
  ipAddressPools:
  - epsx-ip-pool
EOF

    success "MetalLB installed successfully"
fi

# Install storage classes
if [[ "$INSTALL_STORAGE" == "true" ]]; then
    log "Creating storage classes..."

    # Create fast SSD storage class
    cat <<EOF | kubectl apply -f -
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: fast-ssd
provisioner: kubernetes.io/no-provisioner
volumeBindingMode: WaitForFirstConsumer
allowVolumeExpansion: true
---
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: standard
provisioner: kubernetes.io/no-provisioner
volumeBindingMode: WaitForFirstConsumer
allowVolumeExpansion: true
EOF

    success "Storage classes created successfully"
fi

# Install monitoring stack
if [[ "$INSTALL_MONITORING" == "true" ]]; then
    log "Installing monitoring stack..."

    # Create monitoring namespace
    kubectl create namespace monitoring --dry-run=client -o yaml | kubectl apply -f -

    helm upgrade --install kube-prometheus-stack prometheus-community/kube-prometheus-stack \
        --namespace monitoring \
        --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.resources.requests.storage=50Gi \
        --set prometheus.prometheusSpec.resources.requests.cpu=500m \
        --set prometheus.prometheusSpec.resources.requests.memory=1Gi \
        --set prometheus.prometheusSpec.resources.limits.cpu=1000m \
        --set prometheus.prometheusSpec.resources.limits.memory=2Gi \
        --set grafana.persistence.enabled=true \
        --set grafana.persistence.size=10Gi \
        --set grafana.resources.requests.cpu=200m \
        --set grafana.resources.requests.memory=256Mi \
        --set grafana.adminPassword=admin123 \
        --wait

    success "Monitoring stack installed successfully"
fi

# Create network policies for security
log "Creating network policies..."
cat <<EOF | kubectl apply -f -
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: epsx-network-policy
  namespace: epsx
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    - podSelector:
        matchLabels:
          app.kubernetes.io/name: epsx
    - podSelector:
        matchLabels:
          app.kubernetes.io/name: epsx
          app.kubernetes.io/component: frontend
    - podSelector:
        matchLabels:
          app.kubernetes.io/name: epsx
          app.kubernetes.io/component: admin-frontend
    ports:
    - protocol: TCP
      port: 8080
    - protocol: TCP
      port: 3000
  egress:
  - to:
    - podSelector:
        matchLabels:
          app.kubernetes.io/name: epsx
          app.kubernetes.io/component: postgres
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - podSelector:
        matchLabels:
          app.kubernetes.io/name: epsx
          app.kubernetes.io/component: redis
    ports:
    - protocol: TCP
      port: 6379
  - to: []
    ports:
    - protocol: TCP
      port: 443
    - protocol: TCP
      port: 53
    - protocol: UDP
      port: 53
EOF

success "Network policies created successfully"

# Show final status
log "Cluster setup completed successfully!"
echo ""
success "EPSX Kubernetes cluster is ready for deployment"
echo ""
log "Next steps:"
echo "1. Create secrets: ./scripts/k8s/deploy-k8s.sh production --skip-secrets"
echo "2. Deploy EPSX: ./scripts/k8s/deploy-k8s.sh production"
echo "3. Check status: kubectl get pods -n epsx"

# Show component URLs
if [[ "$INSTALL_INGRESS" == "true" ]]; then
    INGRESS_IP=$(kubectl get svc ingress-nginx-controller -n ingress-nginx -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "")
    INGRESS_HOSTNAME=$(kubectl get svc ingress-nginx-controller -n ingress-nginx -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null || echo "")

    if [[ -n "$INGRESS_IP" ]] || [[ -n "$INGRESS_HOSTNAME" ]]; then
        echo ""
        log "Ingress Controller:"
        if [[ -n "$INGRESS_IP" ]]; then
            echo "  IP: $INGRESS_IP"
        fi
        if [[ -n "$INGRESS_HOSTNAME" ]]; then
            echo "  Hostname: $INGRESS_HOSTNAME"
        fi
    fi
fi

if [[ "$INSTALL_MONITORING" == "true" ]]; then
    echo ""
    log "Monitoring:"
    echo "  Grafana: kubectl port-forward svc/kube-prometheus-stack-grafana 3000:80 -n monitoring"
    echo "  Prometheus: kubectl port-forward svc/kube-prometheus-stack-prometheus 9090:9090 -n monitoring"
fi