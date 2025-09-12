/**
 * Auth Helpers for Server Components
 * OIDC token extraction and session management utilities
 */

import { cookies } from 'next/headers'
import { redirect } from 'next/navigation'

export interface AdminSession {
  user?: {
    id: string
    email: string
    name?: string
    role: string
    permissions: string[]
    packageTier: string
  }
  isLoggedIn: boolean
  accessToken?: string
  idToken?: string
}

export class ServerAuth {
  // Extract OIDC tokens from cookies
  static async getTokens(): Promise<{
    accessToken?: string
    idToken?: string
    refreshToken?: string
  }> {
    const cookieStore = await cookies()
    
    return {
      accessToken: cookieStore.get('access_token')?.value,
      idToken: cookieStore.get('id_token')?.value,
      refreshToken: cookieStore.get('refresh_token')?.value
    }
  }

  // Get admin session from OIDC tokens
  static async getAdminSession(): Promise<AdminSession> {
    try {
      const { accessToken, idToken } = await this.getTokens()
      
      if (!accessToken || !idToken) {
        return { isLoggedIn: false }
      }

      // Decode ID token to get user info (basic JWT decode without verification)
      const payload = this.decodeJWT(idToken)
      
      if (!payload) {
        return { isLoggedIn: false }
      }

      // Extract user information from ID token
      const user = {
        id: payload.sub || payload.user_id || '',
        email: payload.email || '',
        name: payload.name || payload.display_name || '',
        role: payload.role || 'user',
        permissions: payload.permissions || [],
        packageTier: payload.package_tier || 'basic'
      }

      return {
        user,
        isLoggedIn: true,
        accessToken,
        idToken
      }
    } catch (error) {
      console.error('Error getting admin session:', error)
      return { isLoggedIn: false }
    }
  }

  // Check if user has admin permissions
  static async requireAdminAuth(): Promise<AdminSession> {
    const session = await this.getAdminSession()
    
    if (!session.isLoggedIn) {
      redirect('/login')
    }

    // Check for admin permissions
    const hasAdminPermission = session.user?.permissions?.some(p => 
      p.startsWith('admin:') || p === 'admin:*:*'
    ) || session.user?.role === 'admin'

    if (!hasAdminPermission) {
      redirect('/access-denied')
    }

    return session
  }

  // Check specific permissions
  static async hasPermission(permission: string): Promise<boolean> {
    const session = await this.getAdminSession()
    
    if (!session.isLoggedIn || !session.user) {
      return false
    }

    const userPermissions = session.user.permissions
    
    // Check for exact permission match
    if (userPermissions.includes(permission)) {
      return true
    }

    // Check for wildcard permissions
    const [platform, resource, action] = permission.split(':')
    
    return userPermissions.some(p => {
      if (p === 'admin:*:*') return true // Super admin
      if (p === `${platform}:*:*`) return true // Platform admin
      if (p === `${platform}:${resource}:*`) return true // Resource admin
      return false
    })
  }

  // Require specific permission
  static async requirePermission(permission: string): Promise<void> {
    const hasAccess = await this.hasPermission(permission)
    
    if (!hasAccess) {
      redirect('/unauthorized')
    }
  }

  // Basic JWT decode (client-side safe, no verification)
  private static decodeJWT(token: string): any {
    try {
      const parts = token.split('.')
      if (parts.length !== 3) {
        return null
      }

      const payload = parts[1]
      // Add padding if needed
      const paddedPayload = payload + '='.repeat((4 - payload.length % 4) % 4)
      const decoded = Buffer.from(paddedPayload, 'base64').toString('utf8')
      
      return JSON.parse(decoded)
    } catch (error) {
      console.error('Error decoding JWT:', error)
      return null
    }
  }

  // Check if token is expired
  static isTokenExpired(token?: string): boolean {
    if (!token) return true
    
    const payload = this.decodeJWT(token)
    if (!payload || !payload.exp) return true
    
    const now = Math.floor(Date.now() / 1000)
    return payload.exp < now
  }

  // Get user info from ID token
  static async getUserFromToken(): Promise<AdminSession['user'] | null> {
    const { idToken } = await this.getTokens()
    
    if (!idToken || this.isTokenExpired(idToken)) {
      return null
    }

    const payload = this.decodeJWT(idToken)
    if (!payload) return null

    return {
      id: payload.sub || payload.user_id || '',
      email: payload.email || '',
      name: payload.name || payload.display_name || '',
      role: payload.role || 'user',
      permissions: payload.permissions || [],
      packageTier: payload.package_tier || 'basic'
    }
  }

  // Create authorization header for server requests
  static async getAuthHeaders(): Promise<Record<string, string>> {
    const { accessToken } = await this.getTokens()
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json'
    }
    
    if (accessToken && !this.isTokenExpired(accessToken)) {
      headers['Authorization'] = `Bearer ${accessToken}`
    }
    
    return headers
  }
}

// Permission checking utilities
export class PermissionUtils {
  // Check if permission is admin-level
  static isAdminPermission(permission: string): boolean {
    return permission.startsWith('admin:') || permission === 'admin:*:*'
  }

  // Extract platform from permission
  static getPlatform(permission: string): string {
    return permission.split(':')[0] || 'unknown'
  }

  // Extract resource from permission
  static getResource(permission: string): string {
    return permission.split(':')[1] || 'unknown'
  }

  // Extract action from permission
  static getAction(permission: string): string {
    return permission.split(':')[2] || 'unknown'
  }

  // Check if permission matches pattern
  static matchesPattern(permission: string, pattern: string): boolean {
    if (pattern === '*' || pattern === permission) return true
    
    const permParts = permission.split(':')
    const patternParts = pattern.split(':')
    
    if (patternParts.length !== permParts.length) return false
    
    return patternParts.every((part, index) => 
      part === '*' || part === permParts[index]
    )
  }

  // Filter permissions by platform
  static filterByPlatform(permissions: string[], platform: string): string[] {
    return permissions.filter(p => p.startsWith(`${platform}:`))
  }

  // Get unique platforms from permissions
  static getUniquePlatforms(permissions: string[]): string[] {
    const platforms = permissions.map(p => this.getPlatform(p))
    return [...new Set(platforms)]
  }
}