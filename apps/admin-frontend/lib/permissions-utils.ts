/**
 * Permission Utility Functions - Updated to use shared system
 * Re-export platform and tier utilities from shared permission system
 */

export {
  derivePackageTierFromPermissions,
  deriveAccessiblePlatformsFromPermissions,
  derivePrimaryPlatformFromPermissions,
  getPackageFromPermissions,
  hasAdminAccess,
  hasEnterpriseAccess
} from '@/shared/permissions/utils'