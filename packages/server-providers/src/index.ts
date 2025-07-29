// Legacy exports (for backward compatibility)
export { getPermissionData } from './ServerPermissionProvider';
export { PermissionProvider, usePermissionContext } from './PermissionProvider';
export { ClientPermissionWrapper } from './ClientPermissionWrapper';

// Enhanced server-providers with error boundaries and multi-state support
export {
  EnhancedPermissionProvider,
  useEnhancedPermissionContext,
  usePermissions,
  usePaymentStatus,
  useFeatureAccess,
  ServerStateLoading,
  ServerStateError
} from './EnhancedPermissionProvider';

export {
  getEnhancedPermissionData,
  getBasicPermissionData,
  getComprehensivePermissionData,
  isValidServerData,
  getCacheStats,
  type EnhancedServerData
} from './EnhancedServerPermissionProvider';

export {
  EnhancedClientWrapper,
  PermissionGuard,
  PaymentTierGuard
} from './EnhancedClientWrapper';

// Type exports
export type { ServerState, ServerStateAction } from './EnhancedPermissionProvider';