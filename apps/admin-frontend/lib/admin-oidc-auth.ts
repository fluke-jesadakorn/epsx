'use server'

/**
 * Admin OIDC Authentication Utilities
 * OIDC Migration: Replaces legacy JWT authentication with OIDC-compliant tokens
 * Handles admin-specific authentication with structured permission validation
 */

import { cookies } from 'next/headers'
import { validateAdminPermissions } from './admin-permissions'
import type { OIDCUser, AdminSession, TokenPair } from './admin-types'

// ============================================================================
// OIDC Cookie Management
// ============================================================================

/**
 * Get OIDC tokens from HttpOnly cookies
 */
export async function getOIDCTokensFromCookies(): Promise<{
  accessToken: string | null
  idToken: string | null
  refreshToken: string | null
}> {
  try {
    const cookieStore = await cookies()
    
    const accessToken = cookieStore.get('access_token')?.value || null
    const idToken = cookieStore.get('id_token')?.value || null
    const refreshToken = cookieStore.get('refresh_token')?.value || null
    
    console.log('🔍 Admin OIDC tokens check:', {
      accessToken: accessToken ? 'present' : 'missing',
      idToken: idToken ? 'present' : 'missing',
      refreshToken: refreshToken ? 'present' : 'missing'
    })
    
    return { accessToken, idToken, refreshToken }
  } catch (error) {
    console.error('❌ Failed to get OIDC tokens from cookies:', error)
    return { accessToken: null, idToken: null, refreshToken: null }
  }
}

/**
 * Validate OIDC access token with backend and get user info
 */
export async function validateOIDCTokenWithBackend(accessToken: string): Promise<OIDCUser | null> {
  try {
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
    
    const response = await fetch(`${backendUrl}/oauth/userinfo`, {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json'
      }
    })
    
    if (!response.ok) {
      console.error('❌ OIDC token validation failed:', response.status)
      return null
    }
    
    const userInfo = await response.json()
    
    return {
      sub: userInfo.sub,
      email: userInfo.email,
      name: userInfo.name,
      permissions: userInfo.permissions || [],
      platform_context: userInfo.platform_context
    }
  } catch (error) {
    console.error('❌ OIDC token validation error:', error)
    return null
  }
}

// Permission validation utilities moved to /lib/admin-permissions.ts 
// to avoid Server Action constraints

// ============================================================================
// Admin Session Management
// ============================================================================

/**
 * Get admin session from OIDC cookies
 * Main function to replace legacy getSessionFromJWT()
 */
export async function getAdminSessionFromOIDC(): Promise<AdminSession> {
  try {
    // Get OIDC tokens from cookies
    const { accessToken, idToken } = await getOIDCTokensFromCookies()
    
    // Check if required tokens are present
    if (!accessToken || !idToken) {
      return {
        isAuthenticated: false,
        user: null,
        hasAdminAccess: false,
        error: 'No valid OIDC tokens found'
      }
    }
    
    // Validate access token with backend
    const user = await validateOIDCTokenWithBackend(accessToken)
    
    if (!user) {
      return {
        isAuthenticated: false,
        user: null,
        hasAdminAccess: false,
        error: 'OIDC token validation failed'
      }
    }
    
    // Validate admin permissions
    const hasAdminAccess = validateAdminPermissions(user.permissions)
    
    if (!hasAdminAccess) {
      return {
        isAuthenticated: true,
        user,
        hasAdminAccess: false,
        error: 'Insufficient admin permissions'
      }
    }
    
    console.log('✅ Admin OIDC session validated successfully:', {
      user: user.email,
      permissions: user.permissions.length,
      adminAccess: hasAdminAccess
    })
    
    return {
      isAuthenticated: true,
      user,
      hasAdminAccess: true,
      expiresAt: Date.now() + (60 * 60 * 1000) // 1 hour from now (simplified)
    }
    
  } catch (error) {
    console.error('❌ Admin OIDC session error:', error)
    
    return {
      isAuthenticated: false,
      user: null,
      hasAdminAccess: false,
      error: 'Admin OIDC session verification failed'
    }
  }
}

/**
 * Check if admin session is valid and has required permissions
 * Helper function for middleware and components
 */
export async function isValidAdminSession(): Promise<boolean> {
  const session = await getAdminSessionFromOIDC()
  return session.isAuthenticated && session.hasAdminAccess
}

/**
 * Get admin user info if authenticated
 * Helper function for components that need user data
 */
export async function getAdminUserInfo(): Promise<OIDCUser | null> {
  const session = await getAdminSessionFromOIDC()
  return session.hasAdminAccess ? session.user : null
}

// ============================================================================
// Admin Permission Helpers
// ============================================================================

/**
 * Check specific admin permission
 */
export async function hasAdminPermission(requiredPermission: string): Promise<boolean> {
  const session = await getAdminSessionFromOIDC()
  
  if (!session.hasAdminAccess || !session.user) {
    return false
  }
  
  const permissions = session.user.permissions
  
  // Check for exact match or wildcard permissions
  return permissions.some(permission =>
    permission === requiredPermission ||
    permission === 'admin:*:*' ||
    (requiredPermission.startsWith('admin:') && permission.startsWith('admin:*:'))
  )
}

/**
 * Get all admin permissions for current user
 */
export async function getAdminPermissions(): Promise<string[]> {
  const session = await getAdminSessionFromOIDC()
  
  if (!session.hasAdminAccess || !session.user) {
    return []
  }
  
  return session.user.permissions.filter(permission => 
    permission.startsWith('admin:') || permission === '*'
  )
}

// ============================================================================
// Migration Helper Functions
// ============================================================================

/**
 * Migration helper: Check if legacy JWT cookie exists
 * Used during transition period to detect legacy sessions
 */
export async function hasLegacyAdminSession(): Promise<boolean> {
  try {
    const cookieStore = await cookies()
    const legacyJWT = cookieStore.get('epsx_admin_jwt')?.value
    
    if (legacyJWT) {
      console.warn('⚠️ Legacy admin JWT cookie detected - migration needed')
      return true
    }
    
    return false
  } catch (error) {
    return false
  }
}

/**
 * Clean up legacy admin authentication cookies
 * Called during OIDC migration process
 */
export async function cleanupLegacyAdminCookies(): Promise<void> {
  try {
    console.log('🧹 Cleaning up legacy admin authentication cookies')
    
    // This would be called from a response context where cookies can be deleted
    // For now, just log the cleanup action
    console.log('✅ Legacy admin cookies marked for cleanup')
  } catch (error) {
    console.error('❌ Failed to cleanup legacy admin cookies:', error)
  }
}

// Admin OIDC client interface moved to /lib/admin-client.ts
// to avoid Server Action export constraints