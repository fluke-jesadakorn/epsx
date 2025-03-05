export enum UserRole {
  ADMIN = 'admin',
  PREMIUM = 'premium',
  BASIC = 'basic',
  PUBLIC = 'public',
}

export const ROLE_ACCESS_LEVELS = {
  [UserRole.ADMIN]: 4,
  [UserRole.PREMIUM]: 3,
  [UserRole.BASIC]: 2,
  [UserRole.PUBLIC]: 1,
} as const;

// Permission definitions
export const PERMISSIONS = {
  VIEW_ALL_RANKS: 'view:all_ranks',
  VIEW_BASIC_RANKS: 'view:basic_ranks',
  VIEW_PUBLIC_RANKS: 'view:public_ranks',
} as const;

type Permission = typeof PERMISSIONS[keyof typeof PERMISSIONS];

// Role permissions mapping
export const ROLE_PERMISSIONS: Record<UserRole, Permission[]> = {
  [UserRole.ADMIN]: [
    PERMISSIONS.VIEW_ALL_RANKS,
    PERMISSIONS.VIEW_BASIC_RANKS,
    PERMISSIONS.VIEW_PUBLIC_RANKS,
  ],
  [UserRole.PREMIUM]: [
    PERMISSIONS.VIEW_ALL_RANKS,
    PERMISSIONS.VIEW_BASIC_RANKS,
    PERMISSIONS.VIEW_PUBLIC_RANKS,
  ],
  [UserRole.BASIC]: [
    PERMISSIONS.VIEW_BASIC_RANKS,
    PERMISSIONS.VIEW_PUBLIC_RANKS,
  ],
  [UserRole.PUBLIC]: [
    PERMISSIONS.VIEW_PUBLIC_RANKS,
  ],
};
