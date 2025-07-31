/**
 * DOM and utility functions for browser environments
 */

/**
 * Create debounced function
 */
export function debounce<T extends (...args: any[]) => any>(
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
 * Create throttled function
 */
export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean;
  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      fn(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

/**
 * Wait for a specified duration
 */
export function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Generate a random ID with optional prefix
 */
export function generateId(prefix: string = 'id'): string {
  return `${prefix}_${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Deep clone an object using JSON methods
 */
export function deepClone<T>(obj: T): T {
  return JSON.parse(JSON.stringify(obj));
}

/**
 * Check if running on client side
 */
export const isClient = typeof window !== 'undefined';

/**
 * Check if running on server side
 */
export const isServer = !isClient;

/**
 * Local storage utilities (client-side only)
 */
export const storage = {
  get: <T>(key: string): T | null => {
    if (!isClient) return null;
    try {
      const item = localStorage.getItem(key);
      return item ? JSON.parse(item) : null;
    } catch {
      return null;
    }
  },
  set: (key: string, value: any): void => {
    if (!isClient) return;
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch (e) {
      console.warn('Failed to save to localStorage:', e);
    }
  },
  remove: (key: string): void => {
    if (!isClient) return;
    localStorage.removeItem(key);
  },
  clear: (): void => {
    if (!isClient) return;
    localStorage.clear();
  },
};

// Legacy aliases for backward compatibility
export const deb = debounce;
export const thr = throttle;
export const sleep = wait;
export const clone = deepClone;
export const id = generateId;
export const ls = storage;