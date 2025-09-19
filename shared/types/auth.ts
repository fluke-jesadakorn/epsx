/**
 * Shared Authentication Types
 * Used across frontend and admin-frontend applications
 */

export interface User {
  id: string
  email: string
  name?: string
  permissions: string[]  // Structured permissions: "platform:resource:action"
  platform_context?: string    // Current platform context
  // Enhanced permission tracking
  permission_version?: number   // Version for cache invalidation
  permission_last_updated?: number  // Unix timestamp
  tier?: string                // User's package tier
  verified?: boolean           // Account verification status
}

export interface AuthState {
  user: User | null
  isLoading: boolean
  isAuthenticated: boolean
  error: string | null
  expiresAt: number | null
  
  // Auto-refresh tracking (frontend only)
  autoRefreshEnabled?: boolean
  refreshInProgress?: boolean
  lastRefreshTime?: number | null
  
  // Actions
  login: () => void
  logout: () => Promise<void>
  getUser: () => Promise<User | null>
  refreshSession: () => Promise<void>
  clearError: () => void
  
  // Auto-refresh management (frontend only)
  enableAutoRefresh?: () => void
  disableAutoRefresh?: () => void
  checkTokenHealth?: () => boolean
  
  // Permission checks - pure permission system
  can: (permission: string) => boolean
  hasAnyPermission: (permissions: string[]) => boolean
  hasAllPermissions: (permissions: string[]) => boolean
  hasTier: (tier: string) => boolean
  
  // Cross-platform functionality
  switchPlatform: (platform: string) => Promise<void>
  getCurrentPlatform: () => string
  getAvailablePlatforms: () => string[]
  canAccessPlatform: (platform: string) => boolean
}

export interface AdminAuthState extends AuthState {
  // Admin-specific permission checks
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

// Platform and permission utilities types
export interface PlatformInfo {
  tier: string
  platforms: string[]
  primaryPlatform: string
}

export interface PackageTierInfo {
  currentTier: string
  hasRequiredTier: boolean
  isPremium: boolean
  isEnterprise: boolean
}

export interface PermissionCheck {
  platform: string
  resource: string
  action: string
}

export interface PlatformContext {
  currentPlatform: string
  availablePlatforms: string[]
  canAccessPlatform: (platform: string) => boolean
  switchPlatform: (platform: string) => Promise<void>
  platformDisplayName: string
  platformIcon: string
}

export interface StructuredPermissions {
  can: (permission: string) => boolean
  hasPermission: (resource: string, action: string, platform?: string) => boolean
  canRead: (resource: string, platform?: string) => boolean
  canWrite: (resource: string, platform?: string) => boolean
  canManage: (resource: string, platform?: string) => boolean
  currentPlatform: string
}

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