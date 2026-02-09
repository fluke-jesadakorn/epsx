/**
 * Consolidated Utility Exports
 * Domain-organized utility functions and classes
 */

// Logging and monitoring
// ============================================================================
// Common Utility Functions (from original utils.ts)
// ============================================================================

import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export * from './logging';

// Security utilities
export * from './security';

// Data processing and export
export * from './data';

// Validation utilities
export * from './validation';

/**
 * Utility function to merge Tailwind CSS classes with clsx
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * Helper to determine if we're in server component context
 */
export function isServerComponentContext(): boolean {
  return typeof window === 'undefined';
}

/**
 * Utility function to format dates
 */
export function formatDate(date: Date | string | number, locale = 'en-US'): string {
  const dateObj = new Date(date);
  return dateObj.toLocaleDateString(locale, {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  });
}

/**
 * Utility function to format relative time
 */
export function formatRelativeTime(date: Date | string | number, locale = 'en-US'): string {
  const now = new Date();
  const dateObj = new Date(date);
  const diffInSeconds = Math.floor((now.getTime() - dateObj.getTime()) / 1000);

  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' });

  if (diffInSeconds < 60) {
    return rtf.format(-diffInSeconds, 'second');
  } else if (diffInSeconds < 3600) {
    return rtf.format(-Math.floor(diffInSeconds / 60), 'minute');
  } else if (diffInSeconds < 86400) {
    return rtf.format(-Math.floor(diffInSeconds / 3600), 'hour');
  } else {
    return rtf.format(-Math.floor(diffInSeconds / 86400), 'day');
  }
}

/**
 * Utility function to capitalize first letter
 */
export function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Utility function to create a slug from a string
 */
export function createSlug(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, '') // Remove special characters
    .replace(/[\s_-]+/g, '-') // Replace spaces and underscores with dashes
    .replace(/^-+|-+$/g, ''); // Remove leading/trailing dashes
}

/**
 * Utility function to truncate text
 */
export function truncate(text: string, length: number, suffix = '...'): string {
  if (text.length <= length) {return text;}
  return text.slice(0, length).trim() + suffix;
}

/**
 * Utility function to generate a random ID
 */
export function generateId(prefix?: string): string {
  const id = Math.random().toString(36).slice(2) + Date.now().toString(36);
  return prefix ? `${prefix}-${id}` : id;
}

/**
 * Utility function to deep clone an object
 */
export function deepClone<T>(obj: T): T {
  if (obj === null || typeof obj !== 'object') {return obj;}
  if (obj instanceof Date) {return new Date(obj.getTime()) as T;}
  if (Array.isArray(obj)) {return obj.map(item => deepClone(item)) as T;}

  const cloned = {} as T;
  for (const key in obj) {
    if (Object.prototype.hasOwnProperty.call(obj, key)) {
      cloned[key] = deepClone(obj[key]);
    }
  }
  return cloned;
}

/**
 * Utility function to debounce a function
 */
export function debounce<T extends (...args: unknown[]) => void>(
  func: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: NodeJS.Timeout;

  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => func(...args), delay);
  };
}

/**
 * Utility function to throttle a function
 */
export function throttle<T extends (...args: unknown[]) => void>(
  func: T,
  delay: number
): (...args: Parameters<T>) => void {
  let lastCall = 0;

  return (...args: Parameters<T>) => {
    const now = Date.now();
    if (now - lastCall >= delay) {
      lastCall = now;
      func(...args);
    }
  };
}

/**
 * Utility function to check if code is running in browser
 */
export function isBrowser(): boolean {
  return typeof window !== 'undefined';
}

/**
 * Utility function to check if code is running on mobile
 */
export function isMobile(): boolean {
  if (!isBrowser()) {return false;}

  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
    navigator.userAgent
  );
}

/**
 * Utility function to get environment variable with fallback
 */
export function getEnvVar(key: string, fallback?: string): string {
  const value = process.env[key];
  if (value === undefined && fallback === undefined) {
    throw new Error(`Environment variable ${key} is not defined`);
  }
  return value || fallback || '';
}

/**
 * Utility function to safely parse JSON
 */
export function safeJsonParse<T>(str: string, fallback?: T): T | null {
  try {
    return JSON.parse(str);
  } catch {
    return fallback || null;
  }
}

/**
 * Utility function to format file size
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) {return '0 Bytes';}

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))  } ${  sizes[i]}`;
}

/**
 * Utility function to format currency
 */
export function fmtCurrency(amount: number, currency = 'USD'): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency
  }).format(amount);
}

/**
 * Utility function to get contrast color (black or white) for a background color
 */
export function getContrastColor(hexColor: string): string {
  // Remove hash if present
  const color = hexColor.replace('#', '');

  // Convert to RGB
  const r = parseInt(color.slice(0, 2), 16);
  const g = parseInt(color.slice(2, 4), 16);
  const b = parseInt(color.slice(4, 6), 16);

  // Calculate luminance
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;

  return luminance > 0.5 ? '#000000' : '#ffffff';
}

/**
 * Utility function to check if a string is a valid UUID
 */
export function isValidUUID(str: string): boolean {
  const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  return uuidRegex.test(str);
}

/**
 * Utility function to retry an async function
 */
export async function retry<T>(
  fn: () => Promise<T>,
  maxAttempts = 3,
  delay = 1000
): Promise<T> {
  let lastError: Error;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));

      if (attempt === maxAttempts) {
        throw lastError;
      }

      await new Promise(resolve => setTimeout(resolve, delay * attempt));
    }
  }

  throw lastError!;
}

/**
 * Utility function to create a promise that resolves after a delay
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Utility function to check if an object is empty
 */
export function isEmpty(obj: unknown): boolean {
  if (obj == null) {return true;}
  if (Array.isArray(obj) || typeof obj === 'string') {return obj.length === 0;}
  if (typeof obj === 'object') {return Object.keys(obj).length === 0;}
  return false;
}