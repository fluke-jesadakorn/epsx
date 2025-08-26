// ============================================================================
// SIMPLE ROLE SYSTEM - FRONTEND VERSION (IDENTICAL TO BACKEND)
// ============================================================================
// This file replaces 2000+ lines of complex permission code with simple role-based access
// Roles: admin, user, guest
// Features: view_eps, export_data, realtime, profile, notifications, billing, advanced_filters
// Must match backend src/auth/roles.rs exactly

// ============================================================================
// SIMPLE ROLE TYPES (MATCHES BACKEND EXACTLY)
// ============================================================================

export enum Role {
  Admin = 'admin',
  User = 'user', 
  Guest = 'guest'
}

export const roleFromString = (roleStr: string): Role => {
  switch (roleStr.toLowerCase()) {
    case 'admin': return Role.Admin;
    case 'user': return Role.User;
    case 'guest': return Role.Guest;
    default: throw new Error(`Invalid role: ${roleStr}`);
  }
};

// ============================================================================
// SIMPLE USER CLAIMS (MATCHES BACKEND)
// ============================================================================

export interface SimpleUserClaims {
  firebase_uid: string;
  email: string;
  role: Role;
  display_name?: string;
  name?: string;
  avatar_url?: string;
  is_active: boolean;
  last_login_at?: string; // ISO string
}

// ============================================================================
// SIMPLE FEATURE ACCESS LOGIC (IDENTICAL TO BACKEND)
// ============================================================================

export const checkFeatureAccess = (userRole: Role, feature: string): boolean => {
  switch (userRole) {
    case Role.Admin:
      // Admin can access everything
      return true;
      
    case Role.User:
      // User can access all premium features  
      return [
        'view_eps',
        'export_data', 
        'realtime',
        'profile',
        'notifications',
        'billing',
        'advanced_filters'
      ].includes(feature);
      
    case Role.Guest:
      // Guest can only view basic EPS data
      return feature === 'view_eps';
      
    default:
      return false;
  }
};

export const checkRoleAccess = (userRole: Role, requiredRole: Role): boolean => {
  switch (userRole) {
    case Role.Admin:
      // Admin can access everything
      return true;
      
    case Role.User:
      // User can access user and guest level
      return requiredRole === Role.User || requiredRole === Role.Guest;
      
    case Role.Guest:
      // Guest can only access guest level
      return requiredRole === Role.Guest;
      
    default:
      return false;
  }
};

// ============================================================================
// ROLE VALIDATION HELPERS (MATCHES BACKEND)
// ============================================================================

export const isAdmin = (role: Role): boolean => role === Role.Admin;

export const isUserOrAdmin = (role: Role): boolean => 
  role === Role.Admin || role === Role.User;

export const canViewEps = (role: Role): boolean =>
  role === Role.Admin || role === Role.User || role === Role.Guest;

export const canExportData = (role: Role): boolean =>
  role === Role.Admin || role === Role.User;

export const canAccessRealtime = (role: Role): boolean =>
  role === Role.Admin || role === Role.User;

export const canManageProfile = (role: Role): boolean =>
  role === Role.Admin || role === Role.User;

export const canReceiveNotifications = (role: Role): boolean =>
  role === Role.Admin || role === Role.User;

export const canManageBilling = (role: Role): boolean =>
  role === Role.Admin || role === Role.User;

export const canUseAdvancedFilters = (role: Role): boolean =>
  role === Role.Admin || role === Role.User;

// ============================================================================
// ERROR TYPES
// ============================================================================

export class RoleError extends Error {
  constructor(
    message: string,
    public readonly code: 'INSUFFICIENT_ROLE' | 'FEATURE_NOT_AVAILABLE' | 'USER_NOT_FOUND' | 'INVALID_ROLE'
  ) {
    super(message);
    this.name = 'RoleError';
  }
}

// ============================================================================
// CLIENT-SIDE ROLE UTILITIES
// ============================================================================

export const requireRole = (userClaims: SimpleUserClaims | null, requiredRole: Role): SimpleUserClaims => {
  if (!userClaims) {
    throw new RoleError('User not found', 'USER_NOT_FOUND');
  }
  
  if (!checkRoleAccess(userClaims.role, requiredRole)) {
    throw new RoleError('Access denied: insufficient role', 'INSUFFICIENT_ROLE');
  }
  
  return userClaims;
};

export const requireFeature = (userClaims: SimpleUserClaims | null, feature: string): SimpleUserClaims => {
  if (!userClaims) {
    throw new RoleError('User not found', 'USER_NOT_FOUND');
  }
  
  if (!checkFeatureAccess(userClaims.role, feature)) {
    throw new RoleError('Access denied: feature not available for role', 'FEATURE_NOT_AVAILABLE');
  }
  
  return userClaims;
};

// ============================================================================
// REACT HOOKS AND COMPONENTS INTEGRATION
// ============================================================================

export const useFeatureAccess = (userClaims: SimpleUserClaims | null, feature: string): boolean => {
  if (!userClaims) return false;
  return checkFeatureAccess(userClaims.role, feature);
};

export const useRoleAccess = (userClaims: SimpleUserClaims | null, requiredRole: Role): boolean => {
  if (!userClaims) return false;
  return checkRoleAccess(userClaims.role, requiredRole);
};

// ============================================================================
// ROLES MATRIX (FOR DOCUMENTATION)
// ============================================================================

/*
ROLES MATRIX:
+----------+----------+-------------+----------+----------+----------------+---------+-----------+
| Role     | view_eps | export_data | realtime | profile  | notifications  | billing | advanced  |
+----------+----------+-------------+----------+----------+----------------+---------+-----------+
| admin    | ✓        | ✓           | ✓        | ✓        | ✓              | ✓       | ✓         |
| user     | ✓        | ✓           | ✓        | ✓        | ✓              | ✓       | ✓         |
| guest    | ✓        | ✗           | ✗        | ✗        | ✗              | ✗       | ✗         |
+----------+----------+-------------+----------+----------+----------------+---------+-----------+

HIERARCHY: admin > user > guest
*/