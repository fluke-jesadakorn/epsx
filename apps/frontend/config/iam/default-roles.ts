import type { PermissionDocument, RoleDocument } from './types';

export const DEFAULT_PERMISSIONS: PermissionDocument[] = [
  // Dashboard permissions
  {
    id: 'dashboard.view',
    name: 'View Dashboard',
    description: 'Access to view the main dashboard',
    category: 'dashboard',
    action: 'view',
    resource: 'dashboard',
    scope: 'own',
    isSystem: true,
    tags: ['dashboard', 'view']
  },
  {
    id: 'dashboard.admin',
    name: 'Admin Dashboard',
    description: 'Access to admin dashboard features',
    category: 'dashboard',
    action: 'admin',
    resource: 'dashboard',
    scope: 'system',
    isSystem: true,
    tags: ['dashboard', 'admin']
  },

  // User management permissions
  {
    id: 'users.view',
    name: 'View Users',
    description: 'View user profiles and information',
    category: 'users',
    action: 'view',
    resource: 'users',
    scope: 'own',
    isSystem: true,
    tags: ['users', 'view']
  },
  {
    id: 'users.create',
    name: 'Create Users',
    description: 'Create new user accounts',
    category: 'users',
    action: 'create',
    resource: 'users',
    scope: 'system',
    isSystem: true,
    tags: ['users', 'create']
  },
  {
    id: 'users.update',
    name: 'Update Users',
    description: 'Update user information and settings',
    category: 'users',
    action: 'update',
    resource: 'users',
    scope: 'own',
    isSystem: true,
    tags: ['users', 'update']
  },
  {
    id: 'users.delete',
    name: 'Delete Users',
    description: 'Delete user accounts',
    category: 'users',
    action: 'delete',
    resource: 'users',
    scope: 'system',
    isSystem: true,
    tags: ['users', 'delete']
  },

  // Role management permissions
  {
    id: 'roles.view',
    name: 'View Roles',
    description: 'View role definitions and permissions',
    category: 'roles',
    action: 'view',
    resource: 'roles',
    scope: 'system',
    isSystem: true,
    tags: ['roles', 'view']
  },
  {
    id: 'roles.create',
    name: 'Create Roles',
    description: 'Create new roles and assign permissions',
    category: 'roles',
    action: 'create',
    resource: 'roles',
    scope: 'system',
    isSystem: true,
    tags: ['roles', 'create']
  },
  {
    id: 'roles.update',
    name: 'Update Roles',
    description: 'Update role definitions and permissions',
    category: 'roles',
    action: 'update',
    resource: 'roles',
    scope: 'system',
    isSystem: true,
    tags: ['roles', 'update']
  },
  {
    id: 'roles.delete',
    name: 'Delete Roles',
    description: 'Delete role definitions',
    category: 'roles',
    action: 'delete',
    resource: 'roles',
    scope: 'system',
    isSystem: true,
    tags: ['roles', 'delete']
  },

  // Analytics permissions
  {
    id: 'analytics.view',
    name: 'View Analytics',
    description: 'Access to view analytics and reports',
    category: 'analytics',
    action: 'view',
    resource: 'analytics',
    scope: 'own',
    isSystem: true,
    tags: ['analytics', 'view']
  },
  {
    id: 'analytics.admin',
    name: 'Admin Analytics',
    description: 'Access to admin analytics and system reports',
    category: 'analytics',
    action: 'admin',
    resource: 'analytics',
    scope: 'system',
    isSystem: true,
    tags: ['analytics', 'admin']
  },

  // Package permissions
  {
    id: 'packages.view',
    name: 'View Packages',
    description: 'View available packages and features',
    category: 'packages',
    action: 'view',
    resource: 'packages',
    scope: 'own',
    isSystem: true,
    tags: ['packages', 'view']
  },
  {
    id: 'packages.create',
    name: 'Create Packages',
    description: 'Create new packages and features',
    category: 'packages',
    action: 'create',
    resource: 'packages',
    scope: 'system',
    isSystem: true,
    tags: ['packages', 'create']
  },
  {
    id: 'packages.update',
    name: 'Update Packages',
    description: 'Update package configurations',
    category: 'packages',
    action: 'update',
    resource: 'packages',
    scope: 'system',
    isSystem: true,
    tags: ['packages', 'update']
  },
  {
    id: 'packages.delete',
    name: 'Delete Packages',
    description: 'Delete packages and features',
    category: 'packages',
    action: 'delete',
    resource: 'packages',
    scope: 'system',
    isSystem: true,
    tags: ['packages', 'delete']
  },

  // Payment permissions
  {
    id: 'payments.view',
    name: 'View Payments',
    description: 'View payment history and transactions',
    category: 'payments',
    action: 'view',
    resource: 'payments',
    scope: 'own',
    isSystem: true,
    tags: ['payments', 'view']
  },
  {
    id: 'payments.create',
    name: 'Create Payments',
    description: 'Create new payment transactions',
    category: 'payments',
    action: 'create',
    resource: 'payments',
    scope: 'own',
    isSystem: true,
    tags: ['payments', 'create']
  },
  {
    id: 'payments.admin',
    name: 'Admin Payments',
    description: 'Manage all payment transactions',
    category: 'payments',
    action: 'admin',
    resource: 'payments',
    scope: 'system',
    isSystem: true,
    tags: ['payments', 'admin']
  }
];

export const DEFAULT_ROLES: RoleDocument[] = [
  {
    id: 'super_admin',
    name: 'Super Admin',
    description: 'Full system access with all permissions',
    color: '#dc2626',
    icon: 'Shield',
    permissions: ['*'], // All permissions
    isSystem: true,
    isActive: true
  },
  {
    id: 'admin',
    name: 'Admin',
    description: 'Administrative access to manage users, roles, and system settings',
    color: '#ea580c',
    icon: 'UserCog',
    permissions: [
      'dashboard.admin',
      'users.view',
      'users.create',
      'users.update',
      'roles.view',
      'roles.create',
      'roles.update',
      'analytics.admin',
      'packages.view',
      'packages.create',
      'packages.update',
      'payments.admin'
    ],
    isSystem: true,
    isActive: true
  },
  {
    id: 'manager',
    name: 'Manager',
    description: 'Management access to view and manage team members',
    color: '#ca8a04',
    icon: 'Users',
    permissions: [
      'dashboard.view',
      'users.view',
      'users.create',
      'users.update',
      'analytics.view',
      'packages.view',
      'payments.view',
      'payments.create'
    ],
    isSystem: true,
    isActive: true
  },
  {
    id: 'user',
    name: 'User',
    description: 'Standard user with basic access to dashboard and personal data',
    color: '#16a34a',
    icon: 'User',
    permissions: [
      'dashboard.view',
      'users.view',
      'analytics.view',
      'packages.view',
      'payments.view',
      'payments.create'
    ],
    isSystem: true,
    isActive: true
  },
  {
    id: 'viewer',
    name: 'Viewer',
    description: 'Read-only access to view dashboard and analytics',
    color: '#2563eb',
    icon: 'Eye',
    permissions: [
      'dashboard.view',
      'analytics.view',
      'packages.view'
    ],
    isSystem: true,
    isActive: true
  }
];
