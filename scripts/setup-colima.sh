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

# A fresh developer machine may not have the Colima toolchain yet. Install only
# the missing Homebrew packages so this setup remains idempotent on machines
# that already have Docker/Kubernetes configured. Never replace an existing
# installation or change a user's unrelated Homebrew packages.
if ! command -v brew &> /dev/null; then
  echo "❌ Homebrew is required to install Colima automatically."
  echo "   Install Homebrew from https://brew.sh and rerun this script."
  exit 1
fi

missing_packages=()
command -v colima &> /dev/null || missing_packages+=(colima)
command -v docker &> /dev/null || missing_packages+=(docker)
if command -v docker &> /dev/null; then
  docker buildx version &> /dev/null || missing_packages+=(docker-buildx)
else
  missing_packages+=(docker-buildx)
fi
command -v kubectl &> /dev/null || missing_packages+=(kubernetes-cli)

if [ "${#missing_packages[@]}" -gt 0 ]; then
  echo "📦 Installing missing Colima dependencies: ${missing_packages[*]}"
  brew install "${missing_packages[@]}"
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
