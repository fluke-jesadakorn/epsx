/**
 * Modern Authentication Service - Clean Admin Module System
 * Completely replaces legacy role-based authentication
 */

'use server'

import { cookies } from 'next/headers'
import { redirect } from 'next/navigation'
import { NextRequest, NextResponse } from 'next/server'

// Modern user interface - admin modules only
export interface ModernAuthUser {
  user_id: string
  email: string
  name?: string
  admin: boolean
  access_level: 'admin' | 'write' | 'read' | 'none'
  admin_modules: string[]
  permissions: string[]
  subscription_tier?: string
  subscription_status?: string
}

// Admin module definitions
export const ADMIN_MODULES = {
  USER_OPERATIONS: 'user_operations',
  PERMISSION_ADMIN: 'permission_admin', 
  ROLE_POLICY_MANAGER: 'role_policy_manager',
  ANALYTICS_SPECIALIST: 'analytics_specialist',
  BILLING_ADMIN: 'billing_admin',
  SYSTEM_ADMIN: 'system_admin',
  DEVELOPER_RELATIONS: 'developer_relations',
  MODULE_COORDINATOR: 'module_coordinator',
  COMPLIANCE_AUDIT: 'compliance_audit',
  SUPPORT_SPECIALIST: 'support_specialist'
} as const

export type AdminModule = typeof ADMIN_MODULES[keyof typeof ADMIN_MODULES]

// Admin module capabilities mapping
export const MODULE_CAPABILITIES = {
  [ADMIN_MODULES.USER_OPERATIONS]: {
    permissions: ['user:read', 'user:write', 'user:status', 'profile:edit'],
    description: 'User CRUD operations and profile management'
  },
  [ADMIN_MODULES.PERMISSION_ADMIN]: {
    permissions: ['permission:read', 'permission:write', 'profile:assign'],
    description: 'Permission profiles and assignments'
  },
  [ADMIN_MODULES.ANALYTICS_SPECIALIST]: {
    permissions: ['analytics:read', 'reports:generate', 'metrics:view'],
    description: 'Analytics and reporting access'
  },
  [ADMIN_MODULES.BILLING_ADMIN]: {
    permissions: ['billing:read', 'billing:write', 'subscription:manage'],
    description: 'Billing and subscription management'
  },
  [ADMIN_MODULES.SYSTEM_ADMIN]: {
    permissions: ['database:admin', 'system:settings', 'cache:manage'],
    description: 'Full system administration'
  }
} as const

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080'
const AUTH_COOKIE_NAME = 'modern_admin_token'

/**
 * Modern Authentication Class - Pure Admin Module System
 */
export class ModernAuthService {
  
  /**
   * Get current authenticated user with modern admin module data
   */
  static async getCurrentUser(): Promise<ModernAuthUser | null> {
    try {
      const token = await this.getBearerToken()
      if (!token) return null

      // Validate token with backend
      const response = await fetch(`${BACKEND_URL}/api/v1/auth/me`, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        }
      })

      if (!response.ok) {
        this.clearAuth()
        return null
      }

      const userData = await response.json()
      
      // Ensure we have modern admin module structure
      return {
        user_id: userData.sub || userData.user_id,
        email: userData.email,
        name: userData.name,
        admin: userData.admin || false,
        access_level: userData.access_level || 'none',
        admin_modules: userData.admin_modules || [],
        permissions: userData.permissions || [],
        subscription_tier: userData.subscription_tier,
        subscription_status: userData.subscription_status
      }
    } catch (error) {
      console.error('Failed to get current user:', error)
      return null
    }
  }

  /**
   * Check if user has specific admin module
   */
  static async hasAdminModule(module: AdminModule): Promise<boolean> {
    const user = await this.getCurrentUser()
    return user?.admin_modules.includes(module) || false
  }

  /**
   * Check if user has any admin modules (is an admin)
   */
  static async isAdmin(): Promise<boolean> {
    const user = await this.getCurrentUser()
    return user?.admin && user.admin_modules.length > 0 || false
  }

  /**
   * Check if user has specific permission
   */
  static async hasPermission(permission: string): Promise<boolean> {
    const user = await this.getCurrentUser()
    return user?.permissions.includes(permission) || false
  }

  /**
   * Get user's effective access level
   */
  static async getAccessLevel(): Promise<'admin' | 'write' | 'read' | 'none'> {
    const user = await this.getCurrentUser()
    return user?.access_level || 'none'
  }

  /**
   * Modern capability checks based on admin modules
   */
  static async canManageUsers(): Promise<boolean> {
    return await this.hasAdminModule(ADMIN_MODULES.USER_OPERATIONS) ||
           await this.hasAdminModule(ADMIN_MODULES.SYSTEM_ADMIN) ||
           await this.hasPermission('user:write')
  }

  static async canViewAnalytics(): Promise<boolean> {
    return await this.hasAdminModule(ADMIN_MODULES.ANALYTICS_SPECIALIST) ||
           await this.hasAdminModule(ADMIN_MODULES.SYSTEM_ADMIN) ||
           await this.hasPermission('analytics:read')
  }

  static async canManageBilling(): Promise<boolean> {
    return await this.hasAdminModule(ADMIN_MODULES.BILLING_ADMIN) ||
           await this.hasAdminModule(ADMIN_MODULES.SYSTEM_ADMIN) ||
           await this.hasPermission('billing:write')
  }

  static async canManagePermissions(): Promise<boolean> {
    return await this.hasAdminModule(ADMIN_MODULES.PERMISSION_ADMIN) ||
           await this.hasAdminModule(ADMIN_MODULES.SYSTEM_ADMIN) ||
           await this.hasPermission('permission:write')
  }

  /**
   * Require authentication - redirect if not authenticated
   */
  static async requireAuth(): Promise<ModernAuthUser> {
    const user = await this.getCurrentUser()
    if (!user) {
      redirect('/login')
    }
    return user
  }

  /**
   * Require admin access - redirect if not admin
   */
  static async requireAdmin(): Promise<ModernAuthUser> {
    const user = await this.requireAuth()
    if (!user.admin || user.admin_modules.length === 0) {
      redirect('/unauthorized')
    }
    return user
  }

  /**
   * Require specific admin module
   */
  static async requireAdminModule(module: AdminModule): Promise<ModernAuthUser> {
    const user = await this.requireAuth()
    if (!user.admin_modules.includes(module)) {
      redirect('/access-denied?required=' + module)
    }
    return user
  }

  /**
   * Require specific permission
   */
  static async requirePermission(permission: string): Promise<ModernAuthUser> {
    const user = await this.requireAuth()
    if (!user.permissions.includes(permission)) {
      redirect('/access-denied?permission=' + permission)
    }
    return user
  }

  /**
   * Modern login using OIDC flow
   */
  static async login(email: string, password: string): Promise<{ success: boolean; error?: string }> {
    try {
      // Use OIDC authorization code flow
      const authResponse = await fetch(`${BACKEND_URL}/oauth/token`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
          grant_type: 'authorization_code',
          code: password, // In development, password acts as code
          client_id: 'epsx-admin',
          redirect_uri: `${process.env.NEXTAUTH_URL}/auth/callback`
        })
      })

      if (!authResponse.ok) {
        return { success: false, error: 'Invalid credentials' }
      }

      const tokenData = await authResponse.json()
      
      // Store the access token
      await this.setAuthToken(tokenData.access_token)
      
      return { success: true }
    } catch (error) {
      console.error('Login error:', error)
      return { success: false, error: 'Login failed' }
    }
  }

  /**
   * Logout - clear auth data
   */
  static async logout(): Promise<void> {
    this.clearAuth()
    redirect('/login')
  }

  /**
   * Get bearer token from cookie
   */
  private static async getBearerToken(): Promise<string | null> {
    const cookieStore = await cookies()
    return cookieStore.get(AUTH_COOKIE_NAME)?.value || null
  }

  /**
   * Set auth token in cookie
   */
  private static async setAuthToken(token: string): Promise<void> {
    const cookieStore = await cookies()
    cookieStore.set(AUTH_COOKIE_NAME, token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 8 * 60 * 60, // 8 hours
      path: '/'
    })
  }

  /**
   * Clear authentication
   */
  private static clearAuth(): void {
    const cookieStore = cookies()
    cookieStore.delete(AUTH_COOKIE_NAME)
  }

  /**
   * Middleware helper for protecting routes
   */
  static async authMiddleware(request: NextRequest): Promise<NextResponse | null> {
    const token = request.cookies.get(AUTH_COOKIE_NAME)?.value

    if (!token) {
      return NextResponse.redirect(new URL('/login', request.url))
    }

    // Validate token with backend
    try {
      const response = await fetch(`${BACKEND_URL}/api/v1/auth/me`, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        }
      })

      if (!response.ok) {
        const loginUrl = new URL('/login', request.url)
        const response = NextResponse.redirect(loginUrl)
        response.cookies.delete(AUTH_COOKIE_NAME)
        return response
      }

      // Add user context to request headers
      const userData = await response.json()
      const requestHeaders = new Headers(request.headers)
      requestHeaders.set('x-user-id', userData.sub)
      requestHeaders.set('x-user-admin', userData.admin.toString())
      requestHeaders.set('x-user-modules', userData.admin_modules.join(','))

      return NextResponse.next({
        request: {
          headers: requestHeaders
        }
      })
    } catch (error) {
      console.error('Auth middleware error:', error)
      return NextResponse.redirect(new URL('/login', request.url))
    }
  }
}

/**
 * Convenience functions for common patterns
 */
export const auth = ModernAuthService
export const { 
  getCurrentUser, 
  hasAdminModule, 
  hasPermission, 
  isAdmin,
  requireAuth, 
  requireAdmin, 
  requireAdminModule,
  canManageUsers,
  canViewAnalytics,
  canManageBilling
} = ModernAuthService