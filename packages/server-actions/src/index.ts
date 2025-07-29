// Action modules - use specific imports to avoid conflicts
export {
  getAdminUsers,
  getUserStats,
  updateUserRole,
  updateUserPackageTier,
  getStockRankingPackages,
  assignStockRankingPackage
} from './actions/admin';

export {
  getIAMUsers,
  getIAMRoles,
  getIAMPolicies,
  getCustomPermissions,
  grantCustomPermission,
  revokeCustomPermission,
  getUserEffectivePermissions,
  evaluatePermission
} from './actions/iam';

export {
  getAnalyticsData,
  getUserAnalytics,
  getSystemMetrics,
  getRevenueAnalytics,
  getRealtimeMetrics,
  generateReports,
  getScheduledReports,
  scheduleReport,
  getDashboardData,
  trackError,
  trackPerformance,
  getHealthCheck
} from './actions/analytics';

export {
  login,
  logout,
  getCurrentUser,
  refreshToken,
  adminLogin,
  checkAdminPermission,
  getAdminSession,
  updateProfile,
  changePassword,
  register,
  checkFeatureAccess,
  getUserFeatures,
  requestPasswordReset,
  resetPassword
} from './actions/auth';

export {
  getUserPermissions,
  checkPermission,
  checkRankingAccess,
  getPermissionProfiles,
  assignPermissionProfile,
  revokePermissionProfile,
  getPermissionMatrix,
  getPaginatedFeatureAccess,
  type UserPermission,
  type PermissionProfile
} from './actions/permissions';

export {
  createPayment,
  validatePayment,
  getPaymentStatus,
  getTransactionHistory,
  getPlanDetails,
  initQRPayment,
  type PaymentStatus,
  type PaymentTransaction
} from './actions/payments';

// Enhanced server actions with improved error handling and validation
export {
  enhancedLogin,
  enhancedLogout,
  enhancedGetCurrentUser,
  enhancedRegister,
  enhancedUpdateProfile,
  enhancedChangePassword,
  enhancedResetPassword,
  enhancedRefreshToken,
  enhancedCheckFeatureAccess,
  enhancedGetUserFeatures,
  enhancedAdminLogin,
  enhancedCheckAdminPermission,
  type EnhancedLoginResult,
  type EnhancedRegisterResult,
  type EnhancedProfileResult
} from './actions/enhanced-auth';

export {
  enhancedCreatePayment,
  enhancedGetPaymentStatus,
  enhancedValidatePayment,
  enhancedGetPaymentPlans,
  enhancedGetPaymentPlan,
  enhancedGetUserSubscription,
  enhancedCancelSubscription,
  enhancedUpdateSubscription,
  enhancedGetPaymentHistory,
  enhancedDownloadInvoice,
  enhancedApplyPromoCode,
  enhancedCreateLegacyPayment,
  enhancedInitQRPayment,
  type EnhancedCreatePaymentResult,
  type EnhancedPaymentStatusResult,
  type EnhancedPaymentPlansResult,
  type EnhancedUserSubscriptionResult
} from './actions/enhanced-payments';

// Core utilities for creating enhanced server actions
export {
  withServerAction,
  createServerAction,
  createAuthenticatedAction,
  CommonSchemas,
  type ActionResult,
  type ActionContext,
  type ActionOptions
} from './core/action-wrapper';

// Enhanced request utilities
export {
  makeServerRequest,
  serverGet,
  serverPost,
  serverPut,
  serverDelete,
  serverPatch,
  type ServerRequestOptions
} from './core/enhanced-request';

// Settings management
export {
  getSystemConfig,
  updateSettings,
  getSettingsByCategory,
  getUserSettings,
  updateUserSettings,
  getFeatureFlags,
  updateFeatureFlag,
  getConfigTemplates,
  applyConfigTemplate,
  getEnvironmentConfig,
  updateEnvironmentConfig
} from './actions/settings';