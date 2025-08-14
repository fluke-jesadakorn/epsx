/**
 * Enhanced Server-Side Authentication for Admin Frontend
 * Uses JWT-based authentication with admin-specific functions
 */
import { redirect } from 'next/navigation';
import { verifyJWTFromCookies, getSessionFromJWT } from './jwt';
import { hasAdminModuleInJWT, hasPermissionInJWT, type EPSXJWTPayload } from '@epsx/auth-shared';

/**
 * Get authenticated admin user from JWT cookies
 */
export async function getAuthUser(): Promise<EPSXJWTPayload | null> {
  try {
    return await verifyJWTFromCookies();
  } catch (error) {
    console.error('❌ Failed to get authenticated admin user:', error);
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
 * Check if user has specific admin module
 */
export async function hasAdminModule(module: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    return user.admin_modules.includes(module) || user.admin_modules.includes('system_admin');
  } catch (error) {
    console.error('❌ Failed to check admin module:', error);
    return false;
  }
}

/**
 * Require specific admin module - redirect to access denied if not found
 */
export async function requireAdminModule(module: string, redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await requireAuth(redirectPath);
  
  const hasRequiredModule = user.admin_modules.includes(module) || user.admin_modules.includes('system_admin');
  
  if (!hasRequiredModule) {
    const accessDeniedUrl = `/access-denied?module=${encodeURIComponent(module)}${redirectPath ? `&route=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(accessDeniedUrl);
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
 * Check if user is system admin
 */
export async function isSystemAdmin(): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    return user.admin_modules.includes('system_admin') || user.role === 'super_admin';
  } catch (error) {
    console.error('❌ Failed to check system admin:', error);
    return false;
  }
}

/**
 * Require system admin access
 */
export async function requireSystemAdmin(redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await requireAuth(redirectPath);
  
  const isSystemAdminUser = user.admin_modules.includes('system_admin') || user.role === 'super_admin';
  
  if (!isSystemAdminUser) {
    const accessDeniedUrl = `/access-denied?reason=system_admin_required${redirectPath ? `&route=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(accessDeniedUrl);
  }
  
  return user;
}

/**
 * Check if user can manage users
 */
export async function canManageUsers(): Promise<boolean> {
  return await hasAdminModule('user_operations') || await hasAdminModule('user_management');
}

/**
 * Require user management permissions
 */
export async function requireUserManagement(redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await requireAuth(redirectPath);
  
  const canManage = user.admin_modules.includes('user_operations') || 
                   user.admin_modules.includes('user_management') ||
                   user.admin_modules.includes('system_admin');
  
  if (!canManage) {
    const accessDeniedUrl = `/access-denied?reason=user_management_required${redirectPath ? `&route=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(accessDeniedUrl);
  }
  
  return user;
}

/**
 * Check if user can access analytics
 */
export async function canAccessAnalytics(): Promise<boolean> {
  return await hasAdminModule('analytics_specialist') || await hasAdminModule('system_admin');
}

/**
 * Require analytics access
 */
export async function requireAnalytics(redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await requireAuth(redirectPath);
  
  const canAccess = user.admin_modules.includes('analytics_specialist') ||
                   user.admin_modules.includes('system_admin');
  
  if (!canAccess) {
    const accessDeniedUrl = `/access-denied?reason=analytics_required${redirectPath ? `&route=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(accessDeniedUrl);
  }
  
  return user;
}