/**
 * Admin OIDC Types and Interfaces
 * Separated from server actions to avoid Next.js build constraints
 */

export interface OIDCUser {
  sub: string
  email: string
  name?: string
  permissions: string[]
  platform_context?: string
}

export interface AdminSession {
  isAuthenticated: boolean
  user: OIDCUser | null
  hasAdminAccess: boolean
  expiresAt?: number
  error?: string
}

export interface TokenPair {
  access_token: string
  id_token?: string
  refresh_token?: string
  expires_in?: number
}

export interface AdminFilters {
  role?: string
  status?: string
  search?: string
  limit?: number
  offset?: number
  page?: number
}

export interface UserPermissionFilters {
  user_id?: string
  platform?: string
  resource?: string
  action?: string
}