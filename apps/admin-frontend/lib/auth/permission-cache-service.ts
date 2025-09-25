/**
 * Centralized Permission Cache Service for Admin Frontend
 * Prevents heavy loops by caching permission checks and debouncing API calls
 */

interface PermissionCache {
  wallet_address: string;
  permissions: string[];
  admin_level: string;
  has_admin_access: boolean;
  timestamp: number;
  expires_at: number;
}

interface PendingRequest {
  promise: Promise<PermissionCache>;
  timestamp: number;
}

class PermissionCacheService {
  private cache = new Map<string, PermissionCache>();
  private pendingRequests = new Map<string, PendingRequest>();
  private readonly CACHE_TTL = 30000; // 30 seconds
  private readonly DEBOUNCE_TIME = 500; // 500ms debounce

  /**
   * Get permissions for a wallet with caching and debouncing
   */
  async getPermissions(walletAddress: string): Promise<PermissionCache | null> {
    const key = walletAddress.toLowerCase();
    const now = Date.now();

    // Check cache first
    const cached = this.cache.get(key);
    if (cached && cached.expires_at > now) {
      console.log('🚀 Permission Cache: Using cached permissions for', walletAddress.slice(0, 6));
      return cached;
    }

    // Check if request is already pending
    const pending = this.pendingRequests.get(key);
    if (pending && (now - pending.timestamp) < this.DEBOUNCE_TIME) {
      console.log('⏳ Permission Cache: Reusing pending request for', walletAddress.slice(0, 6));
      return pending.promise;
    }

    // Make new request
    console.log('🔄 Permission Cache: Fetching fresh permissions for', walletAddress.slice(0, 6));
    const promise = this.fetchPermissions(walletAddress);
    
    this.pendingRequests.set(key, {
      promise,
      timestamp: now,
    });

    try {
      const result = await promise;
      
      // Cache the result
      this.cache.set(key, result);
      
      // Clean up pending request
      this.pendingRequests.delete(key);
      
      return result;
    } catch (error) {
      // Clean up pending request on error
      this.pendingRequests.delete(key);
      throw error;
    }
  }

  /**
   * Fetch permissions from API
   */
  private async fetchPermissions(walletAddress: string): Promise<PermissionCache> {
    const response = await fetch('/api/auth/web3/permissions', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ wallet_address: walletAddress }),
      credentials: 'include',
    });

    if (!response.ok) {
      throw new Error(`Permission check failed: ${response.status}`);
    }

    const data = await response.json();
    const now = Date.now();

    return {
      wallet_address: data.wallet_address,
      permissions: data.permissions || [],
      admin_level: data.admin_level || 'none',
      has_admin_access: data.has_admin_access || false,
      timestamp: now,
      expires_at: now + this.CACHE_TTL,
    };
  }

  /**
   * Clear cache for specific wallet
   */
  clearCache(walletAddress?: string): void {
    if (walletAddress) {
      const key = walletAddress.toLowerCase();
      this.cache.delete(key);
      this.pendingRequests.delete(key);
      console.log('🧹 Permission Cache: Cleared cache for', walletAddress.slice(0, 6));
    } else {
      this.cache.clear();
      this.pendingRequests.clear();
      console.log('🧹 Permission Cache: Cleared all cache');
    }
  }

  /**
   * Get cached permissions without fetching
   */
  getCachedPermissions(walletAddress: string): PermissionCache | null {
    const key = walletAddress.toLowerCase();
    const cached = this.cache.get(key);
    
    if (cached && cached.expires_at > Date.now()) {
      return cached;
    }
    
    return null;
  }

  /**
   * Check if wallet has specific permission (cached)
   */
  hasPermission(walletAddress: string, permission: string): boolean {
    const cached = this.getCachedPermissions(walletAddress);
    if (!cached) return false;
    
    return cached.permissions.includes(permission) ||
           cached.permissions.includes('admin:*:*') ||
           cached.permissions.includes('epsx:*:*');
  }

  /**
   * Check if wallet has admin access (cached)
   */
  hasAdminAccess(walletAddress: string): boolean {
    const cached = this.getCachedPermissions(walletAddress);
    return cached?.has_admin_access || false;
  }

  /**
   * Cleanup expired cache entries and pending requests
   */
  cleanup(): void {
    const now = Date.now();
    
    // Clean expired cache
    for (const [key, cache] of this.cache.entries()) {
      if (cache.expires_at <= now) {
        this.cache.delete(key);
      }
    }
    
    // Clean old pending requests
    for (const [key, pending] of this.pendingRequests.entries()) {
      if (now - pending.timestamp > this.DEBOUNCE_TIME * 2) {
        this.pendingRequests.delete(key);
      }
    }
  }

  /**
   * Get cache statistics
   */
  getStats() {
    return {
      cacheSize: this.cache.size,
      pendingRequests: this.pendingRequests.size,
      cacheEntries: Array.from(this.cache.keys()).map(key => ({
        wallet: key.slice(0, 6) + '...' + key.slice(-4),
        expires_in: this.cache.get(key)!.expires_at - Date.now(),
      })),
    };
  }
}

// Singleton instance
export const permissionCacheService = new PermissionCacheService();

// Auto cleanup every minute
if (typeof window !== 'undefined') {
  setInterval(() => {
    permissionCacheService.cleanup();
  }, 60000);
}

// Convenience hook for React components
export function usePermissionCache() {
  const getPermissions = (walletAddress: string) => {
    return permissionCacheService.getPermissions(walletAddress);
  };

  const getCachedPermissions = (walletAddress: string) => {
    return permissionCacheService.getCachedPermissions(walletAddress);
  };

  const hasPermission = (walletAddress: string, permission: string) => {
    return permissionCacheService.hasPermission(walletAddress, permission);
  };

  const hasAdminAccess = (walletAddress: string) => {
    return permissionCacheService.hasAdminAccess(walletAddress);
  };

  const clearCache = (walletAddress?: string) => {
    permissionCacheService.clearCache(walletAddress);
  };

  return {
    getPermissions,
    getCachedPermissions,
    hasPermission,
    hasAdminAccess,
    clearCache,
    stats: permissionCacheService.getStats(),
  };
}