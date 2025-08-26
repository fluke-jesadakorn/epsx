// UNIFIED PERMISSION SYSTEM - Single source of truth for all frontend permissions
// Uses unified format: "domain:action:scope" matching backend Rust implementation

import type { PermissionDocument, RoleDocument } from './types';
import { 
  PermissionPatterns, 
  PermissionDomain, 
  PermissionAction, 
  PermissionScope 
} from '../../types/permissions';

// NEW: Unified permissions using consistent domain:action:scope format
export const UNIFIED_PERMISSIONS: PermissionDocument[] = [
  // Dashboard permissions
  {
    id: 'dashboard:view:own',
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
    id: 'dashboard:admin:system',
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
    id: 'users:read:own',
    name: 'View Users',
    description: 'View user profiles and information',
    category: 'users',
    action: 'read',
    resource: 'users',
    scope: 'own',
    isSystem: true,
    tags: ['users', 'read']
  },
  {
    id: 'users:create:system',
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
    id: 'users:update:own',
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
    id: 'users:delete:system',
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
    id: 'roles:read:system',
    name: 'View Roles',
    description: 'View role definitions and permissions',
    category: 'roles',
    action: 'read',
    resource: 'roles',
    scope: 'system',
    isSystem: true,
    tags: ['roles', 'read']
  },
  {
    id: 'roles:create:system',
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
    id: 'roles:update:system',
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
    id: 'roles:delete:system',
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
    id: 'analytics:view:own',
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
    id: 'analytics:admin:system',
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
    id: 'packages:view:own',
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
    id: 'packages:create:system',
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
    id: 'packages:update:system',
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
    id: 'packages:delete:system',
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
    id: 'payments:view:own',
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
    id: 'payments:create:own',
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
    id: 'payments:admin:system',
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

// NEW: Unified roles using unified permission IDs
export const UNIFIED_ROLES: RoleDocument[] = [
  {
    id: 'admin',
    name: 'Admin',
    description: 'Full system access with all permissions',
    color: '#dc2626',
    icon: 'Shield',
    permissions: ['*:*:*'], // All permissions using unified wildcard
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
      'dashboard:view:own',
      'users:read:team',
      'users:create:team',
      'users:update:team',
      'analytics:view:own',
      'packages:view:own',
      'payments:view:own',
      'payments:create:own'
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
      'dashboard:view:own',
      'users:read:own',
      'analytics:view:own',
      'packages:view:own',
      'payments:view:own',
      'payments:create:own'
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
      'dashboard:view:own',
      'analytics:view:own',
      'packages:view:own'
    ],
    isSystem: true,
    isActive: true
  }
];

// Backward compatibility: Export legacy permissions with conversion warning
export const DEFAULT_PERMISSIONS = UNIFIED_PERMISSIONS;
export const DEFAULT_ROLES = UNIFIED_ROLES;