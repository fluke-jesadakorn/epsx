/**
 * CANONICAL USER DOMAIN TYPES
 * Single source of truth for all user-related interfaces across EPSX
 * Consolidates 200+ duplicate User interfaces from both frontend and admin apps
 */

// ============================================================================
// CORE USER TYPES - Foundation for all user representations
// ============================================================================

/**
 * Base user identity - minimal fields present in all user contexts
 */
export interface BaseUser {
  id: string
  email: string
  name: string
  role: UserRole
  createdAt: Date
  updatedAt: Date
}

/**
 * User roles across the EPSX platform
 */
export type UserRole = 'user' | 'premium_user' | 'admin' | 'super_admin'

/**
 * User account status
 */
export type UserStatus = 'active' | 'disabled' | 'pending' | 'suspended' | 'trial'

/**
 * Groups for access control (NEW)
 */
export type Group = 'Basic Access Group' | 'Standard Access Group' | 'Premium Access Group' | 'Professional Access Group' | 'Enterprise Access Group'

/**
 * @deprecated Use Group instead
 */
export type PermissionGroup = Group

/**
 * Package/subscription tiers
 * @deprecated Use Group instead
 */
export type PackageTier = 'FREE' | 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM' | 'ENTERPRISE'

// ============================================================================
// USER PROFILE TYPES - Extended user information
// ============================================================================

/**
 * Complete user profile with all account information
 */
export interface UserProfile extends BaseUser {
  // Identity
  firstName?: string
  lastName?: string
  displayName?: string
  avatar?: string

  // Account state
  status: UserStatus
  emailVerified: boolean
  lastLogin?: Date
  lastActivityAt?: Date

  // Contact & preferences
  phoneNumber?: string
  timezone?: string
  language?: string
  twoFactorEnabled: boolean

  // Platform context
  group: Group
  permissionGroup: Group // Backward compatibility alias in the interface as well
  permissions: string[]
  platforms: string[]
  primaryPlatform: string

  // Web3 integration
  walletAddress?: string

  // Context tracking
  platformContext?: string

  // Backward compatibility
  /** @deprecated Use permissionGroup instead */
  packageTier?: PackageTier
}

/**
 * Minimal user info for session/auth contexts
 */
export interface UserSession extends Pick<UserProfile, 'id' | 'email' | 'name' | 'role' | 'permissions' | 'permissionGroup' | 'walletAddress'> {
  expiresAt: number
  sessionId?: string
  isLoggedIn: true
}

/**
 * Admin-specific user representation with management fields
 */
export interface AdminUserProfile extends UserProfile {
  // Admin metadata
  billing: BillingStatus
  moduleAccess: ModuleAccess[]
  moduleQuotas: ModuleQuota[]
  stockRankingPackages: StockRankingPackage[]
  apiKeys: ApiKey[]

  // Activity tracking
  recentActivity: ActivityRecord[]
  loginHistory: LoginRecord[]
  usageMetrics: UsageMetrics
}

// ============================================================================
// BILLING & SUBSCRIPTION TYPES
// ============================================================================

export interface BillingStatus {
  group: Group
  isActive: boolean
  nextBillingDate?: Date
  lastPaymentDate?: Date
  paymentStatus: 'current' | 'overdue' | 'failed' | 'trial'
  trialEndsAt?: Date
}

export interface StockRankingPackage {
  id: string
  name: string
  group: Group
  features: string[]
  assignedAt: Date
  expiresAt?: Date
  isActive: boolean
}

// ============================================================================
// MODULE ACCESS & PERMISSIONS
// ============================================================================

export interface ModuleAccess {
  moduleId: string
  moduleName: string
  accessLevel: 'read' | 'write' | 'admin'
  isActive: boolean
  grantedAt: Date
  expiresAt?: Date
  grantedBy: string
}

export interface ModuleQuota {
  moduleId: string
  quotaType: 'requests' | 'storage' | 'users' | 'time'
  limit: number
  used: number
  period: 'daily' | 'monthly' | 'yearly'
  resetDate?: Date
}

// ============================================================================
// API & DEVELOPER ACCESS
// ============================================================================

export interface ApiKey {
  id: string
  name: string
  keyPreview: string // Only first/last few characters
  permissions: string[]
  isActive: boolean
  createdAt: Date
  lastUsed?: Date
  expiresAt?: Date
}

// ============================================================================
// ACTIVITY & AUDIT TRACKING
// ============================================================================

export interface ActivityRecord {
  id: string
  action: string
  resource: string
  details: Record<string, unknown>
  timestamp: Date
  ipAddress?: string
  userAgent?: string
}

export interface LoginRecord {
  id: string
  timestamp: Date
  ipAddress: string
  userAgent: string
  success: boolean
  failureReason?: string
  location?: {
    country: string
    city?: string
  }
}

export interface UsageMetrics {
  apiCallsThisMonth: number
  storageUsed: number
  lastActiveDate: Date
  sessionsThisMonth: number
  averageSessionDuration: number
}

// ============================================================================
// REQUEST/RESPONSE TYPES - API operations
// ============================================================================

export interface UserListFilters {
  search?: string
  roles?: UserRole[]
  status?: UserStatus[]
  groups?: Group[]
  modules?: string[]
  emailVerified?: boolean
  lastLoginAfter?: Date
  lastLoginBefore?: Date
  createdAfter?: Date
  createdBefore?: Date
  page?: number
  limit?: number
  sortBy?: 'createdAt' | 'lastLogin' | 'displayName' | 'status' | 'name'
  sortOrder?: 'asc' | 'desc'
}

export interface UserListResponse {
  users: UserProfile[]
  totalCount: number
  page: number
  limit: number
  hasNextPage: boolean
}

// ============================================================================
// UPDATE/MUTATION TYPES - Form data for user operations
// ============================================================================

export interface UserProfileUpdateData {
  name?: string
  displayName?: string
  firstName?: string
  lastName?: string
  phoneNumber?: string
  timezone?: string
  language?: string
}

export interface UserStatusUpdateData {
  status: UserStatus
  reason?: string
}

export interface UserRoleUpdateData {
  role: UserRole
  customPermissions?: string[]
}

export interface BillingUpdateData {
  group?: Group
  stockRankingPackages?: string[]
}

// ============================================================================
// ERROR & RESULT TYPES
// ============================================================================

export interface UserOperationError {
  code: string
  message: string
  field?: string
  details?: Record<string, unknown>
}

export interface UserOperationResult<T = unknown> {
  success: boolean
  data?: T
  error?: UserOperationError
}

/**
 * @deprecated Use getGroupLevel instead
 */
export function getUserTierLevel(tier: PackageTier): number {
  const levels: Record<PackageTier, number> = {
    FREE: 0,
    BRONZE: 1,
    SILVER: 2,
    GOLD: 3,
    PLATINUM: 4,
    ENTERPRISE: 5
  }
  return levels[tier] || 0
}

// ============================================================================
// LEGACY COMPATIBILITY ALIASES - For gradual migration
// ============================================================================

/** @deprecated Use UserProfile instead */
export type User = UserProfile

/** @deprecated Use AdminUserProfile instead */
export type UnifiedUserData = AdminUserProfile

/** @deprecated Use UserSession instead */
export type SessionData = UserSession

/** @deprecated Use PackageTier instead */
export type SubscriptionTier = PackageTier