// ============================================================================
// PERMISSION-ONLY SYSTEM - FRONTEND TYPES (MATCHES BACKEND)
// ============================================================================
// Pure permission-based access control using structured permissions
// Format: "platform:resource:action" (e.g., "epsx:analytics:view", "admin:users:manage")

// ============================================================================
// PERMISSION TYPES
// ============================================================================

export interface Permission {
  platform: string;
  resource: string;
  action: string;
}

export interface UserClaims {
  firebase_uid: string;
  email: string;
  permissions: string[];  // Structured permissions: ["platform:resource:action", ...]
  package_tier: string;
  display_name?: string;
  name?: string;
  avatar_url?: string;
  is_active: boolean;
  last_login_at?: string;
  platforms?: string[];
  primary_platform?: string;
  platform_context?: string;
}

// ============================================================================
// PERMISSION PARSING AND VALIDATION
// ============================================================================

export const parsePermission = (permissionString: string): Permission | null => {
  const parts = permissionString.split(':');
  if (parts.length !== 3) return null;
  
  return {
    platform: parts[0],
    resource: parts[1],
    action: parts[2]
  };
};

export const buildPermission = (platform: string, resource: string, action: string): string => {
  return `${platform}:${resource}:${action}`;
};

// ============================================================================
// PERMISSION CHECKING LOGIC
// ============================================================================

export const checkPermissionAccess = (userPermissions: string[], requiredPermission: string): boolean => {
  const required = parsePermission(requiredPermission);
  if (!required) return false;
  
  for (const permStr of userPermissions) {
    const userPerm = parsePermission(permStr);
    if (!userPerm) continue;
    
    // Check for exact match
    if (userPerm.platform === required.platform && 
        userPerm.resource === required.resource && 
        userPerm.action === required.action) {
      return true;
    }
    
    // Check for wildcard matches
    if (userPerm.platform === required.platform) {
      // Platform-level wildcard: "epsx:*:*"
      if (userPerm.resource === '*' && userPerm.action === '*') {
        return true;
      }
      
      // Resource-level wildcard: "epsx:analytics:*"
      if (userPerm.resource === required.resource && userPerm.action === '*') {
        return true;
      }
    }
    
    // Global admin permission: "admin:*:*"
    if (userPerm.platform === 'admin' && userPerm.resource === '*' && userPerm.action === '*') {
      return true;
    }
  }
  
  return false;
};

export const hasPermission = (userClaims: UserClaims | null, permission: string): boolean => {
  if (!userClaims) return false;
  return checkPermissionAccess(userClaims.permissions, permission);
};

export const hasAnyPermission = (userClaims: UserClaims | null, permissions: string[]): boolean => {
  if (!userClaims) return false;
  return permissions.some(permission => checkPermissionAccess(userClaims.permissions, permission));
};

export const hasAllPermissions = (userClaims: UserClaims | null, permissions: string[]): boolean => {
  if (!userClaims) return false;
  return permissions.every(permission => checkPermissionAccess(userClaims.permissions, permission));
};

// ============================================================================
// PLATFORM PERMISSION HELPERS
// ============================================================================

export const hasPlatformPermission = (userClaims: UserClaims | null, platform: string, resource: string, action: string): boolean => {
  const permission = buildPermission(platform, resource, action);
  return hasPermission(userClaims, permission);
};

export const canAccessPlatform = (userClaims: UserClaims | null, platform: string): boolean => {
  if (!userClaims) return false;
  
  // Check if platform is in user's accessible platforms
  if (userClaims.platforms?.includes(platform)) {
    return true;
  }
  
  // Check if user has any permissions for this platform
  return userClaims.permissions.some(perm => perm.startsWith(`${platform}:`));
};

export const getPlatformPermissions = (userClaims: UserClaims | null, platform: string): string[] => {
  if (!userClaims) return [];
  return userClaims.permissions.filter(perm => perm.startsWith(`${platform}:`));
};

// ============================================================================
// FEATURE ACCESS HELPERS (LEGACY COMPATIBILITY)
// ============================================================================

export const canViewAnalytics = (userClaims: UserClaims | null): boolean => {
  return hasPermission(userClaims, "epsx:analytics:view");
};

export const canExportData = (userClaims: UserClaims | null): boolean => {
  return hasPermission(userClaims, "epsx:analytics:export");
};

export const canAccessRealtime = (userClaims: UserClaims | null): boolean => {
  return hasPermission(userClaims, "epsx:realtime:access");
};

export const canManageProfile = (userClaims: UserClaims | null): boolean => {
  return hasPermission(userClaims, "epsx:profile:manage");
};

export const canReceiveNotifications = (userClaims: UserClaims | null): boolean => {
  return hasPermission(userClaims, "epsx:notifications:receive");
};

export const canManageBilling = (userClaims: UserClaims | null): boolean => {
  return hasPermission(userClaims, "epsx:billing:manage");
};

export const canUseAdvancedFilters = (userClaims: UserClaims | null): boolean => {
  return hasPermission(userClaims, "epsx:analytics:advanced");
};

// ============================================================================
// ADMIN PERMISSION HELPERS
// ============================================================================

export const isAdmin = (userClaims: UserClaims | null): boolean => {
  return hasPermission(userClaims, "admin:*:*");
};

export const canManageUsers = (userClaims: UserClaims | null): boolean => {
  return hasAnyPermission(userClaims, ["admin:users:manage", "epsx:users:manage"]);
};

export const canViewUsers = (userClaims: UserClaims | null): boolean => {
  return hasAnyPermission(userClaims, ["admin:users:read", "epsx:users:read", "admin:users:manage", "epsx:users:manage"]);
};

export const canManageSystem = (userClaims: UserClaims | null): boolean => {
  return hasAnyPermission(userClaims, ["admin:system:manage", "admin:*:*"]);
};

export const canViewAuditLogs = (userClaims: UserClaims | null): boolean => {
  return hasAnyPermission(userClaims, ["admin:audit:read", "epsx:audit:read", "admin:*:*"]);
};

// ============================================================================
// PERMISSION SETS (REPLACE ROLE-BASED LOGIC)
// ============================================================================

export const PERMISSION_SETS = {
  ADMIN: [
    "admin:*:*"
  ] as const,
  
  PREMIUM_USER: [
    "epsx:analytics:view",
    "epsx:analytics:export", 
    "epsx:analytics:advanced",
    "epsx:realtime:access",
    "epsx:profile:manage",
    "epsx:notifications:receive",
    "epsx:billing:manage"
  ] as const,
  
  BASIC_USER: [
    "epsx:analytics:view",
    "epsx:profile:manage",
    "epsx:notifications:receive"
  ] as const
} as const;

export type PermissionSet = typeof PERMISSION_SETS[keyof typeof PERMISSION_SETS];

// ============================================================================
// ERROR TYPES
// ============================================================================

export class PermissionError extends Error {
  constructor(
    message: string,
    public readonly code: 'INSUFFICIENT_PERMISSION' | 'FEATURE_NOT_AVAILABLE' | 'USER_NOT_FOUND' | 'INVALID_PERMISSION'
  ) {
    super(message);
    this.name = 'PermissionError';
  }
}

// ============================================================================
// VALIDATION UTILITIES
// ============================================================================

export const requirePermission = (userClaims: UserClaims | null, permission: string): UserClaims => {
  if (!userClaims) {
    throw new PermissionError('User not found', 'USER_NOT_FOUND');
  }
  
  if (!hasPermission(userClaims, permission)) {
    throw new PermissionError(`Access denied: missing permission ${permission}`, 'INSUFFICIENT_PERMISSION');
  }
  
  return userClaims;
};

export const requireAnyPermission = (userClaims: UserClaims | null, permissions: string[]): UserClaims => {
  if (!userClaims) {
    throw new PermissionError('User not found', 'USER_NOT_FOUND');
  }
  
  if (!hasAnyPermission(userClaims, permissions)) {
    throw new PermissionError(`Access denied: missing any of permissions ${permissions.join(', ')}`, 'INSUFFICIENT_PERMISSION');
  }
  
  return userClaims;
};

export const isValidPermission = (permission: string): boolean => {
  const parsed = parsePermission(permission);
  return parsed !== null && parsed.platform.length > 0 && parsed.resource.length > 0 && parsed.action.length > 0;
};

// ============================================================================
// LEGACY COMPATIBILITY (FOR GRADUAL MIGRATION)
// ============================================================================

// Map old role-based checks to permission-based checks
export const checkFeatureAccess = (userClaims: UserClaims | null, feature: string): boolean => {
  const featurePermissionMap: Record<string, string> = {
    'view_eps': 'epsx:analytics:view',
    'export_data': 'epsx:analytics:export',
    'realtime': 'epsx:realtime:access',
    'profile': 'epsx:profile:manage',
    'notifications': 'epsx:notifications:receive',
    'billing': 'epsx:billing:manage',
    'advanced_filters': 'epsx:analytics:advanced'
  };
  
  const permission = featurePermissionMap[feature] || feature;
  return hasPermission(userClaims, permission);
};

// ============================================================================
// REACT HOOKS AND COMPONENTS INTEGRATION
// ============================================================================

export const usePermission = (userClaims: UserClaims | null, permission: string): boolean => {
  return hasPermission(userClaims, permission);
};

export const useAnyPermission = (userClaims: UserClaims | null, permissions: string[]): boolean => {
  return hasAnyPermission(userClaims, permissions);
};

export const useFeatureAccess = (userClaims: UserClaims | null, feature: string): boolean => {
  return checkFeatureAccess(userClaims, feature);
};

export const usePlatformAccess = (userClaims: UserClaims | null, platform: string): boolean => {
  return canAccessPlatform(userClaims, platform);
};

// ============================================================================
// EMBEDDED TIMESTAMP PERMISSION SYSTEM
// ============================================================================

export interface TimestampedPermission {
  permission: string;
  basePermission: string;
  expiresAt?: number; // Unix timestamp
  isExpired: boolean;
  expiresIn?: string; // Human readable (e.g., "2 hours")
  timeRemaining?: number; // Milliseconds remaining
}

export interface PermissionExpiryInfo {
  hasExpiringPermissions: boolean;
  expiringSoon: TimestampedPermission[]; // Expiring within 24 hours
  expired: TimestampedPermission[];
  nextExpiry?: TimestampedPermission;
}

/**
 * Parse permission with embedded timestamp
 * Format: "platform:resource:action:unix_timestamp"
 * Returns: { basePermission, timestamp }
 */
export const parsePermissionWithTimestamp = (permission: string): { basePermission: string; timestamp?: number } => {
  const parts = permission.split(':');
  if (parts.length >= 4) {
    const lastPart = parts[parts.length - 1];
    const timestamp = parseInt(lastPart, 10);
    if (!isNaN(timestamp)) {
      const basePermission = parts.slice(0, -1).join(':');
      return { basePermission, timestamp };
    }
  }
  return { basePermission: permission };
};

/**
 * Check if a permission with timestamp is still valid
 */
export const isPermissionValidWithTime = (permission: string): boolean => {
  const { timestamp } = parsePermissionWithTimestamp(permission);
  if (!timestamp) return true; // No timestamp means permanent permission
  
  const now = Math.floor(Date.now() / 1000); // Current Unix timestamp
  return timestamp > now;
};

/**
 * Convert permission to timestamped permission object
 */
export const createTimestampedPermission = (permission: string): TimestampedPermission => {
  const { basePermission, timestamp } = parsePermissionWithTimestamp(permission);
  const now = Date.now();
  const expiresAt = timestamp;
  
  let isExpired = false;
  let expiresIn: string | undefined;
  let timeRemaining: number | undefined;
  
  if (expiresAt) {
    const expiryTime = expiresAt * 1000; // Convert to milliseconds
    isExpired = expiryTime <= now;
    timeRemaining = Math.max(0, expiryTime - now);
    
    if (!isExpired && timeRemaining) {
      const seconds = Math.floor(timeRemaining / 1000);
      const minutes = Math.floor(seconds / 60);
      const hours = Math.floor(minutes / 60);
      const days = Math.floor(hours / 24);
      
      if (days > 0) {
        expiresIn = `${days} day${days > 1 ? 's' : ''}`;
      } else if (hours > 0) {
        expiresIn = `${hours} hour${hours > 1 ? 's' : ''}`;
      } else if (minutes > 0) {
        expiresIn = `${minutes} minute${minutes > 1 ? 's' : ''}`;
      } else {
        expiresIn = `${seconds} second${seconds > 1 ? 's' : ''}`;
      }
    }
  }
  
  return {
    permission,
    basePermission,
    expiresAt,
    isExpired,
    expiresIn,
    timeRemaining
  };
};

/**
 * Filter out expired permissions from a permission array
 */
export const filterValidPermissions = (permissions: string[]): string[] => {
  return permissions.filter(isPermissionValidWithTime);
};

/**
 * Get expiry information for user's permissions
 */
export const getPermissionExpiryInfo = (userClaims: UserClaims | null): PermissionExpiryInfo => {
  if (!userClaims) {
    return {
      hasExpiringPermissions: false,
      expiringSoon: [],
      expired: []
    };
  }
  
  const timestampedPermissions = userClaims.permissions.map(createTimestampedPermission);
  const now = Date.now();
  const twentyFourHoursFromNow = now + (24 * 60 * 60 * 1000);
  
  const expired = timestampedPermissions.filter(p => p.isExpired);
  const expiringSoon = timestampedPermissions.filter(p => 
    !p.isExpired && 
    p.expiresAt && 
    (p.expiresAt * 1000) <= twentyFourHoursFromNow
  );
  
  const nextExpiry = timestampedPermissions
    .filter(p => !p.isExpired && p.expiresAt)
    .sort((a, b) => (a.expiresAt || 0) - (b.expiresAt || 0))[0];
  
  return {
    hasExpiringPermissions: expiringSoon.length > 0,
    expiringSoon,
    expired,
    nextExpiry
  };
};

/**
 * Enhanced permission checking with timestamp validation
 */
export const checkPermissionAccessWithTime = (userPermissions: string[], requiredPermission: string): boolean => {
  // First filter out expired permissions
  const validPermissions = filterValidPermissions(userPermissions);
  
  // Then use the existing permission checking logic
  return checkPermissionAccess(validPermissions, requiredPermission);
};

/**
 * Enhanced hasPermission with timestamp validation
 */
export const hasPermissionWithTime = (userClaims: UserClaims | null, permission: string): boolean => {
  if (!userClaims) return false;
  return checkPermissionAccessWithTime(userClaims.permissions, permission);
};

/**
 * Add timestamp to permission string
 */
export const addTimestampToPermission = (permission: string, expiresAt: number): string => {
  return `${permission}:${expiresAt}`;
};

/**
 * Create permission string with relative expiry (e.g., "1 hour from now")
 */
export const createPermissionWithRelativeExpiry = (
  permission: string, 
  duration: number, 
  unit: 'minutes' | 'hours' | 'days' | 'weeks' = 'hours'
): string => {
  const now = Math.floor(Date.now() / 1000);
  let seconds: number;
  
  switch (unit) {
    case 'minutes':
      seconds = duration * 60;
      break;
    case 'hours':
      seconds = duration * 60 * 60;
      break;
    case 'days':
      seconds = duration * 24 * 60 * 60;
      break;
    case 'weeks':
      seconds = duration * 7 * 24 * 60 * 60;
      break;
  }
  
  const expiresAt = now + seconds;
  return addTimestampToPermission(permission, expiresAt);
};

/**
 * Check if user has permission that will be valid for at least the specified duration
 */
export const hasPermissionForDuration = (
  userClaims: UserClaims | null, 
  permission: string, 
  durationMinutes: number = 0
): boolean => {
  if (!userClaims) return false;
  
  const now = Date.now();
  const requiredValidUntil = now + (durationMinutes * 60 * 1000);
  
  for (const userPerm of userClaims.permissions) {
    const timestamped = createTimestampedPermission(userPerm);
    
    // Check if permission matches
    if (checkPermissionAccess([timestamped.basePermission], permission)) {
      // If no timestamp, it's permanent
      if (!timestamped.expiresAt) return true;
      
      // Check if it's valid for the required duration
      const expiryTime = timestamped.expiresAt * 1000;
      if (expiryTime >= requiredValidUntil) return true;
    }
  }
  
  return false;
};

/**
 * Get time until next permission expiry
 */
export const getTimeUntilNextExpiry = (userClaims: UserClaims | null): number | null => {
  const expiryInfo = getPermissionExpiryInfo(userClaims);
  return expiryInfo.nextExpiry?.timeRemaining || null;
};

/**
 * Format expiry time as human-readable string
 */
export const formatExpiryTime = (expiresAt: number): string => {
  const expiryDate = new Date(expiresAt * 1000);
  const now = new Date();
  
  if (expiryDate <= now) {
    return 'Expired';
  }
  
  const diffMs = expiryDate.getTime() - now.getTime();
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
  const diffDays = Math.floor(diffHours / 24);
  
  if (diffDays > 0) {
    return `Expires in ${diffDays} day${diffDays > 1 ? 's' : ''}`;
  } else if (diffHours > 0) {
    return `Expires in ${diffHours} hour${diffHours > 1 ? 's' : ''}`;
  } else {
    const diffMinutes = Math.floor(diffMs / (1000 * 60));
    return `Expires in ${diffMinutes} minute${diffMinutes > 1 ? 's' : ''}`;
  }
};

// ============================================================================
// ENHANCED HELPER FUNCTIONS WITH TIMESTAMP SUPPORT
// ============================================================================

/**
 * Enhanced feature access checking with timestamp validation
 */
export const checkFeatureAccessWithTime = (userClaims: UserClaims | null, feature: string): boolean => {
  const featurePermissionMap: Record<string, string> = {
    'view_eps': 'epsx:analytics:view',
    'export_data': 'epsx:analytics:export',
    'realtime': 'epsx:realtime:access',
    'profile': 'epsx:profile:manage',
    'notifications': 'epsx:notifications:receive',
    'billing': 'epsx:billing:manage',
    'advanced_filters': 'epsx:analytics:advanced'
  };
  
  const permission = featurePermissionMap[feature] || feature;
  return hasPermissionWithTime(userClaims, permission);
};

/**
 * Enhanced admin checking with timestamp validation
 */
export const isAdminWithTime = (userClaims: UserClaims | null): boolean => {
  return hasPermissionWithTime(userClaims, "admin:*:*");
};

/**
 * Enhanced user management permission checking with timestamps
 */
export const canManageUsersWithTime = (userClaims: UserClaims | null): boolean => {
  return hasPermissionWithTime(userClaims, "admin:users:manage") || 
         hasPermissionWithTime(userClaims, "epsx:users:manage");
};

// ============================================================================
// PERMISSION MATRIX (FOR DOCUMENTATION AND UI)
// ============================================================================

export const PERMISSION_MATRIX = {
  admin: [
    "admin:*:*"
  ],
  premium_user: [
    "epsx:analytics:view",
    "epsx:analytics:export", 
    "epsx:analytics:advanced",
    "epsx:realtime:access",
    "epsx:profile:manage",
    "epsx:notifications:receive",
    "epsx:billing:manage"
  ],
  basic_user: [
    "epsx:analytics:view",
    "epsx:profile:manage",
    "epsx:notifications:receive"
  ]
} as const;

/*
PERMISSION MATRIX:
+----------+-------------------+-------------------+-------------------+-------------------+
| Tier     | Analytics         | Real-time         | Profile           | Admin             |
+----------+-------------------+-------------------+-------------------+-------------------+
| admin    | epsx:*:* (all)    | epsx:*:* (all)    | epsx:*:* (all)    | admin:*:*         |
| premium  | epsx:analytics:*  | epsx:realtime:*   | epsx:profile:*    | ✗                 |
| basic    | epsx:analytics:view| ✗                 | epsx:profile:*    | ✗                 |
+----------+-------------------+-------------------+-------------------+-------------------+

FORMAT: platform:resource:action
PLATFORMS: epsx, epsx-pay, epsx-token, admin
WILDCARDS: * matches any resource/action at that level
*/