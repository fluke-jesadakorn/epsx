/**
 * OBJECT UTILITIES
 * Object manipulation, validation, and transformation functions
 */

/**
 * Deep clone an object
 */
export function deepClone<T>(obj: T): T {
  if (obj === null || typeof obj !== 'object') {
    return obj
  }

  if (obj instanceof Date) {
    return new Date(obj.getTime()) as any
  }

  if (obj instanceof Array) {
    return obj.map(item => deepClone(item)) as any
  }

  if (typeof obj === 'object') {
    const cloned = {} as T
    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        cloned[key] = deepClone(obj[key])
      }
    }
    return cloned
  }

  return obj
}

/**
 * Object utilities namespace
 */
export const objectUtils = {
  /**
   * Check if object is empty
   */
  isEmpty: (obj: object) => Object.keys(obj).length === 0,

  /**
   * Pick specific keys from object
   */
  pick: <T extends object, K extends keyof T>(obj: T, keys: K[]): Pick<T, K> => {
    const result = {} as Pick<T, K>
    keys.forEach(key => {
      if (key in obj) result[key] = obj[key]
    })
    return result
  },

  /**
   * Omit specific keys from object
   */
  omit: <T, K extends keyof T>(obj: T, keys: K[]): Omit<T, K> => {
    const result = { ...obj }
    keys.forEach(key => delete result[key])
    return result
  }
}

/**
 * Advanced object utilities
 */
export const object = {
  /**
   * Pick specific keys from object
   */
  pick<T extends object, K extends keyof T>(obj: T, keys: K[]): Pick<T, K> {
    const result = {} as Pick<T, K>
    keys.forEach(key => {
      if (key in obj) {
        result[key] = obj[key]
      }
    })
    return result
  },

  /**
   * Omit specific keys from object
   */
  omit<T extends object, K extends keyof T>(obj: T, keys: K[]): Omit<T, K> {
    const result = { ...obj }
    keys.forEach(key => {
      delete result[key]
    })
    return result
  },

  /**
   * Check if object is empty
   */
  isEmpty(obj: object): boolean {
    return Object.keys(obj).length === 0
  },

  /**
   * Deep merge objects
   */
  merge<T extends object>(target: T, ...sources: Partial<T>[]): T {
    if (!sources.length) return target
    const source = sources.shift()

    if (this.isObject(target) && this.isObject(source)) {
      for (const key in source) {
        if (this.isObject(source[key])) {
          if (!target[key]) Object.assign(target, { [key]: {} })
          this.merge(target[key] as any, source[key] as any)
        } else {
          Object.assign(target, { [key]: source[key] })
        }
      }
    }

    return this.merge(target, ...sources)
  },

  /**
   * Check if value is an object
   */
  isObject(item: any): item is object {
    return item && typeof item === 'object' && !Array.isArray(item)
  },

  /**
   * Get nested property value safely
   */
  get<T>(obj: any, path: string, defaultValue?: T): T {
    const keys = path.split('.')
    let current = obj

    for (const key of keys) {
      if (current == null || typeof current !== 'object') {
        return defaultValue as T
      }
      current = current[key]
    }

    return current === undefined ? defaultValue as T : current
  },

  /**
   * Set nested property value
   */
  set(obj: any, path: string, value: any): void {
    const keys = path.split('.')
    const lastKey = keys.pop()

    if (!lastKey) return

    let current = obj
    for (const key of keys) {
      if (!(key in current) || typeof current[key] !== 'object') {
        current[key] = {}
      }
      current = current[key]
    }

    current[lastKey] = value
  },

  /**
   * Deep comparison of two objects
   */
  deepEqual(obj1: any, obj2: any): boolean {
    if (obj1 === obj2) return true

    if (obj1 == null || obj2 == null) return obj1 === obj2

    if (typeof obj1 !== typeof obj2) return false

    if (typeof obj1 !== 'object') return obj1 === obj2

    if (Array.isArray(obj1) !== Array.isArray(obj2)) return false

    const keys1 = Object.keys(obj1)
    const keys2 = Object.keys(obj2)

    if (keys1.length !== keys2.length) return false

    for (const key of keys1) {
      if (!keys2.includes(key)) return false
      if (!this.deepEqual(obj1[key], obj2[key])) return false
    }

    return true
  },

  /**
   * Flatten nested object to dot notation
   */
  flatten(obj: any, prefix: string = ''): Record<string, any> {
    const result: Record<string, any> = {}

    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        const newKey = prefix ? `${prefix}.${key}` : key

        if (this.isObject(obj[key]) && !Array.isArray(obj[key])) {
          Object.assign(result, this.flatten(obj[key], newKey))
        } else {
          result[newKey] = obj[key]
        }
      }
    }

    return result
  },

  /**
   * Unflatten dot notation object to nested object
   */
  unflatten(obj: Record<string, any>): any {
    const result: any = {}

    for (const key in obj) {
      this.set(result, key, obj[key])
    }

    return result
  },

  /**
   * Transform object keys
   */
  transformKeys<T extends object>(
    obj: T,
    transformer: (key: string) => string
  ): any {
    if (!this.isObject(obj)) return obj

    const result: any = {}

    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        const transformedKey = transformer(key)
        const value = obj[key]

        if (this.isObject(value)) {
          result[transformedKey] = this.transformKeys(value, transformer)
        } else if (Array.isArray(value)) {
          result[transformedKey] = value.map(item =>
            this.isObject(item) ? this.transformKeys(item, transformer) : item
          )
        } else {
          result[transformedKey] = value
        }
      }
    }

    return result
  },

  /**
   * Map object values
   */
  mapValues<T, R>(
    obj: Record<string, T>,
    mapper: (value: T, key: string) => R
  ): Record<string, R> {
    const result: Record<string, R> = {}

    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        const value = obj[key];
        if (value !== undefined) {
          result[key] = mapper(value, key)
        }
      }
    }

    return result
  },

  /**
   * Filter object by predicate
   */
  filter<T>(
    obj: Record<string, T>,
    predicate: (value: T, key: string) => boolean
  ): Record<string, T> {
    const result: Record<string, T> = {}

    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        const value = obj[key];
        if (value !== undefined && predicate(value, key)) {
          result[key] = value
        }
      }
    }

    return result
  },

  /**
   * Invert object (keys become values, values become keys)
   */
  invert<T extends string | number>(obj: Record<string, T>): Record<T, string> {
    const result = {} as Record<T, string>

    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        const value = obj[key];
        if (value !== undefined) {
          result[value] = key
        }
      }
    }

    return result
  },

  /**
   * Group object entries by a key function
   */
  groupBy<T>(
    obj: Record<string, T>,
    keyFn: (value: T, key: string) => string
  ): Record<string, Record<string, T>> {
    const result: Record<string, Record<string, T>> = {}

    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        const value = obj[key];
        if (value !== undefined) {
          const groupKey = keyFn(value, key)
          if (!result[groupKey]) {
            result[groupKey] = {}
          }
          const group = result[groupKey];
          if (group) {
            group[key] = value
          }
        }
      }
    }

    return result
  }
}