'use server';

import { unstable_cache } from 'next/cache';

// Cache configuration
const DEFAULT_CACHE_TTL = 300; // 5 minutes
const LONG_CACHE_TTL = 3600; // 1 hour
const SHORT_CACHE_TTL = 60; // 1 minute

// Cache tags for revalidation
export const CACHE_TAGS = {
  PUBLIC_RANKINGS: 'public-rankings',
  USER_PROFILE: 'user-profile',
  ANALYTICS: 'analytics',
  PERMISSIONS: 'permissions',
  STOCK_DATA: 'stock-data',
} as const;

/**
 * Enhanced cache wrapper with optimized SSR performance
 */
export function createCachedFunction<T extends any[], R>(
  fn: (...args: T) => Promise<R>,
  options: {
    keyPrefix: string;
    ttl?: number;
    tags?: string[];
    revalidate?: number;
  }
) {
  return unstable_cache(
    fn,
    [options.keyPrefix],
    {
      revalidate: options.revalidate || options.ttl || DEFAULT_CACHE_TTL,
      tags: options.tags || [],
    }
  );
}

/**
 * Cache public ranking data with optimized performance
 */
export const getCachedPublicRankings = createCachedFunction(
  async (startRank: number, limit: number) => {
    const { fetchPublicRankingData } = await import('@/app/actions/publicRanking');
    return fetchPublicRankingData(startRank, limit);
  },
  {
    keyPrefix: 'public-rankings',
    ttl: LONG_CACHE_TTL,
    tags: [CACHE_TAGS.PUBLIC_RANKINGS, CACHE_TAGS.STOCK_DATA],
    revalidate: 3600, // 1 hour for public data
  }
);

/**
 * Cache user permissions with shorter TTL for security
 */
export const getCachedUserPermissions = createCachedFunction(
  async (userId: string) => {
    // Implementation would fetch from backend API
    const response = await fetch(`${process.env.NEXTAUTH_URL}/api/auth/permissions/${userId}`);
    if (!response.ok) throw new Error('Failed to fetch permissions');
    return response.json();
  },
  {
    keyPrefix: 'user-permissions',
    ttl: SHORT_CACHE_TTL,
    tags: [CACHE_TAGS.PERMISSIONS],
    revalidate: 60, // 1 minute for permissions
  }
);

/**
 * Cache analytics data with medium TTL
 */
export const getCachedAnalytics = createCachedFunction(
  async (userId: string, filters: any) => {
    // Implementation would fetch analytics from backend
    const response = await fetch(`${process.env.NEXTAUTH_URL}/api/analytics`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ userId, filters }),
    });
    if (!response.ok) throw new Error('Failed to fetch analytics');
    return response.json();
  },
  {
    keyPrefix: 'analytics-data',
    ttl: DEFAULT_CACHE_TTL,
    tags: [CACHE_TAGS.ANALYTICS],
    revalidate: 300, // 5 minutes for analytics
  }
);

/**
 * Cache utility for server-side data fetching with automatic revalidation
 */
export class SSRCache {
  private static instance: SSRCache;
  
  public static getInstance(): SSRCache {
    if (!SSRCache.instance) {
      SSRCache.instance = new SSRCache();
    }
    return SSRCache.instance;
  }

  /**
   * Get cached data with fallback and error handling
   */
  async getCachedData<T>(
    key: string,
    fetcher: () => Promise<T>,
    options: {
      ttl?: number;
      tags?: string[];
      fallback?: T;
    } = {}
  ): Promise<T> {
    try {
      const cachedFetcher = createCachedFunction(
        async () => fetcher(),
        {
          keyPrefix: key,
          ttl: options.ttl || DEFAULT_CACHE_TTL,
          tags: options.tags || [],
        }
      );
      
      return await cachedFetcher();
    } catch (error) {
      console.error(`Cache error for key ${key}:`, error);
      
      // Return fallback data if available
      if (options.fallback) {
        return options.fallback;
      }
      
      throw error;
    }
  }

  /**
   * Prefetch data for better performance
   */
  async prefetchData<T>(
    key: string,
    fetcher: () => Promise<T>,
    options: {
      ttl?: number;
      tags?: string[];
    } = {}
  ): Promise<void> {
    try {
      await this.getCachedData(key, fetcher, options);
    } catch (error) {
      // Silently fail prefetch operations
      console.warn(`Prefetch failed for key ${key}:`, error);
    }
  }
}

/**
 * Revalidate cache by tags
 */
export async function revalidateCacheByTag(tag: string) {
  const { revalidateTag } = await import('next/cache');
  revalidateTag(tag);
}

/**
 * Revalidate cache by path
 */
export async function revalidateCacheByPath(path: string) {
  const { revalidatePath } = await import('next/cache');
  revalidatePath(path);
}