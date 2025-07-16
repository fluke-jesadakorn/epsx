'use server';

import { getAuthAdmin } from '@/lib/firebase-admin';
import { UserRole } from '@/types/auth/roles';

export interface CustomClaims {
  role: UserRole;
  emailVerified: boolean;
  permissions: string[];
  createdAt: number;
  lastUpdated: number;
}

/**
 * Set custom claims for a user
 */
export async function setUserCustomClaims(
  uid: string, 
  claims: Partial<CustomClaims>
): Promise<{ success: boolean; error?: string }> {
  try {
    const auth = getAuthAdmin();
    
    // Get existing claims to merge with new ones
    const userRecord = await auth.getUser(uid);
    const existingClaims = (userRecord.customClaims as CustomClaims) || {};
    
    const updatedClaims: CustomClaims = {
      role: claims.role || existingClaims.role || UserRole.USER,
      emailVerified: claims.emailVerified !== undefined ? claims.emailVerified : existingClaims.emailVerified || false,
      permissions: claims.permissions || existingClaims.permissions || [],
      createdAt: existingClaims.createdAt || Date.now(),
      lastUpdated: Date.now(),
    };

    await auth.setCustomUserClaims(uid, updatedClaims);
    
    console.log(`Custom claims updated for user ${uid}:`, updatedClaims);
    return { success: true };
  } catch (error) {
    console.error('Failed to set custom claims:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Failed to update user claims' 
    };
  }
}

/**
 * Get user custom claims
 */
export async function getUserCustomClaims(uid: string): Promise<CustomClaims | null> {
  try {
    const auth = getAuthAdmin();
    const userRecord = await auth.getUser(uid);
    return (userRecord.customClaims as CustomClaims) || null;
  } catch (error) {
    console.error('Failed to get custom claims:', error);
    return null;
  }
}

/**
 * Set user role
 */
export async function setUserRole(
  uid: string, 
  role: UserRole
): Promise<{ success: boolean; error?: string }> {
  const permissions = getPermissionsForRole(role);
  return setUserCustomClaims(uid, { role, permissions });
}

/**
 * Update email verification status
 */
export async function updateEmailVerificationStatus(
  uid: string, 
  verified: boolean
): Promise<{ success: boolean; error?: string }> {
  return setUserCustomClaims(uid, { emailVerified: verified });
}

/**
 * Get permissions for a role
 */
function getPermissionsForRole(role: UserRole): string[] {
  switch (role) {
    case UserRole.ADMIN:
      return [
        'read:all',
        'write:all',
        'delete:all',
        'admin:manage_users',
        'admin:manage_roles',
        'admin:view_analytics',
        'admin:manage_system',
        'trading:all',
        'api:all'
      ];
    case UserRole.USER:
    default:
      return [
        'read:basic',
        'write:basic',
        'trading:basic',
        'api:basic'
      ];
  }
}

/**
 * Initialize custom claims for a new user
 */
export async function initializeUserClaims(
  uid: string, 
  email: string
): Promise<{ success: boolean; error?: string }> {
  try {
    // Determine initial role based on email domain or other criteria
    const role = email.endsWith('@admin.com') || email.endsWith('@epsx.com') 
      ? UserRole.ADMIN 
      : UserRole.USER;

    const claims: CustomClaims = {
      role,
      emailVerified: false,
      permissions: getPermissionsForRole(role),
      createdAt: Date.now(),
      lastUpdated: Date.now(),
    };

    return setUserCustomClaims(uid, claims);
  } catch (error) {
    console.error('Failed to initialize user claims:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Failed to initialize user claims' 
    };
  }
}

/**
 * Check if user has permission
 */
export async function checkUserPermission(
  uid: string, 
  permission: string
): Promise<boolean> {
  try {
    const claims = await getUserCustomClaims(uid);
    if (!claims) return false;
    
    // Admin role has all permissions
    if (claims.role === UserRole.ADMIN) return true;
    
    // Check specific permission
    return claims.permissions.includes(permission) || 
           claims.permissions.includes('*') ||
           claims.permissions.some(p => p.endsWith(':all') && permission.startsWith(p.split(':')[0]));
  } catch (error) {
    console.error('Failed to check user permission:', error);
    return false;
  }
}

/**
 * Refresh user token to get updated claims
 */
export async function refreshUserToken(uid: string): Promise<{ success: boolean; error?: string }> {
  try {
    const auth = getAuthAdmin();
    
    // Revoke existing tokens to force refresh
    await auth.revokeRefreshTokens(uid);
    
    return { success: true };
  } catch (error) {
    console.error('Failed to refresh user token:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Failed to refresh token' 
    };
  }
}
