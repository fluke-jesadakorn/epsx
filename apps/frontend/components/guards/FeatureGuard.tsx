// ============================================================================
// PERMISSION-ONLY FEATURE GUARD - CLEAN IMPLEMENTATION
// ============================================================================
// Pure permission-based access control using structured permissions
// Format: "platform:resource:action" (e.g., "epsx:analytics:view", "admin:users:manage")

'use client';

import { ReactNode } from 'react';
import { useAuth } from '@/lib/auth';
import { 
  filterValidPermissions
} from '@/shared/permissions/utils';
import { type TimestampedPermission } from '@/shared/permissions/types';
import { usePermissionExpiry } from '@/hooks/usePermissionExpiry';

// Simplified permission checking using direct permission arrays

// ============================================================================
// PERMISSION GUARD PROPS
// ============================================================================

interface PermissionGuardProps {
  children: ReactNode;
  permission?: string;        // Direct permission check (e.g., "epsx:analytics:view")
  permissions?: string[];     // Multiple permissions (any of them)
  allPermissions?: string[];  // Multiple permissions (all required)
  feature?: string;          // Legacy feature-based access (mapped to permissions)
  fallback?: ReactNode;      // What to show when access is denied
  loading?: ReactNode;       // What to show while loading user data
  // Embedded timestamp support
  validateExpiry?: boolean;   // Enable timestamp-based expiry validation (default: true)
  requireTimeRemaining?: number; // Require at least N minutes remaining (default: 0)
  onExpiredPermission?: (permission: string) => void; // Callback when permission expired
  showExpiryWarning?: boolean; // Show warning when permission expires soon (default: false)
  expiryFallback?: ReactNode; // Custom fallback for expired permissions
}

// ============================================================================
// MAIN PERMISSION GUARD COMPONENT
// ============================================================================

export function PermissionGuard({
  children,
  permission,
  permissions,
  allPermissions,
  feature,
  fallback = <AccessDenied />,
  loading = <Loading />,
  validateExpiry = true,
  requireTimeRemaining = 0,
  onExpiredPermission,
  showExpiryWarning = false,
  expiryFallback
}: PermissionGuardProps) {
  const { user, loading: authLoading } = useAuth();
  const expiry = usePermissionExpiry();

  // Show loading while fetching user data
  if (authLoading) {
    return <>{loading}</>;
  }

  // No user - deny access
  if (!user) {
    return <>{fallback}</>;
  }

  // User permissions available directly from user object

  // Helper function to check permission with optional expiry validation
  const checkPermissionWithExpiry = (perm: string): boolean => {
    if (validateExpiry) {
      // Check if permission is expired
      if (onExpiredPermission && expiry.expiryInfo.expired.some((ep: TimestampedPermission) => ep.permission === perm)) {
        onExpiredPermission(perm);
        return false;
      }

      // Check if permission has enough time remaining
      if (requireTimeRemaining > 0) {
        const permWithExpiry = expiry.allPermissionsWithExpiry.find(tp => tp.basePermission === perm);
        if (permWithExpiry && permWithExpiry.timeRemaining) {
          const minutesRemaining = permWithExpiry.timeRemaining / (1000 * 60);
          if (minutesRemaining < requireTimeRemaining) {
            return false;
          }
        }
      }

      // Use permissions array directly for timestamp checking
      const validPermissions = filterValidPermissions(user.permissions);
      return validPermissions.includes(perm);
    } else {
      // Simple permission check
      return user.permissions.includes(perm);
    }
  };

  // Check single permission
  if (permission && !checkPermissionWithExpiry(permission)) {
    return <>{expiryFallback || fallback}</>;
  }

  // Check multiple permissions (any of them)
  if (permissions) {
    const hasAnyValid = validateExpiry 
      ? permissions.some(perm => checkPermissionWithExpiry(perm))
      : permissions.some(perm => user.permissions.includes(perm));
    
    if (!hasAnyValid) {
      return <>{expiryFallback || fallback}</>;
    }
  }

  // Check multiple permissions (all required)
  if (allPermissions) {
    const hasAllValid = validateExpiry 
      ? allPermissions.every(perm => checkPermissionWithExpiry(perm))
      : allPermissions.every(perm => user.permissions.includes(perm));
    
    if (!hasAllValid) {
      return <>{expiryFallback || fallback}</>;
    }
  }

  // Check legacy feature access (mapped to permissions)
  if (feature) {
    const permissions = validateExpiry ? filterValidPermissions(user.permissions) : user.permissions;
    const hasFeatureAccess = feature === 'admin' 
      ? permissions.some(p => p.startsWith('admin:'))
      : permissions.some(p => p.includes(feature));
    
    if (!hasFeatureAccess) {
      return <>{expiryFallback || fallback}</>;
    }
  }

  // Show expiry warning if enabled and permissions are expiring soon
  if (showExpiryWarning && expiry.hasExpiringSoon) {
    return (
      <>
        <ExpiryWarningBanner expiry={expiry} />
        {children}
      </>
    );
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

function ExpiryWarningBanner({ expiry }: { expiry: ReturnType<typeof usePermissionExpiry> }) {
  if (!expiry.hasExpiringSoon) return null;

  return (
    <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-md p-3 mb-4">
      <div className="flex">
        <div className="flex-shrink-0">
          <svg className="h-5 w-5 text-yellow-400" viewBox="0 0 20 20" fill="currentColor">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
          </svg>
        </div>
        <div className="ml-3">
          <h3 className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
            Permissions Expiring Soon
          </h3>
          <div className="mt-2 text-sm text-yellow-700 dark:text-yellow-300">
            <p>
              {expiry.expiryInfo.expiringSoon.length} permission{expiry.expiryInfo.expiringSoon.length !== 1 ? 's' : ''} will expire soon.
              {expiry.nextExpiringPermission && (
                <span className="block mt-1">
                  Next: {expiry.nextExpiringPermission.basePermission} in {expiry.formatTimeUntilExpiry(expiry.nextExpiringPermission.expiresAt)}
                </span>
              )}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// PERMISSION-BASED CONVENIENCE COMPONENTS
// ============================================================================

export const AdminOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="admin:*:*" fallback={fallback}>{children}</PermissionGuard>
);

export const UserManagementOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard 
    permissions={["admin:users:manage", "epsx:users:manage"]} 
    fallback={fallback}
  >
    {children}
  </PermissionGuard>
);

export const SystemManagementOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard 
    permissions={["admin:system:manage", "admin:*:*"]} 
    fallback={fallback}
  >
    {children}
  </PermissionGuard>
);

// Analytics permission guards
export const AnalyticsViewOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="epsx:analytics:view" fallback={fallback}>{children}</PermissionGuard>
);

export const AnalyticsExportOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="epsx:analytics:export" fallback={fallback}>{children}</PermissionGuard>
);

export const AnalyticsAdvancedOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="epsx:analytics:advanced" fallback={fallback}>{children}</PermissionGuard>
);

// Real-time permission guards
export const RealtimeOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="epsx:realtime:access" fallback={fallback}>{children}</PermissionGuard>
);

// Profile permission guards
export const ProfileManagementOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="epsx:profile:manage" fallback={fallback}>{children}</PermissionGuard>
);

// Notification permission guards
export const NotificationsOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="epsx:notifications:receive" fallback={fallback}>{children}</PermissionGuard>
);

// Billing permission guards
export const BillingOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="epsx:billing:manage" fallback={fallback}>{children}</PermissionGuard>
);

// Platform permission guards
export const EpsxPlatformOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permissions={["epsx:*:*", "admin:*:*"]} fallback={fallback}>{children}</PermissionGuard>
);

export const PayPlatformOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permissions={["epsx-pay:*:*", "admin:*:*"]} fallback={fallback}>{children}</PermissionGuard>
);

export const TokenPlatformOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permissions={["epsx-token:*:*", "admin:*:*"]} fallback={fallback}>{children}</PermissionGuard>
);

// ============================================================================
// HOOK-BASED PERMISSION GUARDS (FOR CONDITIONAL LOGIC)
// ============================================================================

export function usePermissionGuard(permission: string): boolean {
  const { user } = useAuth();
  return user ? user.permissions.includes(permission) : false;
}

export function useMultiPermissionGuard(permissions: string[]): boolean {
  const { user } = useAuth();
  return user ? permissions.some(perm => user.permissions.includes(perm)) : false;
}

export function useAdminGuard(): boolean {
  const { user } = useAuth();
  return user ? user.permissions.some(p => p.startsWith('admin:')) : false;
}

export function useFeatureGuard(feature: string): boolean {
  const { user } = useAuth();
  if (!user) return false;
  return feature === 'admin' 
    ? user.permissions.some(p => p.startsWith('admin:'))
    : user.permissions.some(p => p.includes(feature));
}

// ============================================================================
// TIMESTAMP-AWARE HOOK-BASED PERMISSION GUARDS
// ============================================================================

export function usePermissionGuardWithTime(permission: string): boolean {
  const { user } = useAuth();
  if (!user) return false;
  const validPermissions = filterValidPermissions(user.permissions);
  return validPermissions.includes(permission);
}

export function useMultiPermissionGuardWithTime(permissions: string[]): boolean {
  const { user } = useAuth();
  if (!user) return false;
  const validPermissions = filterValidPermissions(user.permissions);
  return permissions.some(perm => validPermissions.includes(perm));
}

export function useAdminGuardWithTime(): boolean {
  const { user } = useAuth();
  if (!user) return false;
  const validPermissions = filterValidPermissions(user.permissions);
  return validPermissions.some(p => p.startsWith('admin:'));
}

export function useFeatureGuardWithTime(feature: string): boolean {
  const { user } = useAuth();
  if (!user) return false;
  const validPermissions = filterValidPermissions(user.permissions);
  return feature === 'admin' 
    ? validPermissions.some(p => p.startsWith('admin:'))
    : validPermissions.some(p => p.includes(feature));
}

export function usePermissionGuardWithDuration(permission: string, durationMinutes: number = 60): boolean {
  const { user } = useAuth();
  const expiry = usePermissionExpiry();
  
  if (!user) return false;
  
  const validPermissions = filterValidPermissions(user.permissions);
  if (!validPermissions.includes(permission)) return false;
  
  // Check if permission has enough time remaining
  const permWithExpiry = expiry.allPermissionsWithExpiry.find(tp => tp.basePermission === permission);
  if (permWithExpiry && permWithExpiry.timeRemaining) {
    const minutesRemaining = permWithExpiry.timeRemaining / (1000 * 60);
    return minutesRemaining >= durationMinutes;
  }
  
  // If no expiry, assume it has enough time
  return true;
}

// ============================================================================
// TIMESTAMP-AWARE CONVENIENCE COMPONENTS
// ============================================================================

export const AdminOnlyWithTime = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="admin:*:*" validateExpiry={true} fallback={fallback}>{children}</PermissionGuard>
);

export const AnalyticsViewOnlyWithTime = ({ children, fallback, showExpiryWarning = true }: { 
  children: ReactNode, 
  fallback?: ReactNode,
  showExpiryWarning?: boolean 
}) => (
  <PermissionGuard 
    permission="epsx:analytics:view" 
    validateExpiry={true} 
    showExpiryWarning={showExpiryWarning}
    fallback={fallback}
  >
    {children}
  </PermissionGuard>
);

export const AnalyticsExportOnlyWithTime = ({ children, fallback, requireTimeRemaining = 30 }: { 
  children: ReactNode, 
  fallback?: ReactNode,
  requireTimeRemaining?: number 
}) => (
  <PermissionGuard 
    permission="epsx:analytics:export" 
    validateExpiry={true} 
    requireTimeRemaining={requireTimeRemaining}
    fallback={fallback}
  >
    {children}
  </PermissionGuard>
);

export const RealtimeOnlyWithTime = ({ children, fallback, showExpiryWarning = true }: { 
  children: ReactNode, 
  fallback?: ReactNode,
  showExpiryWarning?: boolean 
}) => (
  <PermissionGuard 
    permission="epsx:realtime:access" 
    validateExpiry={true} 
    showExpiryWarning={showExpiryWarning}
    fallback={fallback}
  >
    {children}
  </PermissionGuard>
);

// Component that requires permission with specific duration remaining
export const TimeSensitiveFeatureGuard = ({ 
  children, 
  permission, 
  requireMinutes = 60,
  fallback,
  expiryFallback 
}: { 
  children: ReactNode,
  permission: string,
  requireMinutes?: number,
  fallback?: ReactNode,
  expiryFallback?: ReactNode 
}) => (
  <PermissionGuard 
    permission={permission} 
    validateExpiry={true} 
    requireTimeRemaining={requireMinutes}
    fallback={fallback}
    expiryFallback={expiryFallback || <TimeRemainingDenied requiredMinutes={requireMinutes} />}
  >
    {children}
  </PermissionGuard>
);

function TimeRemainingDenied({ requiredMinutes }: { requiredMinutes: number }) {
  return (
    <div className="flex items-center justify-center p-8">
      <div className="text-center">
        <h3 className="text-lg font-semibold text-orange-900 dark:text-orange-100">
          Insufficient Time Remaining
        </h3>
        <p className="mt-2 text-sm text-orange-600 dark:text-orange-400">
          This feature requires at least {requiredMinutes} minutes of permission time remaining.
        </p>
      </div>
    </div>
  );
}

// ============================================================================
// LEGACY COMPATIBILITY COMPONENTS (DEPRECATED)
// ============================================================================

// These components map legacy feature names to permission checks for backward compatibility

/**
 * @deprecated Use AnalyticsViewOnly instead
 */
export const ViewEpsOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="epsx:analytics:view" fallback={fallback}>{children}</PermissionGuard>
);

/**
 * @deprecated Use AnalyticsExportOnly instead  
 */
export const ExportDataOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="epsx:analytics:export" fallback={fallback}>{children}</PermissionGuard>
);

/**
 * @deprecated Use AnalyticsAdvancedOnly instead
 */
export const AdvancedFiltersOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="epsx:analytics:advanced" fallback={fallback}>{children}</PermissionGuard>
);

/**
 * @deprecated Use ProfileManagementOnly instead
 */
export const ProfileOnly = ({ children, fallback }: { children: ReactNode, fallback?: ReactNode }) => (
  <PermissionGuard permission="epsx:profile:manage" fallback={fallback}>{children}</PermissionGuard>
);

// Main component is also exported as FeatureGuard for backward compatibility
export const FeatureGuard = PermissionGuard;