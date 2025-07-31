'use client';

import React, { ReactNode, Suspense } from 'react';
import { 
  EnhancedPermissionProvider, 
  ServerStateLoading, 
  ServerStateError,
  useEnhancedPermissionContext,
  usePermissions,
  usePaymentStatus,
  useFeatureAccess
} from './EnhancedPermissionProvider';
import type { EnhancedServerData } from './EnhancedServerPermissionProvider';

interface EnhancedClientWrapperProps {
  children: ReactNode;
  serverData: EnhancedServerData;
  loadingComponent?: ReactNode;
  errorComponent?: ReactNode;
  enableRefresh?: boolean;
  refreshInterval?: number;
  onError?: (error: string) => void;
}

/**
 * Enhanced client wrapper that combines server data with client-side state management
 */
export function EnhancedClientWrapper({
  children,
  serverData,
  loadingComponent,
  errorComponent,
  enableRefresh = false,
  refreshInterval = 5 * 60 * 1000,
  onError
}: EnhancedClientWrapperProps) {
  // Transform server data to initial state
  const initialData = {
    permissions: serverData.permissions,
    paymentStatus: serverData.paymentStatus,
    plans: serverData.plans,
    featureAccess: serverData.featureAccess,
    rankingAccess: serverData.rankingAccess,
    loading: false,
    error: serverData.error,
    lastUpdated: serverData.timestamp
  };

  return (
    <EnhancedPermissionProvider
      initialData={initialData}
      enableRefresh={enableRefresh}
      refreshInterval={refreshInterval}
      onError={onError}
    >
      <Suspense fallback={loadingComponent || <DefaultLoadingComponent />}>
        {/* Show error state if server data has errors */}
        {serverData.error && !errorComponent && <ServerStateError />}
        {serverData.error && errorComponent}
        
        {/* Show loading state during client hydration */}
        <ServerStateLoading>{loadingComponent || <DefaultLoadingComponent />}</ServerStateLoading>
        
        {children}
      </Suspense>
    </EnhancedPermissionProvider>
  );
}

/**
 * Default loading component
 */
function DefaultLoadingComponent() {
  return (
    <div className="animate-pulse space-y-4">
      <div className="h-4 bg-gray-200 rounded w-3/4"></div>
      <div className="h-4 bg-gray-200 rounded w-1/2"></div>
      <div className="h-4 bg-gray-200 rounded w-5/6"></div>
    </div>
  );
}

/**
 * Permission guard component that shows/hides content based on permissions
 */
interface PermissionGuardProps {
  children: ReactNode;
  permission?: string;
  feature?: string;
  requireRanking?: boolean;
  fallback?: ReactNode;
  loading?: ReactNode;
}

export function PermissionGuard({
  children,
  permission,
  feature,
  requireRanking = false,
  fallback,
  loading
}: PermissionGuardProps) {
  const { state } = useEnhancedPermissionContext();

  // Show loading state
  if (state.loading) {
    return loading || <DefaultLoadingComponent />;
  }

  // Show error state
  if (state.error) {
    return fallback || (
      <div className="p-3 bg-yellow-50 border border-yellow-200 rounded text-yellow-700">
        <p className="text-sm">Unable to verify permissions</p>
      </div>
    );
  }

  // Check permission
  if (permission && state.permissions) {
    const hasPermission = state.permissions.some(p => 
      p.name === permission || 
      p.resource === permission || 
      `${p.resource}.${p.action}` === permission
    );
    if (!hasPermission) {
      return fallback || null;
    }
  }

  // Check feature access
  if (feature) {
    const hasFeature = state.featureAccess[feature];
    if (!hasFeature) {
      return fallback || null;
    }
  }

  // Check ranking access
  if (requireRanking && !state.rankingAccess) {
    return fallback || null;
  }

  return <>{children}</>;
}

/**
 * Payment tier guard component
 */
interface PaymentTierGuardProps {
  children: ReactNode;
  minimumTier: 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM';
  fallback?: ReactNode;
  loading?: ReactNode;
}

export function PaymentTierGuard({
  children,
  minimumTier,
  fallback,
  loading
}: PaymentTierGuardProps) {
  const { state } = useEnhancedPermissionContext();

  // Show loading state
  if (state.loading) {
    return loading || <DefaultLoadingComponent />;
  }

  // Show error state  
  if (state.error) {
    return fallback || null;
  }

  // Check payment tier
  if (state.paymentStatus && typeof state.paymentStatus === 'object' && 'status' in state.paymentStatus) {
    const paymentData = state.paymentStatus as any;
    if (paymentData.status === 'completed') {
      const tierLevels = { BRONZE: 1, SILVER: 2, GOLD: 3, PLATINUM: 4 };
      const userLevel = tierLevels[(paymentData.userLevel as keyof typeof tierLevels) || 'BRONZE'] || 0;
      const requiredLevel = tierLevels[minimumTier];

      if (userLevel >= requiredLevel) {
        return <>{children}</>;
      }
    }
  }

  return fallback || null;
}

// Re-export hooks for convenience
export {
  useEnhancedPermissionContext,
  usePermissions,
  usePaymentStatus,
  useFeatureAccess
} from './EnhancedPermissionProvider';