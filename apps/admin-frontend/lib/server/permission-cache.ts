/**
 * Server-side Permission Cache for Admin Frontend API Routes
 * Prevents heavy database queries by caching permission results
 */

import { Pool } from 'pg';

interface PermissionCacheEntry {
  wallet_address: string;
  permissions: string[];
  admin_level: string;
  has_admin_access: boolean;
  timestamp: number;
  expires_at: number;
}

class ServerPermissionCache {
  private cache = new Map<string, PermissionCacheEntry>();
  private readonly CACHE_TTL = 30000; // 30 seconds
  private pool: Pool | null = null;

  private getDbPool() {
    if (!this.pool) {
      const databaseUrl = process.env.DATABASE_URL || 'postgresql://postgres:password@localhost:5432/epsx_db';
      this.pool = new Pool({
        connectionString: databaseUrl,
        max: 5,
        idleTimeoutMillis: 30000,
        connectionTimeoutMillis: 2000,
      });
    }
    return this.pool;
  }

  async getPermissions(walletAddress: string): Promise<PermissionCacheEntry> {
    const key = walletAddress.toLowerCase();
    const now = Date.now();

    // Check cache first
    const cached = this.cache.get(key);
    if (cached && cached.expires_at > now) {
      return cached;
    }

    // Fetch from database
    const pool = this.getDbPool();
    let walletPermissions: any[] = [];
    
    try {
      const client = await pool.connect();
      try {
        const result = await client.query(`
          SELECT permissions, permission_groups
          FROM wallet_users 
          WHERE LOWER(wallet_address) = LOWER($1) 
            AND is_active = true
        `, [walletAddress]);
        
        walletPermissions = result.rows;
      } finally {
        client.release();
      }
    } catch (dbError) {
       
      console.error('❌ Server Cache: Database query failed:', dbError);
      walletPermissions = [];
    }
    
    // Process permissions from JSONB format
    let allPermissions: string[] = [];
    if (walletPermissions.length > 0) {
      const userData = walletPermissions[0];
      // Extract permissions from JSONB columns - permissions are objects with 'name' field
      const permissions = userData.permissions || [];
      const permissionGroups = userData.permission_groups || [];
      
      // Extract permission names from JSONB objects
      const permissionNames = permissions
        .filter((p: any) => p?.name && p.is_active !== false)
        .map((p: any) => p.name);
      
      // Extract group permissions 
      const groupPermissionNames = permissionGroups
        .flatMap((group: any) => group.permissions || [])
        .filter((p: any) => p && typeof p === 'string');
      
      // Combine individual permissions and group-based permissions
      allPermissions = [
        ...permissionNames,
        ...groupPermissionNames
      ];
    }
    
    const adminPermissions = allPermissions.filter((permission: string) => 
      permission === 'admin:*:*' || 
      permission.startsWith('admin:') ||
      permission === 'epsx:admin:*' ||
      permission === 'epsx:*:*'
    );
    
    // Determine admin level
    let adminLevel = 'none';
    if (adminPermissions.includes('admin:*:*') || adminPermissions.includes('epsx:*:*')) {
      adminLevel = 'super';
    } else if (adminPermissions.some((p: string) => p.includes('admin:web3:manage'))) {
      adminLevel = 'manager';
    } else if (adminPermissions.length > 0) {
      adminLevel = 'moderator';
    }

    // Create cache entry
    const entry: PermissionCacheEntry = {
      wallet_address: walletAddress,
      permissions: allPermissions,
      admin_level: adminLevel,
      has_admin_access: adminPermissions.length > 0,
      timestamp: now,
      expires_at: now + this.CACHE_TTL,
    };

    // Store in cache
    this.cache.set(key, entry);
    
    // Clean up expired entries periodically
    if (this.cache.size > 10) {
      this.cleanup();
    }
    
    return entry;
  }

  clearCache(walletAddress?: string): void {
    if (walletAddress) {
      const key = walletAddress.toLowerCase();
      this.cache.delete(key);
    } else {
      this.cache.clear();
    }
  }

  public cleanup(): void {
    const now = Date.now();
    for (const [key, entry] of this.cache.entries()) {
      if (entry.expires_at <= now) {
        this.cache.delete(key);
      }
    }
  }

  getStats() {
    return {
      cacheSize: this.cache.size,
      entries: Array.from(this.cache.keys()).map(key => ({
        wallet: key.slice(0, 6) + '...' + key.slice(-4),
        expires_in: this.cache.get(key)!.expires_at - Date.now(),
      })),
    };
  }

  async close(): Promise<void> {
    if (this.pool) {
      await this.pool.end();
      this.pool = null;
    }
  }
}

// Singleton instance for server-side caching
export const serverPermissionCache = new ServerPermissionCache();

// Auto cleanup every 5 minutes
setInterval(() => {
  serverPermissionCache.cleanup();
}, 300000);