export enum AdminRole {
  SUPER_ADMIN = 'SUPER_ADMIN',
  ADMIN = 'ADMIN',
  MODERATOR = 'MODERATOR',
  SUPPORT = 'SUPPORT'
}

export interface AdminPermission {
  resource: string;
  actions: string[]; // ['read', 'write', 'delete', 'manage']
}

export interface AdminUser {
  id: string;
  email: string;
  role: AdminRole;
  permissions: AdminPermission[];
  assignedBy: string;
  assignedAt: Date;
  isActive: boolean;
}

export const ADMIN_PERMISSIONS: Record<AdminRole, AdminPermission[]> = {
  [AdminRole.SUPER_ADMIN]: [
    { resource: '*', actions: ['*'] }
  ],
  [AdminRole.ADMIN]: [
    { resource: 'users', actions: ['read', 'write', 'delete'] },
    { resource: 'payments', actions: ['read', 'write'] },
    { resource: 'system', actions: ['read', 'write'] },
    { resource: 'analytics', actions: ['read'] }
  ],
  [AdminRole.MODERATOR]: [
    { resource: 'users', actions: ['read', 'write'] },
    { resource: 'content', actions: ['read', 'write', 'delete'] },
    { resource: 'reports', actions: ['read', 'write'] }
  ],
  [AdminRole.SUPPORT]: [
    { resource: 'users', actions: ['read'] },
    { resource: 'tickets', actions: ['read', 'write'] },
    { resource: 'payments', actions: ['read'] }
  ]
};
