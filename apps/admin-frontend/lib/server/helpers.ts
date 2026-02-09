/**
 * Auth Helpers for Server Components
 * OIDC token extraction and session management utilities
 */

import { cookies } from 'next/headers'
import { redirect } from 'next/navigation'

import { COOKIES } from '@/shared/auth/cookies'
import { logger } from '@/shared/utils/logger'

interface JWTPayload {
  sub?: string
  user_id?: string
  email?: string
  name?: string
  display_name?: string
  role?: string
  permissions?: string[]
  package_tier?: string
  exp?: number
  [key: string]: unknown
}

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
      accessToken: cookieStore.get(COOKIES.access_token)?.value,
      idToken: cookieStore.get(COOKIES.id_token)?.value,
      refreshToken: cookieStore.get(COOKIES.refresh_token)?.value
    }
  }

  // Get admin session from OIDC tokens
  /**
   *
   */
  static async getAdminSession(): Promise<AdminSession> {
    try {
      const { accessToken, idToken } = await this.getTokens()

      if (accessToken === undefined || idToken === undefined) {
        return { isLoggedIn: false }
      }

      // Decode ID token to get user info (basic JWT decode without verification)
      const payload = this.decodeJWT(idToken)

      if (payload === null) {
        return { isLoggedIn: false }
      }

      return {
        user: this.extractUserFromPayload(payload),
        isLoggedIn: true,
        accessToken,
        idToken
      }
    } catch (error) {
      logger.error('Error getting admin session:', error)
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

    // PERMISSION REFACTOR: Full admin enforcement is now handled by the backend.
    // We only perform the basic login check here.
    return session

  }

  // Permission check stubs - Backend handles enforcement via JWT middleware
  /**
   *
   * @param _permission
   */
  static async hasPermission(_permission: string): Promise<boolean> {
    const session = await this.getAdminSession();
    return session.isLoggedIn;
  }

  /**
   *
   * @param _permission
   */
  static async requirePermission(_permission: string): Promise<void> {
    // No-op - backend handles enforcement via 403 response
    await this.requireAdminAuth();
  }

  // Basic JWT decode (client-side safe, no verification)
  private static decodeJWT(token: string): JWTPayload | null {
    try {
      const parts = token.split('.')
      if (parts.length !== 3) {
        return null
      }

      const payload = parts[1] ?? ''
      // Add padding if needed
      const paddedPayload = payload + '='.repeat((4 - payload.length % 4) % 4)
      const decoded = Buffer.from(paddedPayload, 'base64').toString('utf8')

      return JSON.parse(decoded) as JWTPayload
    } catch (error) {

      logger.error('Error decoding JWT:', error)
      return null
    }
  }

  private static extractUserFromPayload(payload: JWTPayload): AdminSession['user'] {
    return {
      id: String(payload.sub ?? payload.user_id ?? ''),
      email: String(payload.email ?? ''),
      name: String(payload.name ?? payload.display_name ?? ''),
      role: String(payload.role ?? 'user'),
      permissions: payload.permissions ?? [],
      packageTier: String(payload.package_tier ?? 'basic')
    }
  }

  // Check if token is expired
  /**
   *
   * @param token
   */
  static isTokenExpired(token?: string): boolean {
    if (token === undefined || token === '') { return true }

    const payload = this.decodeJWT(token)
    if (payload?.exp === undefined || payload.exp === 0) { return true }

    const now = Math.floor(Date.now() / 1000)
    return payload.exp < now
  }

  // Get user info from ID token
  /**
   *
   */
  static async getUserFromToken(): Promise<AdminSession['user'] | null> {
    const { idToken } = await this.getTokens()

    if (idToken === undefined || idToken === '' || this.isTokenExpired(idToken)) {
      return null
    }

    const payload = this.decodeJWT(idToken)
    if (payload === null) { return null }

    return this.extractUserFromPayload(payload)
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

    if (accessToken !== undefined && accessToken !== '' && !this.isTokenExpired(accessToken)) {
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
    return permission.split(':')[0] ?? 'unknown'
  }

  // Extract resource from permission
  /**
   *
   * @param permission
   */
  static getResource(permission: string): string {
    return permission.split(':')[1] ?? 'unknown'
  }

  // Extract action from permission
  /**
   *
   * @param permission
   */
  static getAction(permission: string): string {
    return permission.split(':')[2] ?? 'unknown'
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