/**
 * SHARED PERMISSION COMPONENTS INDEX
 * 
 * Consolidated exports for all shared permission-related components.
 * This replaces the need for separate permission components between 
 * admin-frontend and frontend applications with unified components.
 */

import React from 'react';

// Main unified permission expiry indicator component
import UnifiedPermissionExpiryIndicator from './UnifiedPermissionExpiryIndicator';
export { UnifiedPermissionExpiryIndicator };

// Types and interfaces
export type { 
  Platform, 
  PermissionInfo,
  PermissionExpiryData,
  UnifiedPermissionExpiryIndicatorProps
} from './UnifiedPermissionExpiryIndicator';

// Import types for use in components
import type { Platform } from './UnifiedPermissionExpiryIndicator';

// Registration utilities for platform integration
export {
  registerPermissionHook,
  registerUIComponents
} from './UnifiedPermissionExpiryIndicator';

// ============================================================================
// MIGRATION HELPERS
// ============================================================================

/**
 * Legacy component mappings for easier migration
 * These can be used during the transition period
 */

// For admin-frontend migration
export const AdminPermissionExpiryIndicator = UnifiedPermissionExpiryIndicator;
export const UserPermissionHealthCard = ({ permissions, userId, className }: { 
  permissions: string[]; 
  userId: string; 
  className?: string; 
}) => (
  <UnifiedPermissionExpiryIndicator 
    platform="admin" 
    permissions={permissions}
    variant="card"
    showHealth={true}
    showDetails={true}
    showActions={true}
    className={className}
  />
);

export const AdminPermissionBanner = ({ permissions, className }: { 
  permissions: string[]; 
  className?: string; 
}) => (
  <UnifiedPermissionExpiryIndicator 
    platform="admin" 
    permissions={permissions}
    variant="banner"
    showDetails={true}
    className={className}
  />
);

export const AdminPermissionDashboard = ({ permissions, className }: { 
  permissions: string[]; 
  className?: string; 
}) => (
  <UnifiedPermissionExpiryIndicator 
    platform="admin" 
    permissions={permissions}
    variant="dashboard"
    showHealth={true}
    className={className}
  />
);

// For frontend migration  
export const PermissionExpiryIndicator = UnifiedPermissionExpiryIndicator;

// ============================================================================
// CONVENIENCE COMPONENTS FOR SPECIFIC USE CASES
// ============================================================================

export const PermissionBadge = ({ 
  platform, 
  permissions, 
  className 
}: { 
  platform: Platform; 
  permissions: string | string[]; 
  className?: string; 
}) => (
  <UnifiedPermissionExpiryIndicator 
    platform={platform} 
    permissions={permissions}
    variant="badge"
    className={className}
  />
);

export const PermissionStatusCard = ({ 
  platform, 
  permissions, 
  showActions = false,
  className 
}: { 
  platform: Platform; 
  permissions: string | string[];
  showActions?: boolean;
  className?: string; 
}) => (
  <UnifiedPermissionExpiryIndicator 
    platform={platform} 
    permissions={permissions}
    variant="card"
    showHealth={true}
    showDetails={true}
    showActions={showActions}
    className={className}
  />
);

export const PermissionAlert = ({ 
  platform, 
  permissions, 
  showRefresh = true,
  className 
}: { 
  platform: Platform; 
  permissions: string | string[];
  showRefresh?: boolean;
  className?: string; 
}) => (
  <UnifiedPermissionExpiryIndicator 
    platform={platform} 
    permissions={permissions}
    variant="full"
    showDetails={true}
    className={className}
  />
);

// ============================================================================
// USAGE EXAMPLES AND DOCUMENTATION
// ============================================================================

/**
 * USAGE EXAMPLES:
 * 
 * ## Admin Frontend
 * ```tsx
 * import { UnifiedPermissionExpiryIndicator, AdminPermissionBanner } from '@shared/components/permissions';
 * 
 * // Basic usage
 * <UnifiedPermissionExpiryIndicator 
 *   platform="admin" 
 *   permissions={userPermissions}
 *   variant="card"
 *   showHealth={true}
 *   showDetails={true}
 *   showActions={true}
 * />
 * 
 * // Legacy compatibility
 * <AdminPermissionBanner permissions={userPermissions} />
 * ```
 * 
 * ## Frontend
 * ```tsx
 * import { UnifiedPermissionExpiryIndicator, PermissionAlert } from '@shared/components/permissions';
 * 
 * // Single permission check
 * <UnifiedPermissionExpiryIndicator 
 *   platform="frontend" 
 *   permissions="epsx:analytics:premium"
 *   variant="badge"
 *   showCountdown={true}
 * />
 * 
 * // Multiple permissions with alert
 * <PermissionAlert 
 *   platform="frontend" 
 *   permissions={userPermissions}
 *   showRefresh={true}
 * />
 * ```
 * 
 * ## Platform Setup
 * ```tsx
 * // In admin-frontend app initialization
 * import { registerPermissionHook } from '@shared/components/permissions';
 * import { useConsolidatedPermissions } from '@/hooks/useConsolidatedPermissions';
 * 
 * registerPermissionHook('admin', () => ({
 *   hasPermission: (permission: string) => hasPermission(permission),
 *   getPermissionExpiry: (permission: string) => getExpiryInfo(permission),
 *   refreshPermissions: () => refetch(),
 *   loading: isLoading
 * }));
 * 
 * // In frontend app initialization  
 * import { registerPermissionHook } from '@shared/components/permissions';
 * import { useGranularPermissions } from '@/hooks/useGranularPermissions';
 * 
 * registerPermissionHook('frontend', () => {
 *   const { hasPermission, getPermissionExpiry, refreshPermissions, loading } = useGranularPermissions();
 *   return { hasPermission, getPermissionExpiry, refreshPermissions, loading };
 * });
 * ```
 */