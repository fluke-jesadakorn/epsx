#!/bin/bash
# ============================================================================
# EPSX Microservices - Stop All Services
# ============================================================================

echo "Stopping EPSX Microservices..."

overmind quit 2>/dev/null || true

echo "Services stopped."
