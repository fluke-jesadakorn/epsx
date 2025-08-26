// ============================================================================
// SIMPLE FEATURE GUARD - REPLACES ALL COMPLEX GUARD COMPONENTS
// ============================================================================
// This component replaces 500+ lines of complex guard logic with simple feature-based access
// Uses the unified role system from lib/auth/roles.ts

'use client';

import { ReactNode } from 'react';
import { useAuth } from '@/lib/auth';
import { Role, checkFeatureAccess, checkRoleAccess, SimpleUserClaims } from '@/lib/auth/roles';

// ============================================================================
// SIMPLE FEATURE GUARD PROPS
// ============================================================================

interface FeatureGuardProps {
  children: ReactNode;
  feature?: string;           // Feature-based access (e.g., 'view_eps', 'export_data')
  role?: Role;               // Role-based access (admin, user, guest)
  fallback?: ReactNode;      // What to show when access is denied
  loading?: ReactNode;       // What to show while loading user data
}

// ============================================================================
// MAIN FEATURE GUARD COMPONENT
// ============================================================================

export function FeatureGuard({
  children,
  feature,
  role,
  fallback = <AccessDenied />,
  loading = <Loading />
}: FeatureGuardProps) {
  const { user, isLoading } = useAuth();

  // Show loading while fetching user data
  if (isLoading) {
    return <>{loading}</>;
  }

  // No user - deny access
  if (!user) {
    return <>{fallback}</>;
  }

  const userClaims: SimpleUserClaims = {
    firebase_uid: user.firebase_uid || '',
    email: user.email || '',
    role: (user as any).role || Role.Guest, // Default to guest if no role
    display_name: user.name || undefined,
    name: user.name || undefined,
    avatar_url: undefined,
    is_active: true,
    last_login_at: new Date().toISOString()
  };

  // Check feature access
  if (feature && !checkFeatureAccess(userClaims.role, feature)) {
    return <>{fallback}</>;
  }

  // Check role access
  if (role && !checkRoleAccess(userClaims.role, role)) {
    return <>{fallback}</>;
  }

  // Access granted
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
  <FeatureGuard role={Role.Admin} fallback={fallback}>{children}</FeatureGuard>
);

export const UserOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard role={Role.User} fallback={fallback}>{children}</FeatureGuard>
);

export const GuestOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <FeatureGuard role={Role.Guest} fallback={fallback}>{children}</FeatureGuard>
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