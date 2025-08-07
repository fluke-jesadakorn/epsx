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
 * Check if user has specific role
 */
export async function hasRole(role: string): Promise<boolean> {
  const user = await getCurrentUser()
  
  if (!user) {
    return false
  }
  
  return user.role === role
}

/**
 * Check if user can manage other users
 */
export async function canManageUsers(): Promise<boolean> {
  const user = await getCurrentUser()
  
  if (!user) {
    return false
  }
  
  const adminRoles = ['admin', 'system_administrator', 'super_admin']
  const hasAdminRole = adminRoles.includes(user.role)
  const hasUserPermission = user.permissions.includes('manage_users')
  
  return hasAdminRole || hasUserPermission
}

/**
 * Check if user can view analytics
 */
export async function canViewAnalytics(): Promise<boolean> {
  const user = await getCurrentUser()
  
  if (!user) {
    return false
  }
  
  const analyticsRoles = ['admin', 'system_administrator', 'super_admin', 'moderator']
  const hasAnalyticsRole = analyticsRoles.includes(user.role)
  const hasAnalyticsPermission = user.permissions.includes('view_analytics')
  
  return hasAnalyticsRole || hasAnalyticsPermission
}

/**
 * Check if user can manage billing
 */
export async function canManageBilling(): Promise<boolean> {
  const user = await getCurrentUser()
  
  if (!user) {
    return false
  }
  
  const billingRoles = ['admin', 'system_administrator', 'super_admin']
  const hasBillingRole = billingRoles.includes(user.role)
  const hasBillingPermission = user.permissions.includes('manage_billing')
  
  return hasBillingRole || hasBillingPermission
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
 * Require specific role or redirect to access denied  
 */
export async function requireRole(role: string): Promise<EnhancedAuthUser> {
  const user = await requireAuth()
  
  if (user.role !== role) {
    redirect('/access-denied')
  }
  
  return user
}

/**
 * Enhanced user data with computed permissions
 */
function enhanceUserData(user: AuthUser): EnhancedAuthUser {
  const adminRoles = ['admin', 'system_administrator', 'super_admin']
  const isAdmin = adminRoles.includes(user.role) || user.permissions.includes('admin_access')
  const isSuperAdmin = user.role === 'super_admin'
  
  return {
    ...user,
    displayName: user.email, // Can be enhanced later with actual display name
    isAdmin,
    isSuperAdmin,
    canManageUsers: isAdmin || user.permissions.includes('manage_users'),
    canManageBilling: isAdmin || user.permissions.includes('manage_billing'), 
    canViewAnalytics: isAdmin || user.permissions.includes('view_analytics'),
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