/**
 * ASYNC UTILITIES
 * Debounce, throttle, sleep, and async helper functions
 */

/**
 * Debounce function to limit the rate of function calls
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number,
  immediate?: boolean
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | null = null
  
  return function(this: any, ...args: Parameters<T>) {
    const callNow = immediate && !timeout
    
    if (timeout) {
      clearTimeout(timeout)
    }
    
    timeout = setTimeout(() => {
      timeout = null
      if (!immediate) func.apply(this, args)
    }, wait)
    
    if (callNow) func.apply(this, args)
  }
}

/**
 * Throttle function to limit the rate of function calls
 */
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean
  
  return function(this: any, ...args: Parameters<T>) {
    if (!inThrottle) {
      func.apply(this, args)
      inThrottle = true
      setTimeout(() => inThrottle = false, limit)
    }
  }
}

/**
 * Sleep/delay function
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

/**
 * Retry function with exponential backoff
 */
export async function retry<T>(
  fn: () => Promise<T>,
  options: {
    retries?: number
    delay?: number
    backoff?: number
    maxDelay?: number
  } = {}
): Promise<T> {
  const {
    retries = 3,
    delay = 1000,
    backoff = 2,
    maxDelay = 10000
  } = options

  let lastError: Error
  let currentDelay = delay

  for (let attempt = 0; attempt <= retries; attempt++) {
    try {
      return await fn()
    } catch (error) {
      lastError = error as Error
      
      if (attempt === retries) {
        throw lastError
      }
      
      await sleep(Math.min(currentDelay, maxDelay))
      currentDelay *= backoff
    }
  }
  
  throw lastError!
}

/**
 * Create a promise that resolves after a specified timeout
 */
export function timeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return Promise.race([
    promise,
    new Promise<never>((_, reject) =>
      setTimeout(() => reject(new Error('Operation timed out')), ms)
    )
  ])
}

/**
 * Batch async operations with concurrency limit
 */
export async function batchAsync<T, R>(
  items: T[],
  asyncFn: (item: T) => Promise<R>,
  concurrency: number = 10
): Promise<R[]> {
  const results: R[] = []
  const executing: Promise<void>[] = []
  
  for (const item of items) {
    const promise = asyncFn(item).then(result => {
      results.push(result)
    })
    
    executing.push(promise)
    
    if (executing.length >= concurrency) {
      await Promise.race(executing)
      executing.splice(executing.findIndex(p => p === promise), 1)
    }
  }
  
  await Promise.all(executing)
  return results
}

/**
 * Create a cancellable promise
 */
export function cancellable<T>(promise: Promise<T>): {
  promise: Promise<T>
  cancel: () => void
} {
  let cancelled = false
  
  const cancellablePromise = new Promise<T>((resolve, reject) => {
    promise
      .then(value => {
        if (!cancelled) {
          resolve(value)
        }
      })
      .catch(error => {
        if (!cancelled) {
          reject(error)
        }
      })
  })
  
  return {
    promise: cancellablePromise,
    cancel: () => {
      cancelled = true
    }
  }
}

/**
 * Memoize async function results with TTL
 */
export function memoizeAsync<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  options: {
    ttl?: number // Time to live in milliseconds
    maxSize?: number // Maximum cache size
  } = {}
): T {
  const { ttl = 60000, maxSize = 100 } = options
  const cache = new Map<string, { value: any; timestamp: number }>()
  
  return ((...args: Parameters<T>) => {
    const key = JSON.stringify(args)
    const now = Date.now()
    
    // Check if we have a valid cached result
    const cached = cache.get(key)
    if (cached && now - cached.timestamp < ttl) {
      return Promise.resolve(cached.value)
    }
    
    // Execute function and cache result
    const promise = fn(...args)
    promise.then(result => {
      // Implement LRU eviction if cache is full
      if (cache.size >= maxSize) {
        const firstKey = cache.keys().next().value
        if (firstKey) {
          cache.delete(firstKey)
        }
      }
      
      cache.set(key, { value: result, timestamp: now })
    })
    
    return promise
  }) as T
}

/**
 * Create a queue for sequential async operations
 */
export class AsyncQueue {
  private queue: Array<() => Promise<any>> = []
  private running = false
  
  async add<T>(fn: () => Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
      this.queue.push(async () => {
        try {
          const result = await fn()
          resolve(result)
        } catch (error) {
          reject(error)
        }
      })
      
      this.process()
    })
  }
  
  private async process(): Promise<void> {
    if (this.running || this.queue.length === 0) {
      return
    }
    
    this.running = true
    
    while (this.queue.length > 0) {
      const fn = this.queue.shift()!
      await fn()
    }
    
    this.running = false
  }
}

/**
 * Poll a function until it returns a truthy value or times out
 */
export async function poll<T>(
  fn: () => Promise<T>,
  options: {
    interval?: number
    timeout?: number
    condition?: (result: T) => boolean
  } = {}
): Promise<T> {
  const { interval = 1000, timeout = 30000, condition = Boolean } = options
  const startTime = Date.now()
  
  while (Date.now() - startTime < timeout) {
    const result = await fn()
    
    if (condition(result)) {
      return result
    }
    
    await sleep(interval)
  }
  
  throw new Error('Polling timed out')
}

/**
 * Combine multiple promises with different behaviors
 */
export const promises = {
  /**
   * Like Promise.all but fails fast and collects all errors
   */
  allSettled: async <T>(promises: Promise<T>[]): Promise<Array<{
    status: 'fulfilled' | 'rejected'
    value?: T
    reason?: any
  }>> => {
    return Promise.allSettled(promises)
  },
  
  /**
   * Like Promise.all but with a timeout
   */
  allWithTimeout: async <T>(promises: Promise<T>[], timeoutMs: number): Promise<T[]> => {
    return timeout(Promise.all(promises), timeoutMs)
  },
  
  /**
   * Race promises but collect all results
   */
  raceSettled: async <T>(promises: Promise<T>[]): Promise<{
    winner: T
    results: Array<{ status: 'fulfilled' | 'rejected'; value?: T; reason?: any }>
  }> => {
    const winner = await Promise.race(promises)
    const results = await Promise.allSettled(promises)
    return { winner, results }
  }
}