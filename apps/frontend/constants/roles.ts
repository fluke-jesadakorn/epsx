// Role definitions
export const ROLES = {
  ADMIN: 'admin',
  PREMIUM: 'premium',
  BASIC: 'basic',
} as const;

export type Role = typeof ROLES[keyof typeof ROLES];

// Access level mapping
export const ROLE_ACCESS_LEVELS = {
  [ROLES.ADMIN]: 3,
  [ROLES.PREMIUM]: 2,
  [ROLES.BASIC]: 1,
} as const;

// Permission definitions
export const PERMISSIONS = {
  READ_BASIC: 'read:basic',
  READ_PREMIUM: 'read:premium',
  READ_ADMIN: 'read:admin',
  WRITE_BASIC: 'write:basic',
  WRITE_PREMIUM: 'write:premium',
  WRITE_ADMIN: 'write:admin',
  MANAGE_USERS: 'manage:users',
} as const;

export type Permission = typeof PERMISSIONS[keyof typeof PERMISSIONS];

// Role permissions mapping
export const ROLE_PERMISSIONS: Record<Role, Permission[]> = {
  [ROLES.ADMIN]: Object.values(PERMISSIONS),
  [ROLES.PREMIUM]: [
    PERMISSIONS.READ_BASIC,
    PERMISSIONS.WRITE_BASIC,
    PERMISSIONS.READ_PREMIUM,
    PERMISSIONS.WRITE_PREMIUM,
  ],
  [ROLES.BASIC]: [
    PERMISSIONS.READ_BASIC,
    PERMISSIONS.WRITE_BASIC,
  ],
};

// Helper functions
export const getRoleFromAccessLevel = (accessLevel: number): Role => {
  switch (accessLevel) {
    case 3:
      return ROLES.ADMIN;
    case 2:
      return ROLES.PREMIUM;
    default:
      return ROLES.BASIC;
  }
};

export const getPermissionsForRole = (role: Role): Permission[] => {
  return ROLE_PERMISSIONS[role] || ROLE_PERMISSIONS[ROLES.BASIC];
};

export const hasPermission = (userRole: Role, permission: Permission): boolean => {
  return ROLE_PERMISSIONS[userRole]?.includes(permission) || false;
};
