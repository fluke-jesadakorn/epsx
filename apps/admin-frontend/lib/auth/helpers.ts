/**
 * Client-side Authentication Helpers
 * Utility functions for authentication logic
 */

'use client';

// Client-safe session data type
export interface SessionData {
  user?: {
    id: string;
    email: string;
    name: string;
    role: string;
    firebase_uid?: string;
    permissions: string[];
    platform_context?: string;
    primary_platform?: string;
    package_tier: string;
  };
  isLoggedIn: boolean;
  expiresAt?: number;
}

/**
 * Check if user has specific admin module (deprecated - use hasPermission instead)
 * @deprecated Use hasPermission instead
 */
export function hasAdminModule(
  user: SessionData['user'] | null | undefined,
  module: string
): boolean {
  if (!user) return false;
  
  // Convert legacy module to structured permission
  const modulePermissionMap: Record<string, string> = {
    'system_admin': '*',
    'user_management': 'epsx:users:manage',
    'analytics_specialist': 'epsx:analytics:view',
    'billing_admin': 'epsx:billing:manage',
    'permission_admin': 'epsx:permissions:manage',
    'package_coordinator': 'epsx:packages:manage',
  };
  
  const permission = modulePermissionMap[module];
  return permission ? hasPermission(user, permission) : false;
}

/**
 * Check if user is system admin using structured permissions only
 */
export function isSystemAdmin(
  user: SessionData['user'] | null | undefined
): boolean {
  if (!user) return false;
  
  // Check for admin wildcard permission
  if (user.permissions?.includes('admin:*:*')) return true;
  
  // Check for legacy wildcard permission (for backward compatibility)
  if (user.permissions?.includes('*')) return true;
  
  return false;
}

/**
 * Check if user has specific permission using structured permission system
 */
export function hasPermission(
  user: SessionData['user'] | null | undefined,
  permission: string
): boolean {
  if (!user?.permissions) return false;
  
  // Check for exact permission match
  if (user.permissions.includes(permission)) return true;
  
  // Check for admin wildcard permission
  if (user.permissions.includes('admin:*:*')) return true;
  
  // Check for legacy wildcard permission
  if (user.permissions.includes('*')) return true;
  
  // Check for broader permissions (e.g., admin:users:* covers admin:users:view)
  if (permission.includes(':')) {
    const [platform, resource] = permission.split(':');
    return user.permissions.some(p => 
      p === `${platform}:${resource}:*` || 
      p === `${platform}:*:*`
    );
  }
  
  return false;
}

/**
 * Get user display name
 */
export function getUserDisplayName(
  user: SessionData['user'] | null | undefined
): string {
  if (!user) return 'Unknown User';
  return user.name || user.email.split('@')[0] || 'User';
}

/**
 * Format admin modules for display (deprecated - use formatPermissions instead)
 */
export function formatAdminModules(modules: string[]): string[] {
  const moduleLabels: Record<string, string> = {
    'system_admin': 'System Admin',
    'user_management': 'User Management',
    'user_operations': 'User Operations',
    'analytics_specialist': 'Analytics',
    'billing_admin': 'Billing',
    'permission_admin': 'Permission Management',
    'package_coordinator': 'Package Coordinator'
  };
  
  return modules.map(module => moduleLabels[module] || module);
}

/**
 * Format permissions for display
 */
export function formatPermissions(permissions: string[]): string[] {
  return permissions.map(permission => {
    if (permission === '*') return 'System Admin';
    
    const [platform, resource, action] = permission.split(':');
    if (!platform || !resource || !action) return permission;
    
    const platformLabel = platform.toUpperCase();
    const resourceLabel = resource.charAt(0).toUpperCase() + resource.slice(1);
    const actionLabel = action.charAt(0).toUpperCase() + action.slice(1);
    
    return `${platformLabel} ${resourceLabel} ${actionLabel}`;
  });
}

/**
 * Check if user has platform-specific permission
 */
export function hasPlatformPermission(
  user: SessionData['user'] | null | undefined,
  resource: string,
  action: string,
  platform?: string
): boolean {
  if (!user) return false;
  
  const targetPlatform = platform || user.platform_context || user.primary_platform || 'epsx';
  const permission = `${targetPlatform}:${resource}:${action}`;
  
  return hasPermission(user, permission);
}