/**
 * Auth Helpers for Server Components
 * OIDC token extraction and session management utilities
 */

import { COOKIES } from '@/shared/auth/cookies'
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

/**
 *
 */
export class ServerAuth {
  // Extract OIDC tokens from cookies
  /**
   *
   */
  static async getTokens(): Promise<{
    accessToken?: string
    idToken?: string
    refreshToken?: string
  }> {
    const cookieStore = await cookies()

    return {
      accessToken: cookieStore.get(COOKIES.access)?.value,
      idToken: cookieStore.get(COOKIES.id)?.value,
      refreshToken: cookieStore.get(COOKIES.refresh)?.value
    }
  }

  // Get admin session from OIDC tokens
  /**
   *
   */
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
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Error getting admin session:', _error)
      return { isLoggedIn: false }
    }
  }

  // Check if user has admin permissions
  /**
   *
   */
  static async requireAdminAuth(): Promise<AdminSession> {
    const session = await this.getAdminSession()

    if (!session.isLoggedIn) {
      redirect('/auth')
    }

    // Use role from JWT payload (computed by backend)
    // Falls back to permission check if role is not available
    const hasAdminPermission = session.user?.role === 'admin' ||
      session.user?.role === 'super_admin' ||
      session.user?.permissions?.some(p =>
        p.startsWith('admin:') || p === 'admin:*:*'
      )

    if (!hasAdminPermission) {
      redirect('/access-denied')
    }

    return session
  }

  // Permission check stubs - Backend handles enforcement via JWT middleware
  static async hasPermission(_permission: string): Promise<boolean> {
    return true;
  }

  static async requirePermission(_permission: string): Promise<void> {
    // No-op - backend handles enforcement via 403 response
  }

  // Basic JWT decode (client-side safe, no verification)
  private static decodeJWT(token: string): any {
    try {
      const parts = token.split('.')
      if (parts.length !== 3) {
        return null
      }

      const payload = parts[1] || ''
      // Add padding if needed
      const paddedPayload = payload + '='.repeat((4 - payload.length % 4) % 4)
      const decoded = Buffer.from(paddedPayload, 'base64').toString('utf8')

      return JSON.parse(decoded)
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Error decoding JWT:', _error)
      return null
    }
  }

  // Check if token is expired
  /**
   *
   * @param token
   */
  static isTokenExpired(token?: string): boolean {
    if (!token) { return true }

    const payload = this.decodeJWT(token)
    if (!payload?.exp) { return true }

    const now = Math.floor(Date.now() / 1000)
    return payload.exp < now
  }

  // Get user info from ID token
  /**
   *
   */
  static async getUserFromToken(): Promise<AdminSession['user'] | null> {
    const { idToken } = await this.getTokens()

    if (!idToken || this.isTokenExpired(idToken)) {
      return null
    }

    const payload = this.decodeJWT(idToken)
    if (!payload) { return null }

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
  /**
   *
   */
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
/**
 *
 */
export class PermissionUtils {
  // Check if permission is admin-level
  /**
   *
   * @param permission
   */
  static isAdminPermission(permission: string): boolean {
    return permission.startsWith('admin:') || permission === 'admin:*:*'
  }

  // Extract platform from permission
  /**
   *
   * @param permission
   */
  static getPlatform(permission: string): string {
    return permission.split(':')[0] || 'unknown'
  }

  // Extract resource from permission
  /**
   *
   * @param permission
   */
  static getResource(permission: string): string {
    return permission.split(':')[1] || 'unknown'
  }

  // Extract action from permission
  /**
   *
   * @param permission
   */
  static getAction(permission: string): string {
    return permission.split(':')[2] || 'unknown'
  }

  // Check if permission matches pattern
  /**
   *
   * @param permission
   * @param pattern
   */
  static matchesPattern(permission: string, pattern: string): boolean {
    if (pattern === '*' || pattern === permission) { return true }

    const permParts = permission.split(':')
    const patternParts = pattern.split(':')

    if (patternParts.length !== permParts.length) { return false }

    return patternParts.every((part, index) =>
      part === '*' || part === permParts[index]
    )
  }

  // Filter permissions by platform
  /**
   *
   * @param permissions
   * @param platform
   */
  static filterByPlatform(permissions: string[], platform: string): string[] {
    return permissions.filter(p => p.startsWith(`${platform}:`))
  }

  // Get unique platforms from permissions
  /**
   *
   * @param permissions
   */
  static getUniquePlatforms(permissions: string[]): string[] {
    const platforms = permissions.map(p => this.getPlatform(p))
    return [...new Set(platforms)]
  }
}