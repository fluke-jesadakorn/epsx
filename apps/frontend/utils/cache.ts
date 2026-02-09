// Cache utilities
const cache = new Map<string, { value: unknown; expiry: number }>()

export const set = (key: string, val: unknown, ttl = 300000): void => {
  cache.set(key, { value: val, expiry: Date.now() + ttl })
}

export const get = <T>(key: string): T | null => {
  const item = cache.get(key)
  if (!item) {return null}
  if (Date.now() > item.expiry) {
    cache.delete(key)
    return null
  }
  return item.value as T
}

export const stats = () => ({
  size: cache.size,
  keys: Array.from(cache.keys())
})

export const clean = (): void => {
  const now = Date.now()
  for (const [key, item] of cache.entries()) {
    if (now > item.expiry) {cache.delete(key)}
  }
}

export const clear = (key?: string): void => {
  if (key !== undefined) {
    cache.delete(key)
  } else {
    cache.clear()
  }
}

// Stock-specific cache helpers
export const setStock = (symbol: string, data: unknown, ttl = 300000) => set(`stock:${symbol}`, data, ttl)
export const getStock = (symbol: string) => get(`stock:${symbol}`)
export const setBulk = (stocks: { symbol: string }[], ttl = 300000) => stocks.forEach(s => setStock(s.symbol, s, ttl))
export const getBulk = (symbols: string[]) => symbols.map(s => getStock(s)).filter(Boolean)

// Backward compatibility
export const CacheManager = {
  setBulk,
  getBulk,
  getStats: stats,
  scheduleCleanup: () => setInterval(clean, 60000),
  preloadSymbols: setBulk,
  clearCache: clear,
  getCacheMetrics: stats
}
