import { cookies } from 'next/headers';

export interface Web3AdminSessionData {
  isAuthenticated: boolean;
  walletAddress?: string;
  permissions?: string[];
  adminLevel?: 'none' | 'moderator' | 'manager' | 'super';
  hasAdminAccess?: boolean;
  expiresAt?: number;
}

/**
 * Get Web3 admin session - Simple wallet address + database check
 * No complex signature validation, just wallet address and permissions
 */
export async function getWeb3AdminSession(): Promise<Web3AdminSessionData | null> {
  try {
    const cookieStore = await cookies();
    const walletAddress = cookieStore.get('wallet_address')?.value;
    
    if (!walletAddress) {
      console.log('🔍 Web3 Admin: No wallet address found in cookies');
      return { isAuthenticated: false };
    }

    console.log('🔍 Web3 Admin: Checking session for wallet:', walletAddress);

    // Simple database permission check - no complex validation
    const { Pool } = require('pg');
    const databaseUrl = process.env.DATABASE_URL || 'postgresql://postgres:password@localhost:5432/epsx_db';
    const pool = new Pool({
      connectionString: databaseUrl,
      max: 5,
      idleTimeoutMillis: 30000,
      connectionTimeoutMillis: 2000,
      native: false
    });
    
    let walletPermissions: any[] = [];
    
    try {
      const client = await pool.connect();
      try {
        const result = await client.query(`
          SELECT permission, permission_type, is_active, expires_at, granted_at
          FROM wallet_permissions 
          WHERE wallet_address = $1 
            AND is_active = true 
            AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
          ORDER BY granted_at DESC
        `, [walletAddress.toLowerCase()]);
        
        walletPermissions = result.rows;
        console.log('✅ Web3 Admin: Found', walletPermissions.length, 'permissions for wallet:', walletAddress);
      } finally {
        client.release();
      }
    } catch (dbError) {
      console.error('❌ Web3 Admin: Database query failed:', dbError);
      return { isAuthenticated: false };
    } finally {
      await pool.end();
    }
    
    const allPermissions = walletPermissions.map((p: any) => p.permission);
    
    // Check if wallet has admin permissions
    const adminPermissions = allPermissions.filter((permission: string) => 
      permission === 'admin:*:*' || 
      permission.startsWith('admin:') ||
      permission === 'epsx:admin:*'
    );
    
    const hasAdminAccess = adminPermissions.length > 0;
    
    if (!hasAdminAccess) {
      console.warn('❌ Web3 Admin: Wallet has no admin permissions:', walletAddress);
      return { isAuthenticated: false };
    }

    // Determine admin level
    let adminLevel: 'none' | 'moderator' | 'manager' | 'super' = 'none';
    if (adminPermissions.includes('admin:*:*')) {
      adminLevel = 'super';
    } else if (adminPermissions.some((p: string) => p.includes('admin:web3:manage'))) {
      adminLevel = 'manager';
    } else if (adminPermissions.length > 0) {
      adminLevel = 'moderator';
    }
    
    console.log('✅ Web3 Admin: Simple authentication successful for wallet:', walletAddress, 'Level:', adminLevel);
    
    return {
      isAuthenticated: true,
      walletAddress,
      permissions: allPermissions,
      adminLevel,
      hasAdminAccess: true,
      expiresAt: Date.now() + (24 * 60 * 60 * 1000), // 24 hours
    };
    
  } catch (error) {
    console.error('💥 Web3 Admin: Failed to get admin session:', error);
    return { isAuthenticated: false };
  }
}

/**
 * Check if current session has specific admin permission
 */
export async function hasWeb3AdminPermission(permission: string): Promise<boolean> {
  try {
    const session = await getWeb3AdminSession();
    
    if (!session?.isAuthenticated || !session.permissions) {
      return false;
    }

    // Check for exact match or wildcard permissions
    return session.permissions.some(p => 
      p === permission ||
      p === 'admin:*:*' ||
      (permission.startsWith('admin:') && p === 'admin:*:*') ||
      (permission.includes(':') && p === permission.split(':').slice(0, 2).join(':') + ':*')
    );
    
  } catch (error) {
    console.error('💥 Web3 Admin: Permission check failed:', error);
    return false;
  }
}

/**
 * Create Web3 admin user object compatible with existing components
 */
export function createWeb3AdminUser(sessionData: Web3AdminSessionData) {
  if (!sessionData.isAuthenticated || !sessionData.walletAddress) {
    return null;
  }

  return {
    id: sessionData.walletAddress,
    email: `${sessionData.walletAddress}@web3.epsx.io`, // Virtual email for compatibility
    name: `Admin (${sessionData.walletAddress.slice(0, 6)}...${sessionData.walletAddress.slice(-4)})`,
    displayName: `Web3 Admin ${sessionData.walletAddress.slice(0, 6)}...${sessionData.walletAddress.slice(-4)}`,
    role: sessionData.adminLevel === 'super' ? 'admin' : 'moderator',
    walletAddress: sessionData.walletAddress,
    permissions: sessionData.permissions || [],
    isActive: true,
    hasAdminAccess: sessionData.hasAdminAccess,
    adminLevel: sessionData.adminLevel,
    authMethod: 'web3',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    lastLoginAt: new Date().toISOString(),
  };
}