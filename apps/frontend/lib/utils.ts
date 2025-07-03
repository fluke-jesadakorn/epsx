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

/**
 * Create debounced function
 */
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  
  return function (...args: Parameters<T>) {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

/**
 * Create throttled function
 */
export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean = false;
  
  return function (...args: Parameters<T>) {
    if (!inThrottle) {
      fn(...args);
      inThrottle = true;
      setTimeout(() => inThrottle = false, delay);
    }
  };
}

/**
 * Check if client side
 */
export const isClient = typeof window !== 'undefined';

/**
 * Check if server side
 */
export const isServer = !isClient;

/**
 * Deep clone object
 */
export function clone<T>(obj: T): T {
  if (obj === null || typeof obj !== 'object') {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map(clone) as unknown as T;
  }

  return Object.fromEntries(
    Object.entries(obj).map(([key, value]) => [key, clone(value)])
  ) as T;
}

/**
 * Parse URL query to object
 */
export function parseQuery(queryString: string): Record<string, string> {
  const params = new URLSearchParams(queryString.startsWith('?') ? queryString.slice(1) : queryString);
  return Object.fromEntries(params.entries());
}

/**
 * Generate random string
 */
export function genId(len: number = 16): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  return Array.from(
    { length: len }, 
    () => chars.charAt(Math.floor(Math.random() * chars.length))
  ).join('');
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
