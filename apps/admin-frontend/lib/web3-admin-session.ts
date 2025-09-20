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
 * Get Web3 admin session from wallet authentication
 */
export async function getWeb3AdminSession(): Promise<Web3AdminSessionData | null> {
  try {
    const cookieStore = await cookies();
    const walletAddress = cookieStore.get('wallet_address')?.value;
    
    if (!walletAddress) {
      return { isAuthenticated: false };
    }

    console.log('🔍 Web3 Admin: Checking session for wallet:', walletAddress);

    // Get permissions from Web3 backend
    const response = await fetch('/api/auth/web3/permissions', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ wallet_address: walletAddress }),
    });

    if (!response.ok) {
      console.warn('⚠️ Web3 Admin: Failed to get permissions for wallet:', walletAddress);
      return { isAuthenticated: false };
    }

    const data = await response.json();
    
    // Check if wallet has admin permissions
    const allPermissions = data.permissions || [];
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
    
    console.log('✅ Web3 Admin: Session validated for wallet:', walletAddress, 'Level:', adminLevel);
    
    return {
      isAuthenticated: true,
      walletAddress,
      permissions: allPermissions,
      adminLevel,
      hasAdminAccess: true,
      expiresAt: data.timestamp ? data.timestamp + (24 * 60 * 60 * 1000) : undefined, // 24 hours
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