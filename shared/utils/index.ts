/**
 * CONSOLIDATED UTILS INDEX
 * Unified export of all utility functions to replace monolithic admin utils.ts
 * This replaces the 592-line admin monolith with clean, modular utilities
 */

// ============================================================================
// FORMATTING UTILITIES - Import then re-export for utils object
// ============================================================================
import {
  // Currency formatting
  formatCurrency,
  fmtCurrency,
  cur,
  formatEPS,
  formatPrice,
  prc,
  formatLargeNumber,
  formatFileSize,
  formatBytes
} from './formatting/currency'

export {
  formatCurrency,
  fmtCurrency,
  cur,
  formatEPS,
  formatPrice,
  prc,
  formatLargeNumber,
  formatFileSize,
  formatBytes
}

import {
  // Date formatting
  formatDate,
  dt,
  fmtDate,
  formatRelativeTime,
  fmtRelativeTime,
  formatQuarterDate,
  formatAnnouncementDate,
  formatCompactDate,
  getRelativeTime,
  formatTimeRemaining,
  getQuarterLabel,
  isFutureDate,
  formatCountdown,
  calculateDaysRemaining,
  calculateHoursRemaining
} from './formatting/date'

export {
  formatDate,
  dt,
  fmtDate,
  formatRelativeTime,
  fmtRelativeTime,
  formatQuarterDate,
  formatAnnouncementDate,
  formatCompactDate,
  getRelativeTime,
  formatTimeRemaining,
  getQuarterLabel,
  isFutureDate,
  formatCountdown,
  calculateDaysRemaining,
  calculateHoursRemaining
}

import {
  // Display formatting
  formatPercentage,
  pct,
  formatEPSGrowth,
  epsGr,
  capitalize,
  truncate,
  truncateText,
  slugify,
  kebabCase,
  camelCase,
  isValidEmail,
  isValidPhone,
  getEPSPerformanceColor,
  getGrowthIndicator,
  parseJWT,
  isEmpty,
  generateId,
  generateSimpleId,
  copyToClipboard as copyToClipboardDisplay,
  isBrowser as isBrowserDisplay,
  isMobile as isMobileDisplay,
  clamp
} from './formatting/display'

export {
  formatPercentage,
  pct,
  formatEPSGrowth,
  epsGr,
  capitalize,
  truncate,
  truncateText,
  slugify,
  kebabCase,
  camelCase,
  isValidEmail,
  isValidPhone,
  getEPSPerformanceColor,
  getGrowthIndicator,
  parseJWT,
  isEmpty,
  generateId,
  generateSimpleId,
  copyToClipboardDisplay,
  isBrowserDisplay,
  isMobileDisplay,
  clamp
}

// ============================================================================
// CALCULATION UTILITIES
// ============================================================================
import {
  // Financial calculations
  calculateEPSGrowth,
  calculateEPSSurprise,
  calculateEstimatedGrowth,
  calculatePERatio,
  formatPERatio,
  calculatePriceGrowth,
  calculatePriceTarget,
  calculatePercentageChange,
  calculateCurrentRatio,
  calculateQuickRatio,
  calculateGrossMargin,
  calculateOperatingMargin,
  calculateNetMargin,
  calculateBookValuePerShare,
  calculatePriceToBook,
  calculateDividendYield,
  calculatePayoutRatio,
  calculateEnterpriseValue,
  calculateEVToEBITDA,
  calculateAssetTurnover,
  calculateInventoryTurnover,
  calculateReceivablesTurnover,
  calculateWorkingCapital,
  calculateMarketCap,
  calculateEarningsYield,
  calculateFreeCashFlow,
  calculateFCFYield,
  calculateBeta,
  calculateSharpeRatio
} from './calculations/financial'

export {
  calculateEPSGrowth,
  calculateEPSSurprise,
  calculateEstimatedGrowth,
  calculatePERatio,
  formatPERatio,
  calculatePriceGrowth,
  calculatePriceTarget,
  calculatePercentageChange,
  calculateCurrentRatio,
  calculateQuickRatio,
  calculateGrossMargin,
  calculateOperatingMargin,
  calculateNetMargin,
  calculateBookValuePerShare,
  calculatePriceToBook,
  calculateDividendYield,
  calculatePayoutRatio,
  calculateEnterpriseValue,
  calculateEVToEBITDA,
  calculateAssetTurnover,
  calculateInventoryTurnover,
  calculateReceivablesTurnover,
  calculateWorkingCapital,
  calculateMarketCap,
  calculateEarningsYield,
  calculateFreeCashFlow,
  calculateFCFYield,
  calculateBeta,
  calculateSharpeRatio
}

// ============================================================================
// HELPER UTILITIES
// ============================================================================
import {
  // Async utilities
  debounce,
  throttle,
  sleep,
  retry,
  timeout,
  batchAsync,
  cancellable,
  memoizeAsync,
  AsyncQueue,
  poll,
  promises
} from './helpers/async'

export {
  debounce,
  throttle,
  sleep,
  retry,
  timeout,
  batchAsync,
  cancellable,
  memoizeAsync,
  AsyncQueue,
  poll,
  promises
}

import {
  // Object utilities
  deepClone,
  objectUtils,
  object
} from './helpers/objects'

export {
  deepClone,
  objectUtils,
  object
}

import {
  // Browser utilities  
  storage,
  isBrowser,
  isMobile,
  copyToClipboard
} from './helpers/browser'

export {
  // Array utilities
  arrayUtils,
  array
} from './helpers/arrays'

export {
  // Browser utilities
  storage,
  sessionStorage,
  isBrowser,
  isMobile,
  isIOS,
  isAndroid,
  getDeviceType,
  getBrowserInfo,
  copyToClipboard,
  readFromClipboard,
  downloadFile,
  getViewportSize,
  getScrollPosition,
  smoothScrollTo,
  isElementInViewport,
  addEventListener,
  onResize,
  cssVariables,
  getColorSchemePreference,
  watchColorScheme
} from './helpers/browser'

// ============================================================================
// CORE UTILITIES
// ============================================================================
export {
  // Logging utilities
  Logger,
  logger,
  apiLogger,
  authLogger,
  analyticsLogger,
  uiLogger,
  safeError,
  devLog,
  devInfo,
  devWarn,
  isDevEnvironment,
  isProdEnvironment,
  type LogLevel,
  type LogEntry,
  type SafeErrorResult
} from './core/logging'

// ============================================================================
// API CLIENT UTILITIES
// ============================================================================

// Import API client functions
import { 
  UnifiedApiClient,
  APIError,
  createAdminApiClient,
  createFrontendApiClient,
  createApiClient as createApiClientBase,
  handlePaginatedRequest,
  handleSimpleRequest,
  retryRequest,
  isApiError,
  isApiResponse,
  isPaginatedResponse,
  type ApiResponse,
  type ApiError,
  type RequestConfig,
  type PaginatedResponse,
  type Platform
} from './api-client';

// Re-export API client utilities
export {
  UnifiedApiClient,
  APIError,
  createAdminApiClient,
  createFrontendApiClient,
  handlePaginatedRequest,
  handleSimpleRequest,
  retryRequest,
  isApiError,
  isApiResponse,
  isPaginatedResponse,
  type ApiResponse,
  type ApiError,
  type RequestConfig,
  type PaginatedResponse,
  type Platform
};

// Legacy compatibility exports for easier migration
export const createAdminClient = createAdminApiClient;
export const createServerAdminClient = (baseURL?: string, token?: string) => 
  createAdminApiClient({ baseURL, token, serverSide: true });
export const apiClient = createFrontendApiClient();
export const createClient = createFrontendApiClient;
export const createApiClient = createApiClientBase;

// ============================================================================
// UNIFIED PERMISSION UTILITIES
// ============================================================================

// Import unified permission functions first
import {
  // Permission resolution
  resolveUnifiedPermissions,
  hasUnifiedPermission,
  hasAnyUnifiedPermission,
  hasAllUnifiedPermissions,
  
  // Tier utilities
  isActiveTierAssignment,
  getHighestTierGroup,
  getActiveTierGroups,
  hasMinimumTier,
  
  // Expiry utilities
  getExpiringPermissions,
  getExpiredPermissions,
  needsRenewalWarning,
  
  // Legacy compatibility
  legacyTierToPermissionCheck,
  getTierDisplayFromPermissions,
  
  // Cache utilities
  isPermissionCacheValid,
  generatePermissionCacheKey,
  createCacheablePermissions,
  
  // Types
  type PermissionSource,
  type UserGroupMembership
} from './unified-permissions';

// Then export them
export {
  // Permission resolution
  resolveUnifiedPermissions,
  hasUnifiedPermission,
  hasAnyUnifiedPermission,
  hasAllUnifiedPermissions,
  
  // Tier utilities
  isActiveTierAssignment,
  getHighestTierGroup,
  getActiveTierGroups,
  hasMinimumTier,
  
  // Expiry utilities
  getExpiringPermissions,
  getExpiredPermissions,
  needsRenewalWarning,
  
  // Legacy compatibility
  legacyTierToPermissionCheck,
  getTierDisplayFromPermissions,
  
  // Cache utilities
  isPermissionCacheValid,
  generatePermissionCacheKey,
  createCacheablePermissions,
  
  // Types
  type PermissionSource,
  type UserGroupMembership
};

// ============================================================================
// TAILWIND CLASS UTILITY (From both apps)
// ============================================================================
import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

/**
 * Utility function to merge Tailwind CSS classes
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// ============================================================================
// CONVENIENCE RE-EXPORTS FOR LEGACY COMPATIBILITY
// ============================================================================

// Legacy aliases from admin utils.ts
export { formatDate as formatDateAdmin } from './formatting/date'
export { formatCurrency as formatCurrencyAdmin } from './formatting/currency'
export { formatPercentage as formatPercentageAdmin } from './formatting/display'

// Legacy aliases from frontend utils
export { formatCurrency as cur_legacy } from './formatting/currency'
export { formatDate as dt_legacy } from './formatting/date'
export { formatPercentage as pct_legacy } from './formatting/display'
export { formatPrice as prc_legacy } from './formatting/currency'
export { formatEPSGrowth as epsGr_legacy } from './formatting/display'

// Re-export commonly used utilities at top level for convenience
export const utils = {
  // Most frequently used formatters
  formatCurrency,
  formatDate,
  formatPercentage,
  formatPrice,
  formatEPSGrowth,
  
  // Most frequently used calculations
  calculateEPSGrowth,
  calculatePERatio,
  calculatePercentageChange,
  
  // Most frequently used helpers
  debounce,
  throttle,
  deepClone,
  isEmpty,
  generateId,
  cn,
  
  // Most frequently used browser utilities
  isBrowser,
  isMobile,
  copyToClipboard,
  storage,
  
  // Most frequently used permission utilities
  resolveUnifiedPermissions,
  hasUnifiedPermission,
  isActiveTierAssignment,
  getHighestTierGroup,
  legacyTierToPermissionCheck,
  
  // Most frequently used API client utilities
  createAdminApiClient,
  createFrontendApiClient,
  handlePaginatedRequest,
  handleSimpleRequest,
  isApiError
}

// ============================================================================
// QUARTER PRIORITY HELPER (From admin monolith)
// ============================================================================

/**
 * Get quarter priority based on growth and timing
 */
export function getQuarterPriority(epsGrowth: number, daysUntilAnnouncement: number): 'high' | 'medium' | 'low' {
  if (Math.abs(epsGrowth) > 20 || daysUntilAnnouncement <= 7) {
    return 'high'
  } else if (Math.abs(epsGrowth) > 10 || daysUntilAnnouncement <= 14) {
    return 'medium'
  }
  return 'low'
}

// ============================================================================
// DEFAULT EXPORT FOR CONVENIENCE
// ============================================================================

export default utils