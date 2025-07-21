import {  clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

import type {ClassValue} from 'clsx';

/**
 * Combines class names with Tailwind CSS classes, handling conflicts and merging properly
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * Format date to readable string
 */
export function fmtDate(input: string | number | Date): string {
  const date = new Date(input);
  return date.toLocaleDateString('en-US', {
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  });
}

/**
 * Format currency amount
 */
export function fmtCurrency(
  amt: number,
  cur: string = 'USD',
  locale: string = 'en-US'
): string {
  return new Intl.NumberFormat(locale, {
    style: 'currency',
    currency: cur,
  }).format(amt);
}

/**
 * Wait for a specified duration
 */
export function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Import utilities from centralized location
import { deb, thr, clone as deepClone, id as generateId } from '@/utils/util';

/**
 * Create debounced function (re-export)
 */
export const debounce = deb;

/**
 * Create throttled function (re-export) 
 */
export const throttle = thr;

/**
 * Check if client side
 */
export const isClient = typeof window !== 'undefined';

/**
 * Check if server side
 */
export const isServer = !isClient;

/**
 * Deep clone object (re-export)
 */
export const clone = deepClone;

/**
 * Generate random string (re-export)
 */
export const genId = generateId;

/**
 * Parse URL query to object
 */
export function parseQuery(queryString: string): Record<string, string> {
  const params = new URLSearchParams(queryString.startsWith('?') ? queryString.slice(1) : queryString);
  return Object.fromEntries(params.entries());
}

/**
 * Capitalize first letter
 */
export function cap(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Get file extension
 */
export function getExt(filename: string): string {
  return filename.slice((filename.lastIndexOf('.') - 1 >>> 0) + 2);
}
