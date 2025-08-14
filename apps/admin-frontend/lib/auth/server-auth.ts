/**
 * Server-side authentication and user context
 * Provides user context and admin auth requirements
 */

'use server'

import { headers } from 'next/headers'

export interface EnhancedAuthUser {
  id: string
  email: string
  displayName?: string
  role: string
  permissions: string[]
  isAdmin: boolean
  emailVerified: boolean
  createdAt: Date
  lastLogin?: Date
}

export interface UserContext {
  user: EnhancedAuthUser
  session: {
    token: string
    expiresAt: Date
  }
}

/**
 * Get the current user context from session
 */
export async function getUserContext(): Promise<UserContext | null> {
  try {
    // For development, return a mock admin user
    if (process.env.NODE_ENV === 'development') {
      return {
        user: {
          id: 'dev-admin-001',
          email: 'admin@epsx.dev',
          displayName: 'Development Admin',
          role: 'admin',
          permissions: ['admin:read', 'admin:write', 'admin:delete'],
          isAdmin: true,
          emailVerified: true,
          createdAt: new Date(),
          lastLogin: new Date()
        },
        session: {
          token: 'dev-admin-token',
          expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000) // 24 hours
        }
      }
    }

    // TODO: Implement proper session management
    // This should:
    // 1. Extract session token from cookies or headers
    // 2. Validate the token with the backend
    // 3. Return user data if valid

    return null
  } catch (error) {
    console.error('Failed to get user context:', error)
    return null
  }
}

/**
 * Require admin authentication - throws if not admin
 */
export async function requireAdminAuth(): Promise<EnhancedAuthUser> {
  const context = await getUserContext()
  
  if (!context) {
    throw new Error('Authentication required')
  }

  if (!context.user.isAdmin) {
    throw new Error('Admin access required')
  }

  return context.user
}

/**
 * Check if current user is authenticated and has admin role
 */
export async function isAdminAuthenticated(): Promise<boolean> {
  try {
    const context = await getUserContext()
    return context?.user.isAdmin ?? false
  } catch {
    return false
  }
}

/**
 * Get current user ID if authenticated
 */
export async function getCurrentUserId(): Promise<string | null> {
  try {
    const context = await getUserContext()
    return context?.user.id ?? null
  } catch {
    return null
  }
}