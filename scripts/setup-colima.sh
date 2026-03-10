#!/bin/bash
# ============================================================================
# EPSX - Colima Kubernetes Setup
# ============================================================================
# This script configures and starts Colima with Kubernetes and multi-architecture
# (amd64/arm64) support using Rosetta 2 (on Apple Silicon).

set -euo pipefail

PROFILE="epsx"
CPU=${CPU:-4}
MEMORY=${MEMORY:-8}
DISK=${DISK:-60}

echo "============================================"
echo "  Setting up Colima for EPSX ($PROFILE)"
echo "  CPU: $CPU | Memory: ${MEMORY}GB | Disk: ${DISK}GB"
echo "============================================"

if ! command -v colima &> /dev/null; then
  echo "❌ Colima is not installed. Please install it first:"
  echo "   brew install colima docker docker-buildx kubernetes-cli"
  exit 1
fi

# Detect architecture to enable Rosetta if on Apple Silicon (arm64)
ARCH=$(uname -m)
EXTRA_ARGS=""

if [ "$ARCH" = "arm64" ]; then
  echo "✅ Apple Silicon detected. Enabling Rosetta 2 for x86_64 emulation."
  EXTRA_ARGS="--vm-type vz --vz-rosetta"
fi

echo "🚀 Starting Colima..."
# shellcheck disable=SC2086
colima start --profile "$PROFILE" --kubernetes --cpu "$CPU" --memory "$MEMORY" --disk "$DISK" $EXTRA_ARGS

echo ""
echo "✅ Colima setup complete!"
echo "🐳 Docker context is now using Colima: docker context use colima-$PROFILE"
echo "☸️  Kubernetes context is now using Colima: kubectl config use-context colima-$PROFILE"

# Verify K8s
echo ""
echo "Verifying Kubernetes cluster..."
kubectl cluster-info
kubectl get nodes
