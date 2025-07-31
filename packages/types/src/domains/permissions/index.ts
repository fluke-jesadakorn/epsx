// Re-export all permission types from the existing file
export * from '../../permission_profile';

// Re-export UserRole from auth/roles for compatibility
export { UserRole } from '../../auth/roles';

// Extended role enumeration for auth system
export enum AuthRole {
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

// Core permission interface
export interface Permission {
  id: string;
  name: string;
  description: string;
  resource: string;
  action: string;
  scope?: string;
}

// Role interface with permissions
export interface Role {
  id: string;
  name: string;
  description: string;
  permissions: Permission[];
  isSystem: boolean;
  level: number;
}

// Permission profile structure
export interface PermissionProfile {
  id: string;
  name: string;
  permissions: string[];
  category?: string;
  package_tier?: string;
  active: boolean;
}

// Permission checking request
export interface PermissionCheckRequest {
  userId: string;
  permission: string;
  resource?: string;
  context?: Record<string, any>;
}

// Permission checking response (consolidated from multiple sources)
export interface PermissionCheckResponse {
  allowed: boolean;
  reason?: string;
  context?: Record<string, any>;
}

// Enhanced permission checking result with detailed info
export interface PermissionCheckResult {
  allowed: boolean;
  reason: string;
  requiredPermission?: string;
  userPermissions?: string[];
  userRole?: string;
  userProfiles?: string[];
}

// Route permission configuration
export interface RoutePermissionConfig {
  permission: string;
  profile?: string;
  fallbackRole?: string;
  minimumRole?: string;
  description?: string;
}

// Permission cache entry for performance optimization
export interface PermissionCacheEntry {
  permissions: string[];
  role: string;
  profiles: string[];
  isAdmin: boolean;
  timestamp: number;
  ttl: number;
}

// User permission status with comprehensive info
export interface UserPermissionStatus {
  userId: string;
  permissions: Permission[];
  roles: Role[];
  effectivePermissions: string[];
  lastUpdated: Date;
}