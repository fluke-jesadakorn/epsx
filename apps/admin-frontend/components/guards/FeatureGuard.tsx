// ============================================================================
// SIMPLE FEATURE GUARD - REPLACES ALL COMPLEX GUARD COMPONENTS
// ============================================================================
// This component replaces 500+ lines of complex guard logic with simple feature-based access
// Uses the unified role system from lib/auth/roles.ts

'use client';

import { ReactNode } from 'react';
import { useAuth } from '@/lib/auth';

// ============================================================================
// SIMPLE FEATURE GUARD PROPS
// ============================================================================

interface FeatureGuardProps {
  children: ReactNode;
  permission?: string;        // Structured permission (e.g., 'epsx:analytics:view')
  feature?: string;          // Legacy feature (will be mapped to permission)
  isAdmin?: boolean;         // Admin-only access
  fallback?: ReactNode;      // What to show when access is denied
  loading?: ReactNode;       // What to show while loading user data
}

// ============================================================================
// MAIN FEATURE GUARD COMPONENT
// ============================================================================

export function FeatureGuard({
  children,
  permission,
  feature,
  isAdmin,
  fallback = <AccessDenied />,
  loading = <Loading />
}: FeatureGuardProps) {
  const { user, isLoading, isAdmin: checkIsAdmin, can } = useAuth();

  if (isLoading) {
    return <>{loading}</>;
  }

  if (!user) {
    return <>{fallback}</>;
  }

  if (isAdmin && !checkIsAdmin()) {
    return <>{fallback}</>;
  }

  if (permission && !can(permission)) {
    return <>{fallback}</>;
  }

  if (feature) {
    const mappedPermission = mapFeatureToPermission(feature);
    if (!can(mappedPermission)) {
      return <>{fallback}</>;
    }
  }

  return <>{children}</>;
}

// ============================================================================
// DEFAULT FALLBACK COMPONENTS
// ============================================================================

function AccessDenied() {
  return (
    <div className="flex items-center justify-center p-8">
      <div className="text-center">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          Access Denied
        </h3>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          You don't have permission to access this feature.
        </p>
      </div>
    </div>
  );
}

function Loading() {
  return (
    <div className="flex items-center justify-center p-8">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
    </div>
  );
}

// ============================================================================
// CONVENIENCE EXPORT FUNCTIONS
// ============================================================================

export const AdminOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard isAdmin fallback={fallback}>{children}</FeatureGuard>
);

export const UserOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard permission="epsx:analytics:view" fallback={fallback}>{children}</FeatureGuard>
);

export const GuestOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard permission="epsx:analytics:view" fallback={fallback}>{children}</FeatureGuard>
);

// Feature-specific guards
export const ViewEpsOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard feature="view_eps" fallback={fallback}>{children}</FeatureGuard>
);

export const ExportDataOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard feature="export_data" fallback={fallback}>{children}</FeatureGuard>
);

export const RealtimeOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard feature="realtime" fallback={fallback}>{children}</FeatureGuard>
);

export const ProfileOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard feature="profile" fallback={fallback}>{children}</FeatureGuard>
);

export const NotificationsOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard feature="notifications" fallback={fallback}>{children}</FeatureGuard>
);

export const BillingOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard feature="billing" fallback={fallback}>{children}</FeatureGuard>
);

export const AdvancedFiltersOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard feature="advanced_filters" fallback={fallback}>{children}</FeatureGuard>
);

function mapFeatureToPermission(feature: string): string {
  const featureMap: Record<string, string> = {
    'view_eps': 'epsx:analytics:view',
    'export_data': 'epsx:analytics:export',
    'realtime': 'epsx:realtime:access',
    'profile': 'epsx:profile:manage',
    'notifications': 'epsx:notifications:receive',
    'billing': 'epsx:billing:manage',
    'advanced_filters': 'epsx:analytics:advanced',
  };
  
  return featureMap[feature] || `epsx:${feature}:access`;
}