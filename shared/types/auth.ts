/**
 * Shared Authentication Types - MIGRATED TO WALLET-BASED
 * Used across frontend and admin-frontend applications
 * 
 * @deprecated The complex email-based auth system has been replaced with wallet-based auth.
 * Use types from './wallet-auth' instead for new code.
 * 
 * This file is kept for backward compatibility during the migration period.
 */

// Import the new wallet-based types
export * from './wallet-auth'

// Legacy types for backward compatibility
export interface User {
  id: string
  email: string
  name?: string
  permissions: string[]
  platform_context?: string
  permission_version?: number
  permission_last_updated?: number
  tier?: string
  verified?: boolean
}

/** @deprecated Use WalletAuthState instead */
export interface AuthState {
  user: User | null
  isLoading: boolean
  isAuthenticated: boolean
  error: string | null
  expiresAt: number | null
  
  // Legacy fields - not used in wallet-based auth
  autoRefreshEnabled?: boolean
  refreshInProgress?: boolean
  lastRefreshTime?: number | null
  
  // Actions
  login: () => void
  logout: () => Promise<void>
  getUser: () => Promise<User | null>
  refreshSession: () => Promise<void>
  clearError: () => void
  
  // Legacy methods - not used in wallet-based auth
  enableAutoRefresh?: () => void
  disableAutoRefresh?: () => void
  checkTokenHealth?: () => boolean
  
  // Permission checks
  can: (permission: string) => boolean
  hasAnyPermission: (permissions: string[]) => boolean
  hasAllPermissions: (permissions: string[]) => boolean
  hasTier: (tier: string) => boolean
  
  // Legacy platform functionality - not implemented in backend
  switchPlatform: (platform: string) => Promise<void>
  getCurrentPlatform: () => string
  getAvailablePlatforms: () => string[]
  canAccessPlatform: (platform: string) => boolean
}

/** @deprecated Use AdminWalletAuthState instead */
export interface AdminAuthState extends AuthState {
  // Admin-specific permission checks
  isAdmin: () => boolean
  canManageUsers: () => boolean
  canManageSystem: () => boolean
  canViewAnalytics: () => boolean
  canManagePlatforms: () => boolean
  canViewAudit: () => boolean
}

/** @deprecated Use FrontendSessionResponse or AdminSessionResponse instead */
export interface AuthSessionData {
  isAuthenticated: boolean
  user?: User
  expiresAt?: number
}

/** @deprecated Use WalletAuthResponse instead */
export interface AuthResponse {
  success: boolean
  message?: string
  authorizationUrl?: string
  user?: User
  expiresAt?: number
}

/** @deprecated Backend doesn't implement smart refresh - use simple session refresh instead */
export interface SmartRefreshRequest {
  current_permission_version?: number
  force_permission_reload?: boolean
}

/** @deprecated Backend doesn't implement smart refresh - use simple session refresh instead */
export interface SmartRefreshResponse {
  success: boolean
  message?: string
  permissions_changed?: boolean
  new_permissions?: string[]
  permission_version?: number
  expiresAt?: number
}

/** @deprecated Backend doesn't implement platform switching */
export interface PlatformInfo {
  tier: string
  platforms: string[]
  primaryPlatform: string
}

/** @deprecated Use PermissionGroupInfo instead */
export interface PackageTierInfo {
  currentTier: string
  hasRequiredTier: boolean
  isPremium: boolean
  isEnterprise: boolean
}

export interface PermissionGroupInfo {
  currentGroup: string
  hasRequiredGroup: boolean
  isPremium: boolean
  isEnterprise: boolean
}

/** @deprecated Use getPermissionPlatform, getPermissionResource, getPermissionAction utilities instead */
export interface PermissionCheck {
  platform: string
  resource: string
  action: string
}

/** @deprecated Backend doesn't implement platform switching */
export interface PlatformContext {
  currentPlatform: string
  availablePlatforms: string[]
  canAccessPlatform: (platform: string) => boolean
  switchPlatform: (platform: string) => Promise<void>
  platformDisplayName: string
  platformIcon: string
}

/** @deprecated Use wallet auth utilities (hasPermission, hasAnyPermission, etc.) instead */
export interface StructuredPermissions {
  can: (permission: string) => boolean
  hasPermission: (resource: string, action: string, platform?: string) => boolean
  canRead: (resource: string, platform?: string) => boolean
  canWrite: (resource: string, platform?: string) => boolean
  canManage: (resource: string, platform?: string) => boolean
  currentPlatform: string
}

/** @deprecated Use hasAdminPermissions utility and simple permission checks instead */
export interface AdminPermissions {
  // Core admin functions
  isAdmin: boolean
  canManageUsers: boolean
  canManageSystem: boolean
  canViewAnalytics: boolean
  canManagePlatforms: boolean
  canViewAudit: boolean
  
  // Specific admin permissions
  hasUserManagement: boolean
  hasPermissionManagement: boolean
  hasAnalytics: boolean
  hasBilling: boolean
  hasSystemConfig: boolean
  hasAuditLogs: boolean
  hasNotifications: boolean
  
  // Platform-specific admin permissions
  canAdminCurrentPlatform: boolean
  canSwitchPlatforms: boolean
  
  // Helper function for checking any permission
  can: (permission: string) => boolean
  currentPlatform: string
}

/** @deprecated Backend doesn't implement complex debug info - use simple session data instead */
export interface AuthDebugInfo {
  authenticated: boolean
  tokenHealth: boolean
  timeToExpiry: number
  permissionAge: number
  permissionVersion: number
  autoRefreshEnabled: boolean
  refreshInProgress: boolean
  lastRefreshTime: number | null
  permissionCount: number
  platforms: string[]
}

// ============================================================================
// MIGRATION GUIDE
// ============================================================================

/**
 * MIGRATION GUIDE: Email-based Auth → Wallet-based Auth
 * 
 * OLD (email-based):
 * - User { id, email, name, permissions }
 * - AuthState with complex auto-refresh
 * - Platform switching
 * - Enterprise API calls
 * 
 * NEW (wallet-based):
 * - WalletUser { wallet_address, permissions, tier }
 * - WalletAuthState with simple connect/authenticate
 * - Direct backend API calls (/api/v1/auth/web3/...)
 * - Simple permission utilities
 * 
 * BACKEND ENDPOINTS:
 * - GET  /api/v1/auth/web3/challenge?wallet_address=...
 * - POST /api/v1/auth/web3/verify { wallet_address, signature, message, nonce }
 * - GET  /api/v1/auth/web3/session (with Bearer token)
 * - GET  /api/v1/auth/web3/permissions (with Bearer token)
 * - DELETE /api/v1/auth/web3/logout (with Bearer token)
 * 
 * PERMISSION FORMAT:
 * - "platform:resource:action" (e.g., "admin:users:manage")
 * - Admin permissions: "admin:*:*" or anything starting with "admin:"
 * 
 * UTILITY FUNCTIONS:
 * - hasAdminPermissions(permissions: string[]): boolean
 * - hasPermission(userPerms: string[], required: string): boolean  
 * - hasAnyPermission(userPerms: string[], required: string[]): boolean
 * - getPermissionPlatform(perm: string): string
 */