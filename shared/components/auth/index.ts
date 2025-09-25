/**
 * SHARED AUTH COMPONENTS INDEX
 * 
 * Consolidated exports for all shared authentication and permission components.
 * This replaces the need for separate AdminPermissionGuard and PlatformPermissionGuard
 * with unified components that work across both admin-frontend and frontend applications.
 */

// Main unified permission guard component
export { default as UnifiedPermissionGuard } from './UnifiedPermissionGuard';

// Unified progressive auth gate component
export { default as UnifiedProgressiveAuthGate } from './UnifiedProgressiveAuthGate';

// Types and interfaces
export type { 
  Platform, 
  AuthLevel, 
  UnifiedPermissionGuardProps 
} from './UnifiedPermissionGuard';

export type {
  UnifiedProgressiveAuthGateProps
} from './UnifiedProgressiveAuthGate';

// Convenience components - Platform agnostic
export {
  RequirePermission,
  RequireRole,
  RequireTier,
  RequireAccess
} from './UnifiedPermissionGuard';

// Admin-specific convenience components  
export {
  RequireAdminPermission,
  RequireUserManagement,
  RequireSystemManagement,
  RequireAnalyticsAccess,
  RequirePlatformManagement,
  RequireSecurityAccess
} from './UnifiedPermissionGuard';

// Progressive auth gate convenience components
export {
  RequireSignIn,
  RequireProgressiveAuth,
  RequireFullAuth
} from './UnifiedProgressiveAuthGate';

// Higher-order component and hooks
export {
  withUnifiedPermissions,
  useUnifiedPermissionGuard
} from './UnifiedPermissionGuard';

export {
  withProgressiveAuth,
  useProgressiveAuthStatus
} from './UnifiedProgressiveAuthGate';

// Auth adapter for platform integration
export {
  registerAuthHook,
  getAuthHook,
  hasAuthHook,
  clearAuthHooks,
  createAdminAuthAdapter,
  createFrontendAuthAdapter,
  debugAuthHooks,
  createMockAuthHook
} from './UnifiedAuthAdapter';

export type {
  UnifiedAuthInterface,
  AuthContextValue
} from './UnifiedAuthAdapter';

// ============================================================================
// MIGRATION HELPERS
// ============================================================================

/**
 * Legacy component mappings for easier migration
 * These can be used during the transition period
 */

// For admin-frontend migration
export const AdminPermissionGuard = UnifiedPermissionGuard;
export const RequireAdminAccess = RequireAdminPermission;

// For frontend migration  
export const PlatformPermissionGuard = UnifiedPermissionGuard;
export const RequirePlatformAccess = RequirePermission;

// ============================================================================
// USAGE EXAMPLES AND DOCUMENTATION
// ============================================================================

/**
 * USAGE EXAMPLES:
 * 
 * ## Admin Frontend
 * ```tsx
 * import { UnifiedPermissionGuard, RequireUserManagement } from '@shared/components/auth';
 * 
 * // Basic usage
 * <UnifiedPermissionGuard 
 *   platform="admin" 
 *   permissions={['admin:users:manage']}
 *   showPermissionDetails={true}
 * >
 *   <AdminUserPanel />
 * </UnifiedPermissionGuard>
 * 
 * // Convenience component
 * <RequireUserManagement>
 *   <UserManagementDashboard />
 * </RequireUserManagement>
 * ```
 * 
 * ## Frontend
 * ```tsx
 * import { UnifiedPermissionGuard, RequireTier } from '@shared/components/auth';
 * 
 * // Basic usage
 * <UnifiedPermissionGuard 
 *   platform="frontend" 
 *   permissions={['epsx:analytics:premium']}
 *   showUpgradePrompt={true}
 * >
 *   <PremiumAnalytics />
 * </UnifiedPermissionGuard>
 * 
 * // Tier-based access
 * <RequireTier platform="frontend" tier="PRO">
 *   <ProFeatures />
 * </RequireTier>
 * ```
 * 
 * ## Platform Setup
 * ```tsx
 * // In admin-frontend app initialization
 * import { registerAuthHook, createAdminAuthAdapter } from '@shared/components/auth';
 * import { useAdminProgressiveAuth } from '@/hooks/useAdminProgressiveAuth';
 * 
 * registerAuthHook('admin', createAdminAuthAdapter(useAdminProgressiveAuth));
 * 
 * // In frontend app initialization  
 * import { registerAuthHook, createFrontendAuthAdapter } from '@shared/components/auth';
 * import { useAuth } from '@/lib/auth';
 * 
 * registerAuthHook('frontend', createFrontendAuthAdapter(useAuth));
 * ```
 */