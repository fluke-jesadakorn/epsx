/**
 * Enhanced Server-Side Authentication for Frontend
 * Uses JWT-based authentication with jose library
 */
import { redirect } from 'next/navigation';
import { verifyJWTFromCookies, getSessionFromJWT } from './jwt';
import { hasPermissionInJWT, hasPackageTierInJWT, type EPSXJWTPayload } from '@epsx/auth-shared';

/**
 * Get authenticated user from JWT cookies
 */
export async function getAuthUser(): Promise<EPSXJWTPayload | null> {
  try {
    return await verifyJWTFromCookies();
  } catch (error) {
    console.error('❌ Failed to get authenticated user:', error);
    return null;
  }
}

/**
 * Require authentication - redirect to login if not authenticated
 */
export async function requireAuth(redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await getAuthUser();
  
  if (!user) {
    const loginUrl = `/login${redirectPath ? `?callbackUrl=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(loginUrl);
  }
  
  return user;
}

/**
 * Check if user has specific permission
 */
export async function hasPermission(permission: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    return user.permissions.includes(permission) || user.permissions.includes('*');
  } catch (error) {
    console.error('❌ Failed to check permission:', error);
    return false;
  }
}

/**
 * Require specific permission - redirect to access denied if not found
 */
export async function requirePermission(permission: string, redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await requireAuth(redirectPath);
  
  const hasRequiredPermission = user.permissions.includes(permission) || user.permissions.includes('*');
  
  if (!hasRequiredPermission) {
    const accessDeniedUrl = `/access-denied?permission=${encodeURIComponent(permission)}${redirectPath ? `&route=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(accessDeniedUrl);
  }
  
  return user;
}

/**
 * Check if user has specific package tier or higher
 */
export async function hasPackageTier(requiredTier: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    const tierHierarchy: Record<string, number> = {
      'FREE': 1,
      'BRONZE': 2,
      'SILVER': 3,
      'GOLD': 4,
      'PLATINUM': 5,
      'ENTERPRISE': 6
    };
    
    const userLevel = tierHierarchy[user.package_tier] || 0;
    const requiredLevel = tierHierarchy[requiredTier] || 1;
    
    return userLevel >= requiredLevel;
  } catch (error) {
    console.error('❌ Failed to check package tier:', error);
    return false;
  }
}

/**
 * Require specific package tier - redirect to upgrade if not found
 */
export async function requirePackageTier(requiredTier: string, redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await requireAuth(redirectPath);
  
  const hasRequiredTier = await hasPackageTier(requiredTier);
  
  if (!hasRequiredTier) {
    const upgradeUrl = `/payment?tier=${encodeURIComponent(requiredTier)}${redirectPath ? `&callbackUrl=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(upgradeUrl);
  }
  
  return user;
}

/**
 * Check if user has specific role
 */
export async function hasRole(requiredRole: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    const roleHierarchy: Record<string, number> = {
      'user': 1,
      'premium': 2,
      'moderator': 3,
      'admin': 4,
      'super_admin': 5
    };
    
    const userLevel = roleHierarchy[user.role.toLowerCase()] || 0;
    const requiredLevel = roleHierarchy[requiredRole.toLowerCase()] || 1;
    
    return userLevel >= requiredLevel;
  } catch (error) {
    console.error('❌ Failed to check role:', error);
    return false;
  }
}

/**
 * Require specific role - redirect to access denied if not found
 */
export async function requireRole(requiredRole: string, redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await requireAuth(redirectPath);
  
  const hasRequiredRole = await hasRole(requiredRole);
  
  if (!hasRequiredRole) {
    const accessDeniedUrl = `/access-denied?role=${encodeURIComponent(requiredRole)}${redirectPath ? `&route=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(accessDeniedUrl);
  }
  
  return user;
}