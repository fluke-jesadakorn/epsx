/**
 * Shared Authentication Types
 * Used across frontend and admin-frontend applications
 * 
 * This file re-exports types from wallet-auth and provides legacy compatibility types.
 */

// Re-export all wallet-based types
export * from './wallet-auth'

// Permission Group Info (actively used)
export interface PermissionGroupInfo {
  currentGroup: string
  hasRequiredGroup: boolean
  isPremium: boolean
  isEnterprise: boolean
}

// Legacy types kept for compatibility with existing code (shared/auth/store.ts, etc.)
export interface User {
  id: string
  email: string
  name?: string
  /** @deprecated Use plan instead */
  permissions: string[]
  plan: string // Primary access field
  platform_context?: string
  permission_version?: number
  permission_last_updated?: number
  /** @deprecated Use plan instead */
  tier?: string
  verified?: boolean
  enterpriseTier?: string
  hasApiAccess?: boolean
  verifiedTokensUsd?: number
  nftCollections?: string[]
  daoMemberships?: string[]
}

export interface AuthState {
  // Core state
  user: User | null
  isLoading: boolean
  isAuthenticated: boolean
  isAuthenticating: boolean
  hasInitialized: boolean
  error: string | null
  expiresAt: number | null

  // Identity and connection
  walletAddress?: string
  isConnected: boolean

  // Enterprise Data
  plan: string // Primary access field
  /** @deprecated Use plan instead */
  permissions: string[]
  enterpriseTier: string
  hasApiAccess: boolean
  verifiedTokensUsd: number
  nftCollections: string[]
  daoMemberships: string[]

  // Session state
  accessToken?: string
  refreshToken?: string
  isNewUser?: boolean

  // Auto-refresh state
  autoRefreshEnabled?: boolean
  refreshInProgress?: boolean
  lastRefreshTime?: number | null

  // Core Actions
  /** @deprecated Use authenticate for Web3 auth */
  login: () => Promise<void>
  authenticate: (walletAddress: string, signature: string, message: string, nonce: string) => Promise<boolean>
  logout: () => Promise<void>
  getUser: () => Promise<User | null>
  refreshSession: () => Promise<void>
  refreshEnterpriseData: () => Promise<void>
  generateApiKey: (name: string) => Promise<string>
  clearError: () => void

  // Setters
  setConnected: (connected: boolean) => void
  setAuthenticated: (authenticated: boolean) => void
  setAuthenticating: (authenticating: boolean) => void
  setLoading: (loading: boolean) => void
  setInitialized: (initialized: boolean) => void
  setWalletAddress: (address: string | undefined) => void
  setPermissions: (permissions: string[]) => void
  setEnterpriseTier: (tier: string) => void
  setApiAccess: (hasAccess: boolean) => void
  setAccessToken: (token: string | undefined) => void
  setExpiresAt: (expiresAt: number | undefined) => void
  setError: (error: string | undefined) => void

  // Auto-refresh Actions
  enableAutoRefresh?: () => void
  disableAutoRefresh?: () => void
  checkTokenHealth?: () => boolean

  // Permission Helpers

}

export interface AdminAuthState extends AuthState {
  isAdmin: () => boolean
  canManageUsers: () => boolean
  canManageSystem: () => boolean
  canViewAnalytics: () => boolean
  canManagePlatforms: () => boolean
  canViewAudit: () => boolean
}

export interface AuthSessionData {
  isAuthenticated: boolean
  user?: User
  expiresAt?: number
}

export interface AuthResponse {
  success: boolean
  message?: string
  authorizationUrl?: string
  user?: User
  expiresAt?: number
}

export interface SmartRefreshRequest {
  current_permission_version?: number
  force_permission_reload?: boolean
}

export interface SmartRefreshResponse {
  success: boolean
  message?: string
  permissions_changed?: boolean
  new_permissions?: string[]
  permission_version?: number
  expiresAt?: number
}