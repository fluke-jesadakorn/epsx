// ============================================================================
// CONSOLIDATED SERVER ACTIONS - Single Entry Point for All Server Operations
// ============================================================================

// Primary auth actions (most commonly used)
export {
  login,
  logout,
  logoutWithRevalidation,
  getCurrentUser,
  register,
  checkFeatureAccess,
  getUserFeatures,
  requestPasswordReset,
  resetPassword,
  refreshToken,
  updateProfile,
  changePassword
} from './actions/auth';

// Admin auth actions
export {
  adminLogin,
  checkAdminPermission,
  getAdminSession
} from './actions/auth';

// User & admin management
export {
  getAdminUsers,
  getUserStats,
  updateUserRole,
  updateUserPackageTier,
  getStockRankingPackages,
  assignStockRankingPackage,
  createUser,
  updateUser,
  deleteUser
} from './actions/admin';

// Payment operations
export {
  createPayment,
  validatePayment,
  getPaymentStatus,
  getTransactionHistory,
  getPlanDetails,
  initQRPayment
} from './actions/payments';

// Permission operations
export {
  getUserPermissions,
  checkPermission,
  checkRankingAccess,
  getPermissionProfiles,
  assignPermissionProfile,
  revokePermissionProfile,
  getPermissionMatrix,
  getPaginatedFeatureAccess
} from './actions/permissions';

// IAM operations
export {
  getIAMUsers,
  getIAMRoles,
  getIAMPolicies,
  evaluatePermission
} from './actions/iam';

// Analytics & monitoring
export {
  getAnalyticsData,
  getUserAnalytics,
  getSystemMetrics,
  getRevenueAnalytics,
  getRealtimeMetrics,
  getDashboardData,
  trackError,
  trackPerformance,
  getHealthCheck
} from './actions/analytics';

// Module system
export {
  getModules,
  getUserModuleAssignments,
  assignModulesToUser,
  revokeModuleAccess,
  getModuleUsageAnalytics,
  createApiKey,
  listApiKeys,
  revokeApiKey
} from './actions/modules';

// Settings
export {
  getSystemConfig,
  updateSettings,
  getUserSettings,
  updateUserSettings,
  getFeatureFlags,
  getSettingsByCategory,
  getEnvironmentConfig
} from './actions/settings';

// Core utilities (for custom actions)
export {
  withServerAction,
  createServerAction,
  createAuthenticatedAction
} from './core/action-wrapper';

export {
  makeServerRequest
} from './core/request';

// Essential types
export type {
  ActionResult
} from './core/action-wrapper';

export type {
  PaymentStatus,
  PaymentTransaction
} from './actions/payments';

export type {
  UserPermission,
  PermissionProfile
} from './actions/permissions';

// ============================================================================
// IMPORT GUIDANCE - Use specific imports to minimize bundle size:
// 
// Auth:            import { login, logout, getCurrentUser } from '@epsx/server-actions';
// Payments:        import { createPayment, getPaymentStatus } from '@epsx/server-actions';
// Permissions:     import { checkPermission, getUserPermissions } from '@epsx/server-actions';
// Admin:           import { getAdminUsers, getUserStats } from '@epsx/server-actions';
// Analytics:       import { getDashboardData, trackError } from '@epsx/server-actions';
// 
// Avoid: import * from '@epsx/server-actions' (pulls entire package)
// ============================================================================