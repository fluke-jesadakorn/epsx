// ============================================================================
// SIMPLE FEATURE GATE - NO PERMISSION CHECKING
// ============================================================================
// Simplified component that just renders children without permission checks

'use client';

import { ReactNode } from 'react';

interface FeatureGateProps {
  children: ReactNode;
  feature?: string;
  fallback?: ReactNode;
}

export function FeatureGate({ children }: FeatureGateProps) {
  // No permission checking - just render children
  return <>{children}</>;
}

export default FeatureGate;