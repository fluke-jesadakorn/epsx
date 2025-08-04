'use client';

import React from 'react';
import { usePermission } from '@/hooks/useAuth';

interface FeatureGuardProps {
  feature: string;
  children: React.ReactNode;
  fallback?: React.ReactNode;
  loadingComponent?: React.ReactNode;
}

export function FeatureGuard({ 
  feature, 
  children, 
  fallback = <div>Access denied</div>,
  loadingComponent = <div>Checking permissions...</div>
}: FeatureGuardProps) {
  const { hasPermission, loading } = usePermission(feature);
  
  if (loading) {
    return <>{loadingComponent}</>;
  }
  
  if (!hasPermission) {
    return <>{fallback}</>;
  }
  
  return <>{children}</>;
}