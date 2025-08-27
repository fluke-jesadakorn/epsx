// SIMPLIFIED ROLE SYSTEM - Matches backend unified Role enum
// Using simple role hierarchy: admin > user > guest

/**
 * Simple unified role system that matches backend Rust implementation
 */
export enum Role {
  Admin = 'admin',
  User = 'user',
  Guest = 'guest'
}

/**
 * Simple role-based permissions
 */
export const RolePermissions = {
  [Role.Admin]: [
    'read:*',
    'write:*',
    'admin:*',
    'manage:users',
    'manage:system',
    'view:analytics',
    'export:data'
  ],
  [Role.User]: [
    'read:profile',
    'write:profile',
    'view:analytics',
    'export:data'
  ],
  [Role.Guest]: [
    'read:profile',
    'view:analytics'
  ]
} as const;

/**
 * Check if a role has specific permission
 */
export function hasPermission(userRole: Role, permission: string): boolean {
  const userPermissions = RolePermissions[userRole] || [];
  
  // Check for exact match
  if (userPermissions.includes(permission)) {
    return true;
  }
  
  // Check for wildcard matches
  return userPermissions.some(userPerm => {
    const [action, resource] = userPerm.split(':');
    const [reqAction, reqResource] = permission.split(':');
    
    if (action === '*' || resource === '*') return true;
    if (action === reqAction && (resource === '*' || resource === reqResource)) return true;
    
    return false;
  });
}

/**
 * Check if user is admin
 */
export function isAdmin(role: Role): boolean {
  return role === Role.Admin;
}

/**
 * Check if user is user or higher
 */
export function isUser(role: Role): boolean {
  return role === Role.Admin || role === Role.User;
}

/**
 * Get role hierarchy level (higher number = more permissions)
 */
export function getRoleLevel(role: Role): number {
  switch (role) {
    case Role.Admin: return 3;
    case Role.User: return 2;
    case Role.Guest: return 1;
    default: return 0;
  }
}

/**
 * Check if one role is higher than another
 */
export function isRoleHigher(role1: Role, role2: Role): boolean {
  return getRoleLevel(role1) > getRoleLevel(role2);
}

/**
 * User with role information
 */
export interface UserWithRole {
  id: string;
  email: string;
  role: Role;
  createdAt: Date;
  updatedAt: Date;
}

// Legacy types for compatibility - DEPRECATED, use Role enum instead
export enum AdminRole {
  ADMIN = 'ADMIN',
  MODERATOR = 'MODERATOR',
  SUPPORT = 'SUPPORT'
}

/**
 * Convert legacy admin role to new Role enum
 */
export function convertLegacyRole(legacyRole: AdminRole | string): Role {
  const roleStr = typeof legacyRole === 'string' ? legacyRole.toUpperCase() : legacyRole;
  
  switch (roleStr) {
    case 'ADMIN':
    case AdminRole.ADMIN:
      return Role.Admin;
    case 'MODERATOR':
    case AdminRole.MODERATOR:
    case 'USER':
      return Role.User;
    case 'SUPPORT':
    case AdminRole.SUPPORT:
    case 'GUEST':
    default:
      return Role.Guest;
  }
}