/**
 * UNIFIED PERMISSION GUARD COMPONENT
 * 
 * Consolidates permission checking logic for both admin-frontend and frontend apps.
 * Replaces AdminPermissionGuard and PlatformPermissionGuard with a single,
 * platform-aware component that works consistently across both applications.
 * 
 * Features:
 * - Platform-aware permission checking (admin vs frontend)
 * - Progressive auth level support
 * - Detailed error messaging with permission details
 * - Legacy compatibility with existing props
 * - Upgrade prompts for frontend users
 * - Convenience components for common use cases
 */
'use client';

import { ReactNode } from 'react';
import React from 'react';
import { getAuthHook, type UnifiedAuthInterface } from './UnifiedAuthAdapter';

// Platform-specific types
export type Platform = 'admin' | 'frontend';
export type AuthLevel = 'ANONYMOUS' | 'AUTHENTICATED' | 'PROGRESSIVE' | 'FULL';

// Utility function for platform display names
const getPlatformDisplayName = (platform: string): string => {
  switch (platform.toLowerCase()) {
    case 'epsx': return 'EPSX Analytics Platform';
    case 'epsx-pay': return 'EPSX Pay';
    case 'epsx-token': return 'EPSX Token';
    case 'admin': return 'Admin Portal';
    default: return platform.toUpperCase();
  }
};

// Base permission guard props
export interface UnifiedPermissionGuardProps {
  /**
   * Platform context - determines which auth system to use
   */
  platform: Platform;

  /**
   * Content to show when user has the required permissions
   */
  children: ReactNode;

  /**
   * Required permissions (any one will grant access)
   */
  permissions?: string[];

  /**
   * Legacy single permission string (for backward compatibility)
   */
  permission?: string;

  /**
   * Legacy resource + action pattern
   */
  resource?: string;
  action?: string;

  /**
   * Permission platform scope (defaults to 'epsx' for frontend, 'admin' for admin)
   */
  permissionPlatform?: string;

  /**
   * Optional fallback content for insufficient permissions
   */
  fallback?: ReactNode;

  /**
   * Custom message for permission denial
   */
  deniedMessage?: string;

  /**
   * Action name for better UX messaging
   */
  actionName?: string;

  /**
   * Whether to show detailed permission info (admin only)
   */
  showPermissionDetails?: boolean;

  /**
   * Minimum authentication level required (admin only)
   */
  minAuthLevel?: AuthLevel;

  /**
   * Legacy props for backward compatibility
   */
  role?: string;
  tier?: string;
  requireAll?: boolean;
  showAccessDenied?: boolean;
  showUpgradePrompt?: boolean;
  adminAction?: 'manage' | 'read' | 'write' | 'delete';
}

// Platform-specific auth hook selector using the adapter
function useAuthHook(platform: Platform): UnifiedAuthInterface {
  return getAuthHook(platform);
}

export default function UnifiedPermissionGuard({
  platform,
  children,
  permissions,
  permission,
  resource,
  action,
  permissionPlatform,
  fallback,
  deniedMessage,
  actionName,
  showPermissionDetails = false,
  minAuthLevel = 'AUTHENTICATED',
  // Legacy props
  role,
  tier,
  requireAll = false,
  showAccessDenied = true,
  showUpgradePrompt = true,
  adminAction
}: UnifiedPermissionGuardProps) {
  const auth = useAuthHook(platform);

  // Not authenticated
  if (!auth.user) {
    return fallback ? <>{fallback}</> : null;
  }

  // Check minimum auth level (admin only)
  if (platform === 'admin' && minAuthLevel && auth.canAccess) {
    if (!auth.canAccess(minAuthLevel)) {
      // Return progressive auth gate equivalent
      return (
        <div className="flex items-center justify-center min-h-[300px] p-6">
          <div className="max-w-md w-full">
            <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
              <h3 className="text-sm font-medium text-blue-800">
                Authentication Required
              </h3>
              <p className="mt-1 text-sm text-blue-700">
                {actionName ? `To ${actionName}, you` : 'You'} need to complete additional authentication steps.
              </p>
            </div>
          </div>
        </div>
      );
    }
  }

  // Build permissions array from various sources
  const requiredPermissions: string[] = [];
  const targetPlatform = permissionPlatform || (platform === 'admin' ? 'admin' : 'epsx');

  // New permissions prop
  if (permissions) {
    requiredPermissions.push(...permissions);
  }

  // Legacy permission prop
  if (permission) {
    requiredPermissions.push(permission);
  }

  // Legacy resource + action
  if (resource && action) {
    requiredPermissions.push(`${targetPlatform}:${resource}:${action}`);
  }

  // Legacy admin action shortcuts
  if (adminAction && resource) {
    requiredPermissions.push(`${targetPlatform}:${resource}:${adminAction}`);
  }

  // Legacy role check (converted to permission check)
  if (role) {
    if (role.toLowerCase() === 'admin') {
      // PROPOSAL: Map legacy 'admin' role to basic admin view permission
      // Strict separation: 'admin' no longer implies '*:*'
      requiredPermissions.push('admin:users:view');
    }
  }

  // Legacy tier check (simplified)
  if (tier) {
    const tierPermission = `${targetPlatform}:tier:${tier}`;
    requiredPermissions.push(tierPermission);
  }

  // If no permissions specified, just show content
  if (requiredPermissions.length === 0) {
    return <>{children}</>;
  }

  // Check permissions based on requireAll flag
  const hasRequiredPermission = requireAll
    ? auth.hasAllPermissions(requiredPermissions)
    : auth.hasAnyPermission(requiredPermissions);

  if (hasRequiredPermission) {
    return <>{children}</>;
  }

  // Show custom fallback if provided
  if (fallback) {
    return <>{fallback}</>;
  }

  // Don't show access denied if explicitly disabled
  if (!showAccessDenied) {
    return null;
  }

  // Show upgrade prompt for frontend users with tier/role requirements
  if (platform === 'frontend' && showUpgradePrompt && (tier || role)) {
    return (
      <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <div className="flex items-start">
          <div className="flex-shrink-0">
            <span className="text-lg" role="img" aria-hidden="true">⭐</span>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-blue-800">
              Upgrade Required
            </h3>
            <p className="mt-1 text-sm text-blue-700">
              {tier && `This feature requires ${tier} tier or higher. `}
              {role && `This feature requires ${role} role or higher.`}
            </p>
            <div className="mt-2">
              <button className="text-sm font-medium text-blue-800 underline hover:text-blue-900">
                Upgrade Account
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Default permission denied view (enhanced for admin)
  if (platform === 'admin') {
    return (
      <div className="flex items-center justify-center min-h-[300px] p-6">
        <div className="max-w-md w-full">
          <div className="p-4 bg-amber-50 border border-amber-200 rounded-lg">
            <div className="flex items-start">
              <div className="flex-shrink-0">
                <span className="text-lg" role="img" aria-hidden="true">🔒</span>
              </div>
              <div className="ml-3 space-y-3">
                <div>
                  <h3 className="text-sm font-medium text-amber-800">
                    Insufficient Permissions
                  </h3>
                </div>

                <p className="text-sm text-amber-700">
                  {deniedMessage || `You don't have permission to ${actionName || 'access this feature'}.`}
                </p>

                {showPermissionDetails && requiredPermissions.length > 0 && (
                  <div className="space-y-2">
                    <div>
                      <p className="text-xs font-medium text-amber-800">Required (any one):</p>
                      <ul className="text-xs mt-1 space-y-1">
                        {requiredPermissions.map((permission, index) => (
                          <li key={index} className="font-mono bg-amber-100 px-2 py-1 rounded">
                            {permission}
                          </li>
                        ))}
                      </ul>
                    </div>

                    <div>
                      <p className="text-xs font-medium text-amber-800">Your permissions:</p>
                      {auth.permissions.length > 0 ? (
                        <ul className="text-xs mt-1 space-y-1 max-h-20 overflow-y-auto">
                          {auth.permissions.map((permission, index) => (
                            <li key={index} className="font-mono bg-gray-100 px-2 py-1 rounded">
                              {permission}
                            </li>
                          ))}
                        </ul>
                      ) : (
                        <p className="text-xs mt-1 italic text-amber-700">No permissions found</p>
                      )}
                    </div>
                  </div>
                )}

                <p className="text-xs text-amber-700">
                  Contact your system administrator to request the necessary permissions.
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Simple denied view for frontend
  return (
    <div className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
      <div className="text-center">
        <span className="text-2xl" role="img" aria-hidden="true">🚫</span>
        <h3 className="mt-2 text-sm font-medium text-gray-900">
          Access Denied
        </h3>
        <p className="mt-1 text-sm text-gray-500">
          {deniedMessage || "You don't have permission to access this feature."}
        </p>
      </div>
    </div>
  );
}

// ============================================================================
// CONVENIENCE COMPONENTS - PLATFORM AGNOSTIC
// ============================================================================

export function RequirePermission({
  platform,
  permission,
  permissionPlatform,
  children,
  fallback = null
}: {
  platform: Platform;
  permission: string;
  permissionPlatform?: string;
  children: ReactNode;
  fallback?: ReactNode;
}) {
  return (
    <UnifiedPermissionGuard
      platform={platform}
      permissions={[permission]}
      permissionPlatform={permissionPlatform}
      fallback={fallback}
      actionName="access this feature"
    >
      {children}
    </UnifiedPermissionGuard>
  );
}

export function RequireRole({
  platform,
  role,
  children,
  fallback = null
}: {
  platform: Platform;
  role: string;
  children: ReactNode;
  fallback?: ReactNode;
}) {
  return (
    <UnifiedPermissionGuard
      platform={platform}
      role={role}
      fallback={fallback}
      actionName={`access ${role} features`}
    >
      {children}
    </UnifiedPermissionGuard>
  );
}

export function RequireTier({
  platform,
  tier,
  children,
  fallback = null
}: {
  platform: Platform;
  tier: string;
  children: ReactNode;
  fallback?: ReactNode;
}) {
  return (
    <UnifiedPermissionGuard
      platform={platform}
      tier={tier}
      fallback={fallback}
      actionName={`access ${tier} tier features`}
      showUpgradePrompt={platform === 'frontend'}
    >
      {children}
    </UnifiedPermissionGuard>
  );
}

export function RequireAccess({
  platform,
  resource,
  action,
  permissionPlatform,
  children,
  fallback = null
}: {
  platform: Platform;
  resource: string;
  action: string;
  permissionPlatform?: string;
  children: ReactNode;
  fallback?: ReactNode;
}) {
  return (
    <UnifiedPermissionGuard
      platform={platform}
      resource={resource}
      action={action}
      permissionPlatform={permissionPlatform}
      fallback={fallback}
      actionName={`${action} ${resource}`}
    >
      {children}
    </UnifiedPermissionGuard>
  );
}

// ============================================================================
// ADMIN-SPECIFIC CONVENIENCE COMPONENTS
// ============================================================================

export function RequireAdminPermission({
  permission,
  permissionPlatform,
  children,
  fallback = null
}: {
  permission: string;
  permissionPlatform?: string;
  children: ReactNode;
  fallback?: ReactNode;
}) {
  return (
    <UnifiedPermissionGuard
      platform="admin"
      permissions={[permission]}
      permissionPlatform={permissionPlatform}
      fallback={fallback}
      actionName="access this feature"
      showPermissionDetails={true}
    >
      {children}
    </UnifiedPermissionGuard>
  );
}

export function RequireUserManagement({
  children,
  fallback = null
}: {
  children: ReactNode;
  fallback?: ReactNode;
}) {
  return (
    <UnifiedPermissionGuard
      platform="admin"
      permissions={['admin:users:manage', 'admin:users:*']}
      fallback={fallback}
      actionName="manage users"
      showPermissionDetails={true}
    >
      {children}
    </UnifiedPermissionGuard>
  );
}

export function RequireSystemManagement({
  children,
  fallback = null
}: {
  children: ReactNode;
  fallback?: ReactNode;
}) {
  return (
    <UnifiedPermissionGuard
      platform="admin"
      permissions={['admin:system:manage', 'admin:system:*']}
      fallback={fallback}
      actionName="manage system settings"
      showPermissionDetails={true}
    >
      {children}
    </UnifiedPermissionGuard>
  );
}

export function RequireAnalyticsAccess({
  children,
  fallback = null
}: {
  children: ReactNode;
  fallback?: ReactNode;
}) {
  return (
    <UnifiedPermissionGuard
      platform="admin"
      permissions={['admin:analytics:view', 'admin:analytics:*']}
      fallback={fallback}
      actionName="view analytics"
      showPermissionDetails={true}
    >
      {children}
    </UnifiedPermissionGuard>
  );
}

export function RequirePlatformManagement({
  children,
  fallback = null
}: {
  children: ReactNode;
  fallback?: ReactNode;
}) {
  return (
    <UnifiedPermissionGuard
      platform="admin"
      permissions={['admin:platforms:manage', 'admin:platforms:*']}
      fallback={fallback}
      actionName="manage platforms"
      showPermissionDetails={true}
    >
      {children}
    </UnifiedPermissionGuard>
  );
}

export function RequireSecurityAccess({
  children,
  fallback = null
}: {
  children: ReactNode;
  fallback?: ReactNode;
}) {
  return (
    <UnifiedPermissionGuard
      platform="admin"
      permissions={['admin:security:*']}
      fallback={fallback}
      actionName="access security features"
      showPermissionDetails={true}
    >
      {children}
    </UnifiedPermissionGuard>
  );
}

// ============================================================================
// HIGHER-ORDER COMPONENT FOR WRAPPING COMPONENTS
// ============================================================================

export function withUnifiedPermissions<P extends object>(
  Component: React.ComponentType<P>,
  platform: Platform,
  permissions: string[],
  options?: {
    actionName?: string;
    showPermissionDetails?: boolean;
    minAuthLevel?: AuthLevel;
  }
) {
  const WrappedComponent = React.forwardRef((props: any, ref: React.Ref<any>) => (
    <UnifiedPermissionGuard
      platform={platform}
      permissions={permissions}
      actionName={options?.actionName}
      showPermissionDetails={options?.showPermissionDetails}
      minAuthLevel={options?.minAuthLevel}
    >
      <Component {...props} {...(ref && { ref })} />
    </UnifiedPermissionGuard>
  ));
  WrappedComponent.displayName = `withUnifiedPermissions(${Component.displayName || Component.name})`;
  return WrappedComponent as unknown as React.ComponentType<P>;
}

// ============================================================================
// UNIFIED PERMISSION HOOK
// ============================================================================

export function useUnifiedPermissionGuard(platform: Platform, permissions: string[]) {
  const auth = useAuthHook(platform);

  return {
    hasPermission: auth.hasAnyPermission(permissions),
    userPermissions: auth.permissions,
    canAccess: (permission: string) => auth.hasAnyPermission([permission])
  };
}