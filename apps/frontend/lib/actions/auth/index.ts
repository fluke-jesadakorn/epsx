'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { getBackendUrl, getFrontendUrl, oidcUrls, callbackUrls } from '@/lib/server-shared';
import { verifyJWT, type JWTUser } from '@/lib/shared';
import { logger, safeError } from '@/lib/shared';

// ============================================================================
// Types
// ============================================================================

export interface AuthUser extends JWTUser {
  id: string;
  name?: string;
  image?: string;
  tier?: string;
  level?: string;
  user_id?: string;
  emailVerified?: boolean;
}

// ============================================================================
// Authentication Server Actions
// ============================================================================

/**
 * Handle sign out action
 */
export async function handleSignOut() {
  // Clear JWT cookie
  const cookieStore = await cookies();
  cookieStore.delete('epsx_frontend_jwt');
  
  // Redirect to backend OAuth login page - NEXT_REDIRECT error is expected behavior
  const backendLoginUrl = new URL('/oauth/authorize', getBackendUrl('server'));
  backendLoginUrl.searchParams.set('client_id', 'epsx-frontend');
  backendLoginUrl.searchParams.set('redirect_uri', callbackUrls.frontend('server'));
  backendLoginUrl.searchParams.set('scope', 'openid profile email');
  backendLoginUrl.searchParams.set('response_type', 'code');
  redirect(backendLoginUrl.toString());
}

/**
 * Get current authenticated user from OIDC session
 */
export async function getCurrentUser(): Promise<AuthUser | null> {
  try {
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    
    if (!accessToken) {
      return null;
    }

    // Verify the JWT token
    const decoded = await verifyJWT(accessToken);
    
    if (!decoded) {
      return null;
    }
    
    return {
      id: decoded.sub,
      uid: decoded.sub,
      email: decoded.email,
      name: decoded.name,
      permissions: decoded.permissions || [],
      emailVerified: Boolean(decoded.email_verified),
    };
  } catch (error) {
    logger.error('Failed to get current user', { error: safeError(error).message });
    return null;
  }
}

/**
 * Check if user is authenticated
 */
export async function isAuthenticated(): Promise<boolean> {
  const user = await getCurrentUser();
  return user !== null;
}

/**
 * Get user permissions
 */
export async function getUserPermissions(): Promise<string[]> {
  const user = await getCurrentUser();
  return user?.permissions || [];
}

/**
 * Check if user has specific permission
 */
export async function hasPermission(permission: string): Promise<boolean> {
  const permissions = await getUserPermissions();
  return permissions.includes(permission);
}

/**
 * Check if user has any of the specified permissions
 */
export async function hasAnyPermission(requiredPermissions: string[]): Promise<boolean> {
  const permissions = await getUserPermissions();
  return requiredPermissions.some(permission => permissions.includes(permission));
}

/**
 * Check if user has all specified permissions
 */
export async function hasAllPermissions(requiredPermissions: string[]): Promise<boolean> {
  const permissions = await getUserPermissions();
  return requiredPermissions.every(permission => permissions.includes(permission));
}

/**
 * Require authentication - redirect to login if not authenticated
 */
export async function requireAuth(): Promise<AuthUser> {
  const user = await getCurrentUser();
  
  if (!user) {
    redirect('/login');
  }
  
  return user;
}

/**
 * Require specific permission - redirect if not authorized
 */
export async function requirePermission(permission: string, redirectPath: string = '/unauthorized'): Promise<AuthUser> {
  const user = await requireAuth();
  const hasPerms = await hasPermission(permission);
  
  if (!hasPerms) {
    redirect(redirectPath);
  }
  
  return user;
}

/**
 * Check feature access for server components
 */
export async function checkFeatureAccess(feature: string): Promise<{ hasAccess: boolean; reason: string }> {
  try {
    const user = await getCurrentUser();
    
    if (!user) {
      return { hasAccess: false, reason: 'Authentication required' };
    }
    
    // Basic feature access logic - can be expanded based on actual requirements
    switch (feature) {
      case 'api':
        return { hasAccess: true, reason: 'API access granted' };
      case 'analytics':
        return { hasAccess: user.permissions.some(p => p.includes('epsx:')), reason: 'Analytics access granted' };
      case 'admin':
        return { hasAccess: user.permissions.some(p => p.includes('admin:')), reason: 'Admin access granted' };
      default:
        return { hasAccess: true, reason: 'Default access granted' };
    }
  } catch (error) {
    logger.error('Failed to check feature access', { feature, error: safeError(error).message });
    return { hasAccess: false, reason: 'Feature access check failed' };
  }
}

// ============================================================================
// Payment Server Actions
// ============================================================================

export interface PaymentTransaction {
  orderNo: string;
  amount: number;
  currency: string;
  status: string;
  finishTime: string;
  blockchainData: {
    txHash: string;
    network: string;
  };
  blockExplorerUrl: string;
}

/**
 * Get transaction history for current user
 */
export async function getTransactionHistory(excludePending = false): Promise<PaymentTransaction[]> {
  try {
    const user = await getCurrentUser();
    if (!user) {
      return [];
    }

    // Mock transaction data for now - TODO: implement real API call
    const mockTransactions: PaymentTransaction[] = [
      {
        orderNo: 'ORDER_001',
        amount: 29.99,
        currency: 'USDT',
        status: 'completed',
        finishTime: new Date().toISOString(),
        blockchainData: {
          txHash: '0x1234567890abcdef1234567890abcdef12345678',
          network: 'ethereum'
        },
        blockExplorerUrl: 'https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef12345678'
      }
    ];

    if (excludePending) {
      return mockTransactions.filter(tx => tx.status === 'completed');
    }

    return mockTransactions;
  } catch (error) {
    logger.error('Failed to get transaction history', { error: safeError(error).message });
    return [];
  }
}