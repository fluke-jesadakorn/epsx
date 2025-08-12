/**
 * Enhanced Server Auth Utilities for Server Components
 * Builds on existing bearer token auth system with additional utilities
 */

'use server'

import { redirect } from 'next/navigation'
import { getCurrentUser } from '@/lib/actions/server-auth'
import type { AuthUser } from '@/lib/actions/server-auth'
import { FEATURE_FLAGS } from '@/lib/feature-flags'

// Enhanced user info with additional fields
export interface EnhancedAuthUser extends AuthUser {
  displayName?: string
  isAdmin: boolean
  isSuperAdmin: boolean
  canManageUsers: boolean
  canManageBilling: boolean
  canViewAnalytics: boolean
}

/**
 * Require authentication - redirect to login if not authenticated
 */
export async function requireAuth(): Promise<EnhancedAuthUser> {
  const user = await getCurrentUser()
  
  if (!user) {
    redirect('/login')
  }
  
  return enhanceUserData(user)
}

/**
 * Require admin authentication with role checking
 */
export async function requireAdminAuth(): Promise<EnhancedAuthUser> {
  const user = await requireAuth()
  
  if (!user.isAdmin) {
    redirect('/unauthorized')
  }
  
  return user
}

/**
 * Get current authenticated user (returns null if not authenticated)
 */
export async function getAuthUser(): Promise<EnhancedAuthUser | null> {
  const user = await getCurrentUser()
  
  if (!user) {
    return null
  }
  
  return enhanceUserData(user)
}

/**
 * Check if user has specific permission
 */
export async function hasPermission(permission: string): Promise<boolean> {
  const user = await getCurrentUser()
  
  if (!user) {
    return false
  }
  
  return user.permissions.includes(permission)
}

/**
 * Check if user has specific admin module (modern approach)
 */
export async function hasAdminModule(adminModule: string): Promise<boolean> {
  const user = await getCurrentUser()
  
  if (!user || !user.admin_modules) {
    return false
  }
  
  return user.admin_modules.includes(adminModule)
}

/**
 * Check if user has any admin modules (is an admin)
 */
export async function hasAnyAdminModule(): Promise<boolean> {
  const user = await getCurrentUser()
  
  if (!user || !user.admin_modules) {
    return false
  }
  
  return user.admin_modules.length > 0
}

// Legacy role functions completely removed - use admin modules only

/**
 * Check if user can manage other users (using modern admin modules)
 */
export async function canManageUsers(): Promise<boolean> {
  const user = await getCurrentUser()
  
  if (!user) {
    return false
  }
  
  // Check if user has user management admin module
  const hasUserOpsModule = user.admin_modules?.includes('user_operations') || false
  const hasPermissionModule = user.admin_modules?.includes('permission_admin') || false
  const hasSystemModule = user.admin_modules?.includes('system_admin') || false
  
  // Also check for explicit permission
  const hasUserPermission = user.permissions.includes('user:write') || user.permissions.includes('manage_users')
  
  return hasUserOpsModule || hasPermissionModule || hasSystemModule || hasUserPermission
}

/**
 * Check if user can view analytics (using modern admin modules)
 */
export async function canViewAnalytics(): Promise<boolean> {
  const user = await getCurrentUser()
  
  if (!user) {
    return false
  }
  
  // Check if user has analytics admin module
  const hasAnalyticsModule = user.admin_modules?.includes('analytics_specialist') || false
  const hasSystemModule = user.admin_modules?.includes('system_admin') || false
  
  // Also check for explicit permission
  const hasAnalyticsPermission = user.permissions.includes('analytics:read') || user.permissions.includes('view_analytics')
  
  return hasAnalyticsModule || hasSystemModule || hasAnalyticsPermission
}

/**
 * Check if user can manage billing (using modern admin modules)
 */
export async function canManageBilling(): Promise<boolean> {
  const user = await getCurrentUser()
  
  if (!user) {
    return false
  }
  
  // Check if user has billing admin module
  const hasBillingModule = user.admin_modules?.includes('billing_admin') || false
  const hasSystemModule = user.admin_modules?.includes('system_admin') || false
  
  // Also check for explicit permission
  const hasBillingPermission = user.permissions.includes('billing:write') || user.permissions.includes('manage_billing')
  
  return hasBillingModule || hasSystemModule || hasBillingPermission
}

/**
 * Require specific permission or redirect to access denied
 */
export async function requirePermission(permission: string): Promise<EnhancedAuthUser> {
  const user = await requireAuth()
  
  if (!user.permissions.includes(permission)) {
    redirect('/access-denied')
  }
  
  return user
}

/**
 * Require specific admin module or redirect to access denied
 */
export async function requireAdminModule(adminModule: string): Promise<EnhancedAuthUser> {
  const user = await requireAuth()
  
  if (!user.admin_modules?.includes(adminModule)) {
    redirect('/access-denied')
  }
  
  return user
}

// Legacy requireRole function completely removed

/**
 * Enhanced user data with computed permissions using modern admin modules
 */
function enhanceUserData(user: AuthUser): EnhancedAuthUser {
  // Modern approach: check admin modules instead of hardcoded roles
  const hasAnyAdminModules = user.admin_modules && user.admin_modules.length > 0
  const hasSystemAdminModule = user.admin_modules?.includes('system_admin') || false
  const isAdmin = hasAnyAdminModules || user.permissions.includes('admin_access')
  const isSuperAdmin = hasSystemAdminModule // System admin is the highest level
  
  // Compute capabilities based on admin modules
  const hasUserOpsModule = user.admin_modules?.includes('user_operations') || false
  const hasPermissionModule = user.admin_modules?.includes('permission_admin') || false
  const hasBillingModule = user.admin_modules?.includes('billing_admin') || false
  const hasAnalyticsModule = user.admin_modules?.includes('analytics_specialist') || false
  
  return {
    ...user,
    displayName: user.email, // Can be enhanced later with actual display name
    isAdmin,
    isSuperAdmin,
    canManageUsers: hasUserOpsModule || hasPermissionModule || hasSystemAdminModule || user.permissions.includes('user:write'),
    canManageBilling: hasBillingModule || hasSystemAdminModule || user.permissions.includes('billing:write'), 
    canViewAnalytics: hasAnalyticsModule || hasSystemAdminModule || user.permissions.includes('analytics:read'),
  }
}

/**
 * Server Component auth context provider
 * Use this in layouts to provide auth data to child components
 */
export async function getServerAuthContext() {
  const user = await getAuthUser()
  
  return {
    user,
    isAuthenticated: !!user,
    isAdmin: user?.isAdmin || false,
    permissions: user?.permissions || [],
    featureFlags: FEATURE_FLAGS,
  }
}

/**
 * Auth check for Server Actions
 * Use at the beginning of server actions that require auth
 */
export async function validateServerActionAuth(requireAdmin = false): Promise<EnhancedAuthUser> {
  const user = requireAdmin ? await requireAdminAuth() : await requireAuth()
  
  return user
}

/**
 * User context for data fetching
 * Provides user context for server-side data operations
 */
export async function getUserContext() {
  const user = await getAuthUser()
  
  if (!user) {
    return null
  }
  
  return {
    userId: user.user_id,
    role: user.role,
    permissions: user.permissions,
    subscriptionTier: user.subscription_tier,
    isAdmin: user.isAdmin,
  }
}