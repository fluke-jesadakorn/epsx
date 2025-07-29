// Re-export permission types from the main types package
export type {
  Permission,
  Role,
  PermissionCheckRequest,
  PermissionCheckResponse,
  UserPermissionStatus,
  // Permission profile types from the comprehensive system
  DynamicPermissionProfile,
  PermissionProfilePermission,
  PermissionCondition,
  PermissionCategory,
  PermissionScope,
  PermissionProfileScope,
  PermissionProfileStatus,
  PackageTier
} from '@epsx/types';

// Auth-shared specific permission utilities and constants
export enum UserRole {
  USER = 'user',
  PREMIUM = 'premium', 
  MODERATOR = 'moderator',
  ADMIN = 'admin',
  SUPER_ADMIN = 'super_admin'
}

// Role hierarchy for permission checking
export const ROLE_HIERARCHY: Record<string, number> = {
  'user': 1,
  'premium': 2,
  'moderator': 3,
  'admin': 4,
  'super_admin': 5
};

// Permission checking result - enhanced version
export interface PermissionCheckResult {
  allowed: boolean;
  reason: string;
  requiredPermission?: string;
  userPermissions?: string[];
  userRole?: string;
  userProfiles?: string[];
  context?: Record<string, any>;
}

// Route permission configuration for Next.js middleware
export interface RoutePermissionConfig {
  permission: string;
  profile?: string;
  fallbackRole?: string;
  minimumRole?: string;
  description?: string;
  allowAnonymous?: boolean;
  requireEmailVerification?: boolean;
}

// Permission cache entry for performance optimization
export interface PermissionCacheEntry {
  permissions: string[];
  role: string;
  profiles: string[];
  isAdmin: boolean;
  timestamp: number;
  ttl: number;
  userId: string;
}

// Simplified permission profile for auth-shared context
export interface PermissionProfile {
  id: string;
  name: string;
  permissions: string[];
  category?: string;
  package_tier?: string;
  active: boolean;
  description?: string;
}

// Additional auth-shared specific types
export interface AuthMiddlewareConfig {
  publicRoutes: string[];
  protectedRoutes: RoutePermissionConfig[];
  adminRoutes: string[];
  fallbackRoute: string;
  loginRoute: string;
}

export interface AuthGuardProps {
  children: React.ReactNode;
  permission?: string;
  profile?: string;
  minimumRole?: UserRole;
  fallback?: React.ReactNode;
  loading?: React.ReactNode;
}

// Permission utility functions types
export type PermissionChecker = (
  permission: string,
  userPermissions: string[],
  userRole?: string,
  userProfiles?: string[]
) => boolean;

export type RoleChecker = (
  requiredRole: string,
  userRole: string,
  hierarchy?: Record<string, number>
) => boolean;