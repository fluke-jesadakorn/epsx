import { revalidateTag, revalidatePath } from 'next/cache';
import { unstable_cache } from 'next/cache';

// Cache configuration constants
const CACHE_TAGS = {
  STOCKS: 'stocks',
  RANKINGS: 'rankings',
  USER_DATA: 'user-data',
  PERMISSIONS: 'permissions',
  ANALYTICS: 'analytics',
  SETTINGS: 'settings',
  ADMIN: 'admin',
} as const;

const CACHE_DURATIONS = {
  VERY_SHORT: 60, // 1 minute - for real-time data
  SHORT: 180, // 3 minutes - for frequently changing data
  MEDIUM: 600, // 10 minutes - for moderate change frequency
  LONG: 1800, // 30 minutes - for stable data
  VERY_LONG: 3600, // 1 hour - for very stable data
} as const;

export async function getCacheTags() {
  return CACHE_TAGS;
}

export async function getCacheDurations() {
  return CACHE_DURATIONS;
}

// Cache wrapper function with automatic tagging and revalidation
export async function createCachedFunction<T extends any[], R>(
  fn: (...args: T) => Promise<R>,
  options: {
    tags: string[];
    revalidate?: number;
    key?: string[];
  }
) {
  return unstable_cache(
    fn,
    options.key,
    {
      tags: options.tags,
      revalidate: options.revalidate || CACHE_DURATIONS.MEDIUM,
    }
  );
}

// Specific cache helpers for different data types
export async function cacheStock<T extends any[], R>(
  fn: (...args: T) => Promise<R>,
  key?: string[]
) {
  return await createCachedFunction(fn, {
    tags: [CACHE_TAGS.STOCKS],
    revalidate: CACHE_DURATIONS.SHORT,
    key,
  });
}

export async function cacheRankings<T extends any[], R>(
  fn: (...args: T) => Promise<R>,
  key?: string[]
) {
  return await createCachedFunction(fn, {
    tags: [CACHE_TAGS.RANKINGS],
    revalidate: CACHE_DURATIONS.MEDIUM,
    key,
  });
}

export async function cacheUserData<T extends any[], R>(
  fn: (...args: T) => Promise<R>,
  key?: string[]
) {
  return await createCachedFunction(fn, {
    tags: [CACHE_TAGS.USER_DATA],
    revalidate: CACHE_DURATIONS.SHORT,
    key,
  });
}

export async function cachePermissions<T extends any[], R>(
  fn: (...args: T) => Promise<R>,
  key?: string[]
) {
  return await createCachedFunction(fn, {
    tags: [CACHE_TAGS.PERMISSIONS],
    revalidate: CACHE_DURATIONS.LONG,
    key,
  });
}

export async function cacheAnalytics<T extends any[], R>(
  fn: (...args: T) => Promise<R>,
  key?: string[]
) {
  return await createCachedFunction(fn, {
    tags: [CACHE_TAGS.ANALYTICS],
    revalidate: CACHE_DURATIONS.VERY_LONG,
    key,
  });
}

// Cache invalidation functions
export async function invalidateStockCache() {
  revalidateTag(CACHE_TAGS.STOCKS);
}

export async function invalidateRankingsCache() {
  revalidateTag(CACHE_TAGS.RANKINGS);
}

export async function invalidateUserCache() {
  revalidateTag(CACHE_TAGS.USER_DATA);
}

export async function invalidatePermissionsCache() {
  revalidateTag(CACHE_TAGS.PERMISSIONS);
}

export async function invalidateAnalyticsCache() {
  revalidateTag(CACHE_TAGS.ANALYTICS);
}

export async function invalidateAdminCache() {
  revalidateTag(CACHE_TAGS.ADMIN);
}

// Path-based revalidation for specific pages
export async function revalidateDashboard() {
  revalidatePath('/dashboard');
}

export async function revalidateRankings() {
  revalidatePath('/rankings');
}

export async function revalidateProfile() {
  revalidatePath('/profile');
}

export async function revalidateAdminPages() {
  revalidatePath('/admin', 'layout');
}

// Combined invalidation for related data
export async function invalidateUserRelatedCache() {
  await Promise.all([
    invalidateUserCache(),
    invalidatePermissionsCache(),
    revalidateProfile(),
  ]);
}

export async function invalidateStockRelatedCache() {
  await Promise.all([
    invalidateStockCache(),
    invalidateRankingsCache(),
    revalidateDashboard(),
    revalidateRankings(),
  ]);
}

// Background cache warming functions
export async function warmStockCache(symbols: string[]) {
  // Preload popular stocks to improve performance
  try {
    const { preloadStocks } = await import('../actions/stocks');
    await preloadStocks(symbols);
  } catch (error) {
    console.error('Failed to warm stock cache:', error);
  }
}

// ISR-style cache management with stale-while-revalidate pattern
export async function createStaleWhileRevalidate<T extends any[], R>(
  fetchFn: (...args: T) => Promise<R>,
  options: {
    staleTime: number;
    cacheKey: string;
    tags: string[];
  }
) {
  const cachedFn = await createCachedFunction(fetchFn, {
    tags: options.tags,
    revalidate: options.staleTime,
  });

  return async (...args: T): Promise<R> => {
    try {
      // Try to get cached data first
      const result = await cachedFn(...args);
      
      // Trigger background revalidation if needed
      // This is handled automatically by Next.js cache system
      
      return result;
    } catch (error) {
      console.error(`Cache miss for ${options.cacheKey}:`, error);
      // Fallback to direct function call
      return await fetchFn(...args);
    }
  };
}

// Cache status monitoring
export async function getCacheStatus() {
  return {
    timestamp: new Date().toISOString(),
    tags: Object.values(CACHE_TAGS),
    config: {
      durations: CACHE_DURATIONS,
      enabled: true,
    },
  };
}