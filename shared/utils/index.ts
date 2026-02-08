/**
 * CONSOLIDATED UTILS INDEX
 * Unified export of all utility functions
 */

// ============================================================================
// FORMATTING UTILITIES
// ============================================================================
import {
  cur,
  fmtCurrency,
  formatBytes,
  formatCurrency,
  formatEPS,
  formatFileSize,
  formatLargeNumber,
  formatPrice,
  prc
} from './formatting/currency'

import {
  calculateDaysRemaining,
  calculateHoursRemaining,
  dt,
  fmtDate,
  fmtRelativeTime,
  formatAnnouncementDate,
  formatCompactDate,
  formatCountdown,
  formatDate,
  formatDateTime,
  formatQuarterDate,
  formatRelativeTime,
  formatTimeRemaining,
  getQuarterLabel,
  getRelativeTime,
  isFutureDate
} from './formatting/date'

import {
  camelCase,
  capitalize,
  clamp,
  copyToClipboard as copyToClipboardDisplay,
  epsGr,
  formatEPSGrowth,
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

// ============================================================================
// API CLIENT UTILITIES
// ============================================================================
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

// ============================================================================
// TAILWIND CLASS UTILITY
// ============================================================================
import clsx, { type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import { logger as appLogger } from './logger'

export {
  cur, fmtCurrency, formatBytes, formatCurrency, formatEPS, formatFileSize, formatLargeNumber, formatPrice,
  prc
}

export {
  calculateDaysRemaining,
  calculateHoursRemaining, dt,
  fmtDate, fmtRelativeTime, formatAnnouncementDate,
  formatCompactDate, formatCountdown, formatDate, formatDateTime,
  formatQuarterDate, formatRelativeTime, formatTimeRemaining,
  getQuarterLabel, getRelativeTime, isFutureDate
}

export {
  camelCase, capitalize, clamp, copyToClipboardDisplay, epsGr, formatEPSGrowth, formatPercentage, generateId,
  generateSimpleId, getEPSPerformanceColor,
  getGrowthIndicator, isBrowserDisplay, isEmpty, isMobileDisplay, isValidEmail,
  isValidPhone, kebabCase, parseJWT, pct, slugify, truncate,
  truncateText
}

// ============================================================================
// BROWSER UTILITIES
// ============================================================================
export {
  addEventListener, copyToClipboard, cssVariables, downloadFile, getBrowserInfo,
  getColorSchemePreference, getDeviceType, getScrollPosition, getViewportSize,
  isAndroid, isBrowser, isElementInViewport, isIOS, isMobile, onResize,
  readFromClipboard, sessionStorage, smoothScrollTo,
  storage, watchColorScheme
} from './helpers/browser'

// ============================================================================
// BLOCKCHAIN ENVIRONMENT HELPERS
// ============================================================================
export {
  getDefaultChainId,
  getNetworkName, isDev, isDev as isDevNetwork, isLocal, isLocal as isLocalNetwork, isProduction, isProduction as isProductionNetwork, isTestnet, isTestnet as isTestnetNetwork
} from './env-helpers'

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

// Legacy compatibility exports
export const createAdminClient = createAdminApiClient;
export const createServerAdminClient = (baseURL?: string, token?: string) =>
  createAdminApiClient({ baseURL, token, serverSide: true });
export const apiClient = createFrontendApiClient();
export const createClient = createFrontendApiClient;
export const createApiClient = createApiClientBase;

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// ============================================================================
// SIMPLE HELPER UTILITIES (inline implementations)
// ============================================================================

/**
 * Debounce function - delays execution until after wait milliseconds have elapsed
 * since the last time it was invoked. Useful for search inputs, resize handlers.
 * @example
 * const debouncedSearch = debounce((query: string) => search(query), 300);
 * input.addEventListener('input', (e) => debouncedSearch(e.target.value));
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

/**
 * Throttle function - ensures function is called at most once per specified time limit.
 * Useful for scroll handlers, mousemove events.
 * @example
 * const throttledScroll = throttle(() => updatePosition(), 100);
 * window.addEventListener('scroll', throttledScroll);
 */
export function throttle<T extends (...args: unknown[]) => unknown>(
  fn: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle = false;
  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      fn(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

/**
 * Sleep/delay function - returns promise that resolves after specified milliseconds.
 * Useful for adding delays in async functions, retry logic, or animations.
 * @example
 * await sleep(1000); // Wait 1 second
 * await sleep(500).then(() => console.log('Done waiting'));
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Deep clone object using JSON serialization. Note: Does NOT preserve:
 * - Functions, undefined, Symbol properties
 * - Date objects (become strings)
 * - Circular references (will throw)
 * Consider structuredClone() for full support if available.
 */
export function deepClone<T>(obj: T): T {
  return JSON.parse(JSON.stringify(obj)) as T;
}

// ============================================================================
// CONVENIENCE RE-EXPORTS
// ============================================================================
export const utils = {
  formatCurrency,
  formatDate,
  formatPercentage,
  formatPrice,
  formatEPSGrowth,
  debounce,
  throttle,
  deepClone,
  isEmpty,
  generateId,
  cn,
  createAdminApiClient,
  createFrontendApiClient,
  handlePaginatedRequest,
  handleSimpleRequest,
  isApiError
}

// ============================================================================
// QUARTER PRIORITY HELPER
// ============================================================================
export function getQuarterPriority(epsGrowth: number, daysUntilAnnouncement: number): 'high' | 'medium' | 'low' {
  if (Math.abs(epsGrowth) > 20 || daysUntilAnnouncement <= 7) {
    return 'high'
  } else if (Math.abs(epsGrowth) > 10 || daysUntilAnnouncement <= 14) {
    return 'medium'
  }
  return 'low'
}

// ============================================================================
// LOGGING UTILITIES (from deleted core/logging)
// ============================================================================

/** Check if running in development environment */
export const isDevEnvironment = typeof window !== 'undefined'
  ? window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'
  : process.env.NODE_ENV === 'development';

/** Check if running in production environment */
export const isProdEnvironment = !isDevEnvironment;

/** Development-only logging (no-op in production) */
export function devLog(...args: unknown[]): void {
  if (isDevEnvironment) {
    const [message, ...rest] = args;
    if (typeof message === 'string') {
      appLogger.debug(message, ...rest);
    } else {
      appLogger.debug('Dev Log:', ...args);
    }
  }
}

/** Development-only warning (no-op in production) */
export function devWarn(...args: unknown[]): void {
  if (isDevEnvironment) {
    const [message, ...rest] = args;
    if (typeof message === 'string') {
      appLogger.warn(message, ...rest);
    } else {
      appLogger.warn('Dev Warning:', ...args);
    }
  }
}

/** Development-only info (no-op in production) */
export function devInfo(...args: unknown[]): void {
  if (isDevEnvironment) {
    const [message, ...rest] = args;
    if (typeof message === 'string') {
      appLogger.info(message, ...rest);
    } else {
      appLogger.info('Dev Info:', ...args);
    }
  }
}

/** Safe error handling */
export function safeError(error: unknown): { message: string; stack?: string } {
  if (error instanceof Error) {
    return { message: error.message, stack: error.stack };
  }
  return { message: String(error) };
}

/** Simple logger object */
export const logger = {
  debug: (...args: unknown[]) => {
    const [msg, ...rest] = args;
    if (typeof msg === 'string') { appLogger.debug(msg, ...rest); }
    else { appLogger.debug('Debug:', ...args); }
  },
  info: (...args: unknown[]) => {
    const [msg, ...rest] = args;
    if (typeof msg === 'string') { appLogger.info(msg, ...rest); }
    else { appLogger.info('Info:', ...args); }
  },
  warn: (...args: unknown[]) => {
    const [msg, ...rest] = args;
    if (typeof msg === 'string') { appLogger.warn(msg, ...rest); }
    else { appLogger.warn('Warn:', ...args); }
  },
  error: (...args: unknown[]) => {
    const [msg, ...rest] = args;
    if (typeof msg === 'string') { appLogger.error(msg, ...rest); }
    else { appLogger.error('Error:', ...args); }
  },
};

export default utils