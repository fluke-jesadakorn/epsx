// Permission and role types
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

// Permission checking result
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

// Permission cache entry
export interface PermissionCacheEntry {
  permissions: string[];
  role: string;
  profiles: string[];
  isAdmin: boolean;
  timestamp: number;
  ttl: number;
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