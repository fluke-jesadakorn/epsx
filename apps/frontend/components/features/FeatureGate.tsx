// ============================================================================
// SIMPLE FEATURE GATE - REPLACES COMPLEX FEATURE GATES
// ============================================================================
// Simple stub - use FeatureGuard instead

'use client';

import { ReactNode } from 'react';
import { FeatureGuard } from '@/components/guards/FeatureGuard';

interface FeatureGateProps {
  children: ReactNode;
  feature?: string;
  fallback?: ReactNode;
}

export function FeatureGate({ children, feature, fallback }: FeatureGateProps) {
  return (
    <FeatureGuard feature={feature} fallback={fallback}>
      {children}
    </FeatureGuard>
  );
}

export default FeatureGate;