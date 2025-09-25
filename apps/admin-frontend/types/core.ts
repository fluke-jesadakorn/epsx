/**
 * Core Type Definitions
 * Consolidates User, Authentication, and Permission types
 * Replaces: auth-separation.ts, admin-types.ts, iam.ts, unified-user.ts
 */

// ============================================================================
// Authentication & User Types
// ============================================================================

export interface User {
  id: string; // wallet_address as primary identifier
  wallet_address: string; // Primary identifier for wallet authentication
  email?: string; // Optional linked email
  name?: string;
  firstName?: string;
  lastName?: string;
  displayName?: string;
  role: 'admin' | 'user' | 'premium_user';
  status?: 'active' | 'inactive' | 'suspended' | 'deleted';
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
  lastLoginAt?: string;
  
  // Wallet authentication context
  sub?: string; // wallet_address as subject
  
  // Permission system
  permissions?: string[];
  packageTier: string;
  
  // Platform context
  platforms?: string[];
  primaryPlatform?: string;
  platformContext?: string;
  
  // Wallet-specific fields
  walletConnectedAt?: string;
  lastSignatureAt?: string;
  nftHoldings?: number;
  tokenBalance?: string;
}

export interface WalletUser {
  sub: string; // wallet_address as subject
  wallet_address: string;
  email?: string; // Optional linked email
  name?: string;
  permissions: string[];
  platform_context?: string;
}

// Standard authenticated admin user interface
export interface AuthenticatedAdminUser extends WalletUser {
  email?: string; // Optional linked email
  adminPermissions: string[];
  adminLevel: 'super' | 'manager' | 'moderator';
}

export interface AdminSession {
  isAuthenticated: boolean;
  isLoggedIn: boolean; // Alias for backwards compatibility
  user: WalletUser | null;
  hasAdminAccess: boolean;
  expiresAt?: number;
  error?: string;
}

export interface UserSession {
  user: User;
  isLoggedIn: boolean;
  sessionType: 'user' | 'admin';
  expiresAt: number;
  securityContext?: SecurityContext;
}

export interface SecurityContext {
  sessionType: 'admin' | 'user';
  securityLevel?: string;
  mfaVerified?: boolean;
  deviceTrusted?: boolean;
  ipAddress?: string;
  userAgent?: string;
}

// ============================================================================
// Permission System Types
// ============================================================================

export interface Permission {
  id?: string;
  resource: string;
  action: string;
  platform?: string;
  context?: string;
  expiresAt?: string;
  createdAt?: string;
  assignedBy?: string;
}

export interface StructuredPermission {
  platform: string;
  resource: string;
  action: string;
  timestamp?: number; // For embedded timestamp permissions
}

export interface PermissionProfile {
  id: string;
  name: string;
  description: string;
  category: string;
  targetTier: string;
  isActive: boolean;
  permissions: Permission[];
  createdAt: string;
  updatedAt: string;
  assignedUserCount?: number;
}

export interface PermissionAnalytics {
  totalPermissions: number;
  usersWithPermissions: number;
  expiringSoon: number;
  expired: number;
  healthScore: number;
  recentActivity: number;
}

export interface PermissionHealth {
  userId: string;
  totalPermissions: number;
  expiringPermissions: string[];
  expiredPermissions: string[];
  healthScore: number;
  lastCalculated: string;
  recommendations?: string[];
}

// ============================================================================
// Role & Access Control Types
// ============================================================================

export interface Role {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  isSystemRole: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface UserRole {
  userId: string;
  roleId: string;
  role: Role;
  assignedAt: string;
  assignedBy: string;
  expiresAt?: string;
}

// ============================================================================
// Token Management Types
// ============================================================================

export interface TokenPair {
  access_token: string;
  id_token?: string;
  refresh_token?: string;
  expires_in?: number;
  token_type?: string;
}

export interface Web3AuthTokens {
  accessToken: string | null;
  sessionToken: string | null;
  signatureHash: string | null;
  expiresAt?: number;
}

// ============================================================================
// User Statistics & Analytics
// ============================================================================

export interface UserStats {
  totalUsers: number;
  activeUsers: number;
  deletedUsers: number;
  recentUsers30Days: number;
  byPermissions: Record<string, number>;
  byTier: Record<string, number>;
  userCreationByMonth: Record<string, number>;
  generatedAt: string;
}

export interface ActivityStats {
  totalActivities: number;
  last24Hours: number;
  lastWeek: number;
  lastMonth: number;
  topActivities: Array<{
    type: string;
    count: number;
  }>;
}

// ============================================================================
// Filtering & Search Types
// ============================================================================

export interface UserFilters {
  role?: string;
  status?: 'active' | 'inactive' | 'deleted';
  search?: string;
  platform?: string;
  tier?: string;
  permissions?: string[];
  limit?: number;
  offset?: number;
  page?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface PermissionFilters {
  userId?: string;
  platform?: string;
  resource?: string;
  action?: string;
  expiring?: boolean;
  expired?: boolean;
  assignedBy?: string;
  createdAfter?: string;
  createdBefore?: string;
}

// ============================================================================
// Pagination Types
// ============================================================================

export interface PaginationInfo {
  currentPage: number;
  totalPages: number;
  totalItems: number;
  itemsPerPage: number;
  hasNextPage: boolean;
  hasPrevPage: boolean;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: PaginationInfo;
}

// ============================================================================
// System Configuration Types
// ============================================================================

export interface SystemConfig {
  jwtSecretConfigured: boolean;
  apiBaseUrl: string;
  smtpConfigured: boolean;
  oauthConfigured: boolean;
  features: Record<string, boolean>;
  maintenance: {
    enabled: boolean;
    message?: string;
    estimatedEnd?: string;
  };
}

export interface FeatureFlags {
  [key: string]: boolean;
}

// ============================================================================
// Audit & Logging Types
// ============================================================================

export interface AuditLog {
  id: string;
  userId: string;
  action: string;
  resource: string;
  details: Record<string, any>;
  ipAddress: string;
  userAgent: string;
  timestamp: string;
  success: boolean;
}

export interface ActivityLog {
  id: string;
  userId: string;
  type: 'login' | 'logout' | 'permission_change' | 'role_change' | 'profile_update' | 'api_call';
  description: string;
  metadata?: Record<string, any>;
  timestamp: string;
}

// ============================================================================
// Type Guards
// ============================================================================

export function isAdminUser(user: User): boolean {
  return user.role === 'admin' || (user.permissions || []).some(p => 
    p === 'admin:*:*' || p.startsWith('admin:')
  );
}

export function isValidSession(session: AdminSession | UserSession): boolean {
  if ('isAuthenticated' in session) {
    return session.isAuthenticated;
  }
  return session.isLoggedIn && !!session.user;
}

export function hasExpiredPermissions(permissions: string[]): boolean {
  const now = Date.now() / 1000;
  return permissions.some(permission => {
    const parts = permission.split(':');
    if (parts.length === 4) {
      const timestamp = parseInt(parts[3], 10);
      return !isNaN(timestamp) && timestamp <= now;
    }
    return false;
  });
}

// ============================================================================
// Utility Types
// ============================================================================

export type UserStatus = 'active' | 'inactive' | 'suspended' | 'deleted';
export type PermissionType = 'permanent' | 'temporary' | 'expired';
export type Platform = 'admin' | 'epsx' | 'epsx-pay' | 'epsx-token';
export type SortOrder = 'asc' | 'desc';

// Generic utility types
export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;
export type RequiredFields<T, K extends keyof T> = T & Required<Pick<T, K>>;

// Response wrapper types
export type DataWithPagination<T> = {
  data: T[];
  pagination: PaginationInfo;
};

export type TimestampedEntity = {
  createdAt: string;
  updatedAt: string;
};

export type AuditableEntity = TimestampedEntity & {
  createdBy?: string;
  updatedBy?: string;
};