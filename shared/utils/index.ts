/**
 * CONSOLIDATED UTILS INDEX
 * Unified export of all utility functions to replace monolithic admin utils.ts
 * This replaces the 592-line admin monolith with clean, modular utilities
 */

// ============================================================================
// FORMATTING UTILITIES - Import then re-export for utils object
// ============================================================================
import {
  cur,
  fmtCurrency,
  formatBytes,
  // Currency formatting
  formatCurrency,
  formatEPS,
  formatFileSize,
  formatLargeNumber,
  formatPrice,
  prc
} from './formatting/currency'

export {
  cur, fmtCurrency, formatBytes, formatCurrency, formatEPS, formatFileSize, formatLargeNumber, formatPrice,
  prc
}

import {
  calculateDaysRemaining,
  calculateHoursRemaining,
  dt,
  fmtDate,
  fmtRelativeTime,
  formatAnnouncementDate,
  formatCompactDate,
  formatCountdown,
  // Date formatting
  formatDate,
  formatDateTime,
  formatQuarterDate,
  formatRelativeTime,
  formatTimeRemaining,
  getQuarterLabel,
  getRelativeTime,
  isFutureDate
} from './formatting/date'

export {
  calculateDaysRemaining,
  calculateHoursRemaining, dt,
  fmtDate, fmtRelativeTime, formatAnnouncementDate,
  formatCompactDate, formatCountdown, formatDate, formatDateTime,
  formatQuarterDate, formatRelativeTime, formatTimeRemaining,
  getQuarterLabel, getRelativeTime, isFutureDate
}

import {
  camelCase,
  capitalize,
  clamp,
  copyToClipboard as copyToClipboardDisplay,
  epsGr,
  formatEPSGrowth,
  // Display formatting
  formatPercentage,
  generateId,
  generateSimpleId,
  getEPSPerformanceColor,
  getGrowthIndicator,
  isBrowser as isBrowserDisplay,
  isEmpty,
  isMobile as isMobileDisplay,
  isValidEmail,
  isValidPhone,
  kebabCase,
  parseJWT,
  pct,
  slugify,
  truncate,
  truncateText
} from './formatting/display'

export {
  camelCase, capitalize, clamp, copyToClipboardDisplay, epsGr, formatEPSGrowth, formatPercentage, generateId,
  generateSimpleId, getEPSPerformanceColor,
  getGrowthIndicator, isBrowserDisplay, isEmpty, isMobileDisplay, isValidEmail,
  isValidPhone, kebabCase, parseJWT, pct, slugify, truncate,
  truncateText
}

// ============================================================================
// CALCULATION UTILITIES
// ============================================================================
// Financial calculations moved to app-specific implementations:
// - apps/admin-frontend/lib/consolidated-utils.ts
// - apps/frontend/components/home/utils/financialCalculations.ts

// ============================================================================
// HELPER UTILITIES
// ============================================================================
import {
  AsyncQueue,
  batchAsync,
  cancellable,
  // Async utilities
  debounce,
  memoizeAsync,
  poll,
  promises,
  retry,
  sleep,
  throttle,
  timeout
} from './helpers/async'

export {
  AsyncQueue, batchAsync,
  cancellable, debounce, memoizeAsync, poll,
  promises, retry, sleep, throttle, timeout
}

import {
  // Object utilities
  deepClone,
  object,
  objectUtils
} from './helpers/objects'

export {
  deepClone, object, objectUtils
}

import {
  copyToClipboard,
  isBrowser,
  isMobile,
  // Browser utilities  
  storage
} from './helpers/browser'

export {
  array,
  // Array utilities
  arrayUtils
} from './helpers/arrays'

export {
  addEventListener, copyToClipboard, cssVariables, downloadFile, getBrowserInfo, getColorSchemePreference, getDeviceType, getScrollPosition, getViewportSize, isAndroid, isBrowser, isElementInViewport, isIOS, isMobile, onResize, readFromClipboard, sessionStorage, smoothScrollTo,
  // Browser utilities
  storage, watchColorScheme
} from './helpers/browser'

// ============================================================================
// CORE UTILITIES
// ============================================================================
export {
  analyticsLogger, apiLogger,
  authLogger, devInfo, devLog, devWarn,
  isDevEnvironment,
  isProdEnvironment,
  // Logging utilities
  Logger,
  logger, safeError, uiLogger, type LogEntry, type LogLevel, type SafeErrorResult
} from './core/logging'

// ============================================================================
// API CLIENT UTILITIES
// ============================================================================

// Import API client functions
import {
  APIError,
  createAdminApiClient,
  createApiClient as createApiClientBase,
  createFrontendApiClient,
  handlePaginatedRequest,
  handleSimpleRequest,
  isApiError,
  isApiResponse,
  isPaginatedResponse,
  retryRequest,
  UnifiedApiClient,
  type ApiError,
  type ApiResponse,
  type PaginatedResponse,
  type Platform,
  type RequestConfig
} from './api-client'

// Re-export API client utilities
export {
  APIError,
  createAdminApiClient,
  createFrontendApiClient,
  handlePaginatedRequest,
  handleSimpleRequest, isApiError,
  isApiResponse,
  isPaginatedResponse, retryRequest, UnifiedApiClient, type ApiError, type ApiResponse, type PaginatedResponse,
  type Platform, type RequestConfig
}

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
  createCacheablePermissions,
  generatePermissionCacheKey,
  getActiveTierGroups,
  getExpiredPermissions,
  // Expiry utilities
  getExpiringPermissions,
  getHighestTierGroup,
  getTierDisplayFromPermissions,
  hasAllUnifiedPermissions,
  hasAnyUnifiedPermission,
  hasMinimumTier,
  hasUnifiedPermission,
  // Tier utilities
  isActiveTierAssignment,
  // Cache utilities
  isPermissionCacheValid,
  // Legacy compatibility
  legacyTierToPermissionCheck,
  needsRenewalWarning,
  // Permission resolution
  resolveUnifiedPermissions,
  // Types
  type PermissionSource,
  type UserGroupMembership
} from './unified-permissions'

// Then export them
export {
  createCacheablePermissions, generatePermissionCacheKey, getActiveTierGroups, getExpiredPermissions,
  // Expiry utilities
  getExpiringPermissions, getHighestTierGroup, getTierDisplayFromPermissions, hasAllUnifiedPermissions, hasAnyUnifiedPermission, hasMinimumTier, hasUnifiedPermission,
  // Tier utilities
  isActiveTierAssignment,
  // Cache utilities
  isPermissionCacheValid,
  // Legacy compatibility
  legacyTierToPermissionCheck, needsRenewalWarning,
  // Permission resolution
  resolveUnifiedPermissions,
  // Types
  type PermissionSource,
  type UserGroupMembership
}

// ============================================================================
// TAILWIND CLASS UTILITY (From both apps)
// ============================================================================

/**
 * Utility function to merge Tailwind CSS classes
 * Direct imports since dependencies are now available
 */
import clsx from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: any[]) {
  return twMerge(clsx(inputs));
}

/**
 * Synchronous version for environments where clsx/tailwind-merge are available
 * Use this when you know the dependencies are installed
 */
export function cnSync(...inputs: any[]) {
  // This will need to be implemented by the consuming application
  // if they want synchronous class merging
  return inputs
    .filter(Boolean)
    .map(input => typeof input === 'string' ? input : '')
    .join(' ')
}

// ============================================================================
// CONVENIENCE RE-EXPORTS FOR LEGACY COMPATIBILITY
// ============================================================================

// Legacy aliases from admin utils.ts
export { formatCurrency as formatCurrencyAdmin } from './formatting/currency'
export { formatDate as formatDateAdmin } from './formatting/date'
export { formatPercentage as formatPercentageAdmin } from './formatting/display'

// Legacy aliases from frontend utils
export { formatCurrency as cur_legacy, formatPrice as prc_legacy } from './formatting/currency'
export { formatDate as dt_legacy } from './formatting/date'
export { formatEPSGrowth as epsGr_legacy, formatPercentage as pct_legacy } from './formatting/display'

// Re-export commonly used utilities at top level for convenience
export const utils = {
  // Most frequently used formatters
  formatCurrency,
  formatDate,
  formatPercentage,
  formatPrice,
  formatEPSGrowth,

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