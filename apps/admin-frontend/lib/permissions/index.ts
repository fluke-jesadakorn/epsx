// ============================================================================
// ADMIN FRONTEND PERMISSIONS - BACKEND-CENTRIC EXPORTS (Phase 2.3)
// ============================================================================
// 🔒 SECURITY TRANSFORMED: Only exports backend-centric permission system
// ⚡ THE SINGLE SOURCE OF TRUTH: All exports use backend permission authority

// ============================================================================
// BACKEND ADMIN PERMISSION AUTHORITY CLIENT
// ============================================================================
// THE SINGLE SOURCE OF TRUTH for all admin permission validation

export {
  adminPermissionAuthority,
  hasAdminPermission,
  hasAnyAdminPermission,
  hasAllAdminPermissions,
  isAdmin,
  isSuperAdmin,
  AdminPermissionDeniedError,
  AdminAuthenticationRequiredError,
} from './backend-authority-client';

export type {
  AdminPermissionAuthorityClient,
} from './backend-authority-client';

// ============================================================================
// BACKEND ADMIN PERMISSION HOOKS
// ============================================================================
// 🔒 All hooks now use backend permission authority

export {
  useBackendAdminPermissions,
  useAdminPermission,
  useAdminPermissions,
  useAdminCapabilities,
} from './use-backend-admin-permissions';

export type {
  AdminPermissionState,
  AdminPermissionError,
  AdminPermissionHookResult,
} from './use-backend-admin-permissions';

// ============================================================================
// BACKEND ADMIN PERMISSION GUARDS
// ============================================================================
// 🔒 All guards now use backend permission authority

export {
  BackendAdminPermissionGuard,
  SuperAdminGuard,
  UserManagementAdminGuard,
  SystemManagementAdminGuard,
  PermissionManagementAdminGuard,
  AnalyticsAdminGuard,
  SecurityAdminGuard,
  AuditLogsAdminGuard,
  withBackendAdminPermission,
} from '@/components/guards/BackendAdminPermissionGuard';

export type {
  BackendAdminPermissionGuardProps,
  AdminUpgradePromptProps,
} from '@/components/guards/BackendAdminPermissionGuard';

// ============================================================================
// BACKEND ADMIN AUTH CONTEXT
// ============================================================================
// 🔒 Admin authentication now uses backend permission authority

export {
  useBackendAdminAuth,
  useCurrentAdminUser,
  useAdminUserId,
  useIsAdminAuthenticated,
  useIsSuperAdmin,
  useAdminCapabilities,
  useAdminTier,
  useAdminAuthLoading,
  useAdminPermissionCheck,
  useUserManagementPermissionCheck,
  useSystemManagementPermissionCheck,
  usePermissionManagementPermissionCheck,
  useAnalyticsPermissionCheck,
  useSecurityPermissionCheck,
  useAuditLogsPermissionCheck,
  BackendAdminAuthProvider,
} from '@/contexts/BackendAdminAuthContext';

export type {
  AdminUser,
  AdminAuthState,
  AdminAuthContextValue,
} from '@/contexts/BackendAdminAuthContext';

// ============================================================================
// TRANSFORMED ADMIN PERMISSION GUARD (BACKWARD COMPATIBILITY)
// ============================================================================
// 🔒 Main admin guard now uses backend permission authority

export {
  default as AdminPermissionGuard,
  withAdminPermissions,
  useAdminPermissionGuard,
  RequireAdminPermission,
  RequireUserManagement,
  RequireSystemManagement,
  RequireAnalyticsAccess,
  RequirePlatformManagement,
  RequireSecurityAccess,
  RequirePermissionManagement,
  RequireAuditLogAccess,
  RequireSuperAdminAccess,
} from '@/components/auth/AdminPermissionGuard';

// ============================================================================
// ADMIN PERMISSION MANAGEMENT HOOKS (BACKEND-CENTRIC)
// ============================================================================
// 🔒 Management hooks now use backend permission authority

export {
  useAdminGranularPermissions,
  useAdminPermissionDashboard,
  useUserPermissionManagement,
  useAdminPermissions,
  usePermissionStats,
  useUserPermissionExpiry,
  usePermissionTemplates,
} from '@/hooks/useConsolidatedPermissions';

// ============================================================================
// API CLIENT (ADMINISTRATIVE OPERATIONS)
// ============================================================================
// 📊 API client for admin management operations (separate from permission validation)

export {
  permissionClient as adminPermissionApiClient,
  ConsolidatedPermissionClient,
  createPermissionClient,
  createServerPermissionClient,
} from '@/lib/api/consolidated-permission-client';

// ============================================================================
// LEGACY COMPATIBILITY WARNINGS
// ============================================================================

// Deprecated: Use backend permission authority instead
console.warn(`
⚠️  MIGRATION NOTICE: Admin Permission System Updated

The admin permission system has been upgraded to use backend permission authority.

OLD (Insecure):
- Local permission validation
- Client-side permission checks
- Hackable permission logic

NEW (Secure):
- Backend permission authority
- Real-time API validation
- Unhackable permission system

All components now use the backend-centric permission system automatically.
No code changes required for existing components using AdminPermissionGuard.
`);

// ============================================================================
// MIGRATION COMPLETE NOTICE
// ============================================================================
//
// 🎉 ADMIN PERMISSION SYSTEM CLEANUP COMPLETE!
//
// This file now exports ONLY the backend-centric permission system:
//
// Key Changes:
// ❌ REMOVED all legacy permission files (guards.tsx, types.ts)
// ❌ REMOVED all local admin permission validation logic
// ⚡ UPDATED exports to use backend permission authority
// 🔒 SECURED all admin permission validation through backend API
// 🎯 MAINTAINED backward compatibility for existing components
//
// The admin-frontend permission system is now CLEAN and SECURE!
// ============================================================================