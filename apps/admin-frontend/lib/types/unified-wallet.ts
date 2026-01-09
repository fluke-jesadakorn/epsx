/**
 * Unified Wallet Data Types
 * Consolidates user data from multiple fragmented sources into a single interface
 */

// Base user information
export interface BaseWallet {
  id: string
  email: string
  displayName: string
  firstName?: string
  lastName?: string
  avatar?: string
  emailVerified: boolean
  createdAt: Date
  updatedAt: Date
  lastLogin?: Date
}

// User status and account state
export type WalletStatus = 'active' | 'disabled' | 'pending' | 'suspended'

export interface WalletAccountInfo extends BaseWallet {
  status: WalletStatus
  phoneNumber?: string
  timezone?: string
  language?: string
  twoFactorEnabled: boolean
}

// Groups and permissions (from IAM system)
export interface WalletGroup {
  id: string
  name: string
  description: string
  permissions: Permission[]
  createdAt: Date
}

export interface Permission {
  id: string
  name: string
  resource: string
  action: string
  description: string
}

export interface PermissionProfile {
  id: string
  name: string
  description: string
  permissions: Permission[]
  isDefault: boolean
  createdAt: Date
}

// Module access and quotas
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

// Billing and packages
export type SubscriptionTier = 'basic' | 'premium' | 'enterprise' | 'custom'

export interface BillingStatus {
  tier: SubscriptionTier
  isActive: boolean
  nextBillingDate?: Date
  lastPaymentDate?: Date
  paymentStatus: 'current' | 'overdue' | 'failed' | 'trial'
  trialEndsAt?: Date
}

export interface StockRankingPackage {
  id: string
  name: string
  tier: SubscriptionTier
  features: string[]
  assignedAt: Date
  expiresAt?: Date
  isActive: boolean
}

// API keys and developer access
export interface ApiKey {
  id: string
  name: string
  keyPreview: string // Only show first/last few characters
  permissions: string[]
  isActive: boolean
  createdAt: Date
  lastUsed?: Date
  expiresAt?: Date
}

// Activity and audit logs
export interface ActivityRecord {
  id: string
  action: string
  resource: string
  details?: Record<string, unknown>;
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

// Usage metrics
export interface UsageMetrics {
  apiCallsThisMonth: number
  storageUsed: number
  lastActiveDate: Date
  sessionsThisMonth: number
  averageSessionDuration: number
}

// Combined unified user data
export interface UnifiedWalletData extends WalletAccountInfo {
  // Permissions & Groups (consolidates IAM data)
  groups: WalletGroup[]
  customPermissions: Permission[]
  permissionProfiles: PermissionProfile[]

  // Modules & Access (consolidates module management)
  moduleAccess: ModuleAccess[]
  moduleQuotas: ModuleQuota[]

  // Billing & Packages (consolidates billing + stock ranking)
  billing: BillingStatus
  stockRankingPackages: StockRankingPackage[]

  // Developer Access
  apiKeys: ApiKey[]

  // Activity & Analytics (new consolidated view)
  recentActivity: ActivityRecord[]
  loginHistory: LoginRecord[]
  usageMetrics: UsageMetrics
}

// Request/Response types for API operations
export interface WalletListFilters {
  search?: string
  groups?: string[]
  status?: WalletStatus[]
  tier?: SubscriptionTier[]
  modules?: string[]
  emailVerified?: boolean
  lastLoginAfter?: Date
  lastLoginBefore?: Date
  createdAfter?: Date
  createdBefore?: Date
  page?: number
  limit?: number
  sortBy?: 'createdAt' | 'lastLogin' | 'displayName' | 'status'
  sortOrder?: 'asc' | 'desc'
}

export interface WalletListResponse {
  wallets: UnifiedWalletData[]
  totalCount: number
  page: number
  limit: number
  hasNextPage: boolean
}

// Form data types for user operations
export interface WalletProfileUpdateData {
  displayName?: string
  firstName?: string
  lastName?: string
  phoneNumber?: string
  timezone?: string
  language?: string
}

export interface WalletStatusUpdateData {
  status: WalletStatus
  reason?: string
}

export interface WalletGroupUpdateData {
  groupIds: string[]
  customPermissions?: Permission[]
}

export interface ModuleAccessUpdateData {
  moduleAccess: Omit<ModuleAccess, 'grantedAt' | 'grantedBy'>[]
  quotas?: ModuleQuota[]
}

export interface BillingUpdateData {
  tier?: SubscriptionTier
  stockRankingPackages?: string[]
}

// Error types
export interface WalletOperationError {
  code: string
  message: string
  field?: string
  details?: Record<string, any>
}

export interface WalletOperationResult<T = unknown> {
  success: boolean
  data?: T
  error?: WalletOperationError
}