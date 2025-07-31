/**
 * Array and object manipulation utilities
 */

/**
 * Array utilities
 */
export const array = {
  /**
   * Remove duplicates from array
   */
  unique: <T>(arr: T[]): T[] => [...new Set(arr)],

  /**
   * Split array into chunks
   */
  chunk: <T>(arr: T[], size: number): T[][] =>
    Array.from({ length: Math.ceil(arr.length / size) }, (_, i) =>
      arr.slice(i * size, i * size + size)
    ),

  /**
   * Group array items by key
   */
  groupBy: <T>(arr: T[], key: keyof T): Record<string, T[]> =>
    arr.reduce((acc, item) => {
      const k = String(item[key]);
      acc[k] = acc[k] || [];
      acc[k].push(item);
      return acc;
    }, {} as Record<string, T[]>),

  /**
   * Shuffle array randomly
   */
  shuffle: <T>(arr: T[]): T[] => {
    const shuffled = [...arr];
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
    }
    return shuffled;
  },

  /**
   * Get random item from array
   */
  random: <T>(arr: T[]): T | undefined =>
    arr[Math.floor(Math.random() * arr.length)],

  /**
   * Check if arrays are equal
   */
  equal: <T>(a: T[], b: T[]): boolean =>
    a.length === b.length && a.every((val, i) => val === b[i]),
};

/**
 * Object utilities
 */
export const object = {
  /**
   * Pick specific keys from object
   */
  pick: <T, K extends keyof T>(obj: T, keys: K[]): Pick<T, K> =>
    keys.reduce((acc, key) => ({ ...acc, [key]: obj[key] }), {} as Pick<T, K>),

  /**
   * Omit specific keys from object  
   */
  omit: <T, K extends keyof T>(obj: T, keys: K[]): Omit<T, K> => {
    const result = { ...obj };
    keys.forEach((key) => delete result[key]);
    return result;
  },

  /**
   * Check if object is empty
   */
  isEmpty: (obj: any): boolean =>
    !obj || Object.keys(obj).length === 0,

  /**
   * Deep merge objects
   */
  merge: <T extends Record<string, any>>(target: T, ...sources: Partial<T>[]): T => {
    if (!sources.length) return target;
    const source = sources.shift();
    
    if (source) {
      for (const key in source) {
        if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
          if (!target[key]) target[key] = {} as any;
          object.merge(target[key], source[key]);
        } else {
          target[key] = source[key] as any;
        }
      }
    }
    
    return object.merge(target, ...sources);
  },

  /**
   * Get nested property value safely
   */
  get: (obj: any, path: string, defaultValue?: any): any => {
    const keys = path.split('.');
    let result = obj;
    
    for (const key of keys) {
      if (result == null || typeof result !== 'object') {
        return defaultValue;
      }
      result = result[key];
    }
    
    return result !== undefined ? result : defaultValue;
  },

  /**
   * Set nested property value
   */
  set: (obj: any, path: string, value: any): void => {
    const keys = path.split('.');
    const lastKey = keys.pop()!;
    let current = obj;
    
    for (const key of keys) {
      if (!(key in current) || typeof current[key] !== 'object') {
        current[key] = {};
      }
      current = current[key];
    }
    
    current[lastKey] = value;
  },
};

/**
 * URL utilities
 */
export const url = {
  /**
   * Build URL with query parameters
   */
  build: (base: string, params: Record<string, any>): string => {
    const url = new URL(base);
    Object.entries(params).forEach(([key, value]) => {
      if (value !== null && value !== undefined) {
        url.searchParams.set(key, String(value));
      }
    });
    return url.toString();
  },

  /**
   * Parse URL query parameters to object
   */
  parseQuery: (queryString: string): Record<string, string> => {
    const params = new URLSearchParams(
      queryString.startsWith('?') ? queryString.slice(1) : queryString
    );
    return Object.fromEntries(params.entries());
  },

  /**
   * Get file extension from URL or filename
   */
  getExtension: (filename: string): string => {
    return filename.slice(((filename.lastIndexOf('.') - 1) >>> 0) + 2);
  },
};

// Legacy aliases for backward compatibility
export const arr = array;
export const obj = object;