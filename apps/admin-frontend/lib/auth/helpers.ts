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
    admin_modules: string[];
    permissions: string[];
    package_tier: string;
  };
  isLoggedIn: boolean;
  expiresAt?: number;
}

/**
 * Check if user has specific admin module
 */
export function hasAdminModule(
  user: SessionData['user'] | null | undefined,
  module: string
): boolean {
  if (!user?.admin_modules) return false;
  return user.admin_modules.includes(module) || user.admin_modules.includes('system_admin');
}

/**
 * Check if user is system admin
 */
export function isSystemAdmin(
  user: SessionData['user'] | null | undefined
): boolean {
  if (!user) return false;
  return hasAdminModule(user, 'system_admin') || user.role === 'super_admin';
}

/**
 * Check if user has specific permission
 */
export function hasPermission(
  user: SessionData['user'] | null | undefined,
  permission: string
): boolean {
  if (!user?.permissions) return false;
  return user.permissions.includes(permission) || user.permissions.includes('*');
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
 * Format admin modules for display
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