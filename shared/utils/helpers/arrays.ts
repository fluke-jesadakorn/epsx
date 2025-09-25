/**
 * ARRAY UTILITIES
 * Array manipulation, transformation, and analysis functions
 */

/**
 * Array utilities namespace (simple version)
 */
export const arrayUtils = {
  /**
   * Remove duplicates from array
   */
  unique: <T>(arr: T[]): T[] => [...new Set(arr)],
  
  /**
   * Group array by key
   */
  groupBy: <T>(arr: T[], key: keyof T): Record<string, T[]> => {
    return arr.reduce((groups, item) => {
      const val = String(item[key])
      groups[val] = groups[val] || []
      groups[val].push(item)
      return groups
    }, {} as Record<string, T[]>)
  },
  
  /**
   * Chunk array into smaller arrays
   */
  chunk: <T>(arr: T[], size: number): T[][] => {
    const chunks: T[][] = []
    for (let i = 0; i < arr.length; i += size) {
      chunks.push(arr.slice(i, i + size))
    }
    return chunks
  }
}

/**
 * Advanced array utilities
 */
export const array = {
  /**
   * Remove duplicates from array
   */
  unique<T>(arr: T[]): T[] {
    return [...new Set(arr)]
  },

  /**
   * Remove duplicates by key
   */
  uniqueBy<T>(arr: T[], key: keyof T): T[] {
    const seen = new Set()
    return arr.filter(item => {
      const val = item[key]
      if (seen.has(val)) {
        return false
      }
      seen.add(val)
      return true
    })
  },

  /**
   * Group array by key
   */
  groupBy<T>(arr: T[], key: keyof T): Record<string, T[]> {
    return arr.reduce((groups, item) => {
      const val = String(item[key])
      groups[val] = groups[val] || []
      groups[val].push(item)
      return groups
    }, {} as Record<string, T[]>)
  },

  /**
   * Group array by function result
   */
  groupByFn<T, K extends string | number>(
    arr: T[], 
    keyFn: (item: T) => K
  ): Record<K, T[]> {
    return arr.reduce((groups, item) => {
      const key = keyFn(item)
      groups[key] = groups[key] || []
      groups[key].push(item)
      return groups
    }, {} as Record<K, T[]>)
  },

  /**
   * Sort array by key
   */
  sortBy<T>(arr: T[], key: keyof T, direction: 'asc' | 'desc' = 'asc'): T[] {
    return [...arr].sort((a, b) => {
      const aVal = a[key]
      const bVal = b[key]
      
      if (aVal < bVal) return direction === 'asc' ? -1 : 1
      if (aVal > bVal) return direction === 'asc' ? 1 : -1
      return 0
    })
  },

  /**
   * Sort array by function result
   */
  sortByFn<T>(
    arr: T[], 
    keyFn: (item: T) => any, 
    direction: 'asc' | 'desc' = 'asc'
  ): T[] {
    return [...arr].sort((a, b) => {
      const aVal = keyFn(a)
      const bVal = keyFn(b)
      
      if (aVal < bVal) return direction === 'asc' ? -1 : 1
      if (aVal > bVal) return direction === 'asc' ? 1 : -1
      return 0
    })
  },

  /**
   * Chunk array into smaller arrays
   */
  chunk<T>(arr: T[], size: number): T[][] {
    const chunks: T[][] = []
    for (let i = 0; i < arr.length; i += size) {
      chunks.push(arr.slice(i, i + size))
    }
    return chunks
  },

  /**
   * Flatten nested arrays
   */
  flatten<T>(arr: (T | T[])[]): T[] {
    return arr.reduce((acc, val) => {
      if (Array.isArray(val)) {
        return acc.concat(this.flatten(val))
      } else {
        return acc.concat([val])
      }
    }, [] as T[])
  },

  /**
   * Deep flatten arrays to specified depth
   */
  flattenDeep<T>(arr: any[], depth: number = Infinity): T[] {
    return depth > 0 
      ? arr.reduce((acc, val) => 
          acc.concat(Array.isArray(val) ? this.flattenDeep(val, depth - 1) : val), [])
      : arr.slice()
  },

  /**
   * Get intersection of two arrays
   */
  intersection<T>(arr1: T[], arr2: T[]): T[] {
    return arr1.filter(item => arr2.includes(item))
  },

  /**
   * Get difference between two arrays (items in arr1 but not in arr2)
   */
  difference<T>(arr1: T[], arr2: T[]): T[] {
    return arr1.filter(item => !arr2.includes(item))
  },

  /**
   * Get union of two arrays (all unique items from both)
   */
  union<T>(arr1: T[], arr2: T[]): T[] {
    return this.unique([...arr1, ...arr2])
  },

  /**
   * Partition array into two arrays based on predicate
   */
  partition<T>(arr: T[], predicate: (item: T) => boolean): [T[], T[]] {
    const pass: T[] = []
    const fail: T[] = []
    
    arr.forEach(item => {
      if (predicate(item)) {
        pass.push(item)
      } else {
        fail.push(item)
      }
    })
    
    return [pass, fail]
  },

  /**
   * Take first n items from array
   */
  take<T>(arr: T[], count: number): T[] {
    return arr.slice(0, count)
  },

  /**
   * Take last n items from array
   */
  takeLast<T>(arr: T[], count: number): T[] {
    return arr.slice(-count)
  },

  /**
   * Drop first n items from array
   */
  drop<T>(arr: T[], count: number): T[] {
    return arr.slice(count)
  },

  /**
   * Drop last n items from array
   */
  dropLast<T>(arr: T[], count: number): T[] {
    return arr.slice(0, -count)
  },

  /**
   * Find index of item using predicate
   */
  findIndex<T>(arr: T[], predicate: (item: T) => boolean): number {
    return arr.findIndex(predicate)
  },

  /**
   * Find last index of item using predicate
   */
  findLastIndex<T>(arr: T[], predicate: (item: T) => boolean): number {
    for (let i = arr.length - 1; i >= 0; i--) {
      if (predicate(arr[i])) {
        return i
      }
    }
    return -1
  },

  /**
   * Check if array contains all items from another array
   */
  containsAll<T>(arr: T[], items: T[]): boolean {
    return items.every(item => arr.includes(item))
  },

  /**
   * Check if array contains any items from another array
   */
  containsAny<T>(arr: T[], items: T[]): boolean {
    return items.some(item => arr.includes(item))
  },

  /**
   * Shuffle array randomly
   */
  shuffle<T>(arr: T[]): T[] {
    const shuffled = [...arr]
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1))
      ;[shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
    }
    return shuffled
  },

  /**
   * Sample n random items from array
   */
  sample<T>(arr: T[], count: number): T[] {
    if (count >= arr.length) return [...arr]
    
    const shuffled = this.shuffle(arr)
    return shuffled.slice(0, count)
  },

  /**
   * Sample one random item from array
   */
  sampleOne<T>(arr: T[]): T | undefined {
    if (arr.length === 0) return undefined
    return arr[Math.floor(Math.random() * arr.length)]
  },

  /**
   * Transpose 2D array (swap rows and columns)
   */
  transpose<T>(matrix: T[][]): T[][] {
    if (matrix.length === 0) return []
    
    const result: T[][] = []
    const maxLength = Math.max(...matrix.map(row => row.length))
    
    for (let i = 0; i < maxLength; i++) {
      result[i] = []
      for (let j = 0; j < matrix.length; j++) {
        result[i][j] = matrix[j][i]
      }
    }
    
    return result
  },

  /**
   * Zip multiple arrays together
   */
  zip<T>(...arrays: T[][]): T[][] {
    if (arrays.length === 0) return []
    
    const maxLength = Math.max(...arrays.map(arr => arr.length))
    const result: T[][] = []
    
    for (let i = 0; i < maxLength; i++) {
      result[i] = arrays.map(arr => arr[i])
    }
    
    return result
  },

  /**
   * Create array of numbers in range
   */
  range(start: number, end: number, step: number = 1): number[] {
    const result: number[] = []
    
    if (step > 0) {
      for (let i = start; i < end; i += step) {
        result.push(i)
      }
    } else if (step < 0) {
      for (let i = start; i > end; i += step) {
        result.push(i)
      }
    }
    
    return result
  },

  /**
   * Compact array (remove falsy values)
   */
  compact<T>(arr: (T | null | undefined | false | 0 | '')[]): T[] {
    return arr.filter(Boolean) as T[]
  },

  /**
   * Get min value from array
   */
  min<T>(arr: T[], keyFn?: (item: T) => number): T | undefined {
    if (arr.length === 0) return undefined
    
    if (keyFn) {
      return arr.reduce((min, item) => 
        keyFn(item) < keyFn(min) ? item : min
      )
    }
    
    return arr.reduce((min, item) => item < min ? item : min)
  },

  /**
   * Get max value from array
   */
  max<T>(arr: T[], keyFn?: (item: T) => number): T | undefined {
    if (arr.length === 0) return undefined
    
    if (keyFn) {
      return arr.reduce((max, item) => 
        keyFn(item) > keyFn(max) ? item : max
      )
    }
    
    return arr.reduce((max, item) => item > max ? item : max)
  },

  /**
   * Calculate sum of array
   */
  sum<T>(arr: T[], keyFn?: (item: T) => number): number {
    if (keyFn) {
      return arr.reduce((sum, item) => sum + keyFn(item), 0)
    }
    
    return (arr as any[]).reduce((sum, item) => sum + item, 0)
  },

  /**
   * Calculate average of array
   */
  average<T>(arr: T[], keyFn?: (item: T) => number): number {
    if (arr.length === 0) return 0
    return this.sum(arr, keyFn) / arr.length
  },

  /**
   * Get frequency count of items in array
   */
  frequencies<T>(arr: T[]): Record<string, number> {
    return arr.reduce((freq, item) => {
      const key = String(item)
      freq[key] = (freq[key] || 0) + 1
      return freq
    }, {} as Record<string, number>)
  }
}