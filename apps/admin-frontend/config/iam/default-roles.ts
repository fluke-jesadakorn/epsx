// Default system roles and permissions
// These are seeded into Firestore on system initialization

import type { RoleDocument, PermissionDocument } from '../../types/iam/firestore-iam';
import { PermissionCategory, PermissionScope } from '../../types/iam/firestore-iam';

export const DEFAULT_ROLES: RoleDocument[] = [
  {
    id: 'super_admin',
    name: 'Super Administrator',
    description: 'Full system access with all permissions',
    color: '#DC2626',
    icon: '👑',
    permissions: ['*'], // Wildcard for all permissions
    isSystem: true,
    isActive: true,
    createdBy: 'system',
    createdAt: new Date(),
    updatedAt: new Date(),
    usageCount: 0,
    metadata: {
      category: 'system',
      tags: ['admin', 'full-access'],
      version: 1
    }
  },
  {
    id: 'admin',
    name: 'Administrator',
    description: 'Administrative access to manage users, roles, and system settings',
    color: '#EA580C',
    icon: '⚙️',
    permissions: [
      'admin.users.view',
      'admin.users.manage',
      'admin.roles.view',
      'admin.roles.manage',
      'admin.permissions.view',
      'admin.permissions.manage',
      'admin.billing.view',
      'admin.billing.manage',
      'dashboard.view',
      'dashboard.advanced',
      'dashboard.customize',
      'dashboard.export',
      'api.unlimited',
      'data.view',
      'data.export',
      'data.modify',
      'data.delete',
      'analytics.advanced',
      'analytics.custom',
      'integration.webhook',
      'integration.api_key',
      'integration.partner'
    ],
    isSystem: true,
    isActive: true,
    createdBy: 'system',
    createdAt: new Date(),
    updatedAt: new Date(),
    usageCount: 0,
    metadata: {
      category: 'system',
      tags: ['admin', 'management'],
      version: 1
    }
  },
  {
    id: 'manager',
    name: 'Manager',
    description: 'Management access for team oversight and analytics',
    color: '#059669',
    icon: '👥',
    permissions: [
      'dashboard.view',
      'dashboard.advanced',
      'dashboard.export',
      'admin.users.view',
      'api.company',
      'data.view',
      'data.export',
      'analytics.advanced',
      'analytics.custom',
      'integration.webhook'
    ],
    isSystem: true,
    isActive: true,
    createdBy: 'system',
    createdAt: new Date(),
    updatedAt: new Date(),
    usageCount: 0,
    metadata: {
      category: 'system',
      tags: ['management', 'team-lead'],
      version: 1
    }
  },
  {
    id: 'user',
    name: 'Standard User',
    description: 'Basic user access with package-based permissions',
    color: '#2563EB',
    icon: '👤',
    permissions: [], // Package-based permissions are calculated dynamically
    isSystem: true,
    isActive: true,
    createdBy: 'system',
    createdAt: new Date(),
    updatedAt: new Date(),
    usageCount: 0,
    metadata: {
      category: 'system',
      tags: ['user', 'standard'],
      version: 1
    }
  },
  {
    id: 'viewer',
    name: 'Viewer',
    description: 'Read-only access to basic features',
    color: '#7C3AED',
    icon: '👁️',
    permissions: [
      'dashboard.view',
      'data.view',
      'analytics.basic'
    ],
    isSystem: true,
    isActive: true,
    createdBy: 'system',
    createdAt: new Date(),
    updatedAt: new Date(),
    usageCount: 0,
    metadata: {
      category: 'system',
      tags: ['viewer', 'read-only'],
      version: 1
    }
  }
];

export const DEFAULT_PERMISSIONS: PermissionDocument[] = [
  // Dashboard Permissions
  {
    id: 'dashboard.view',
    name: 'View Dashboard',
    description: 'Access to view the main dashboard',
    category: PermissionCategory.DASHBOARD,
    action: 'view',
    resource: 'dashboard',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['dashboard', 'basic'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'dashboard.advanced',
    name: 'Advanced Dashboard',
    description: 'Access to advanced dashboard features and analytics',
    category: PermissionCategory.DASHBOARD,
    action: 'view',
    resource: 'dashboard.advanced',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['dashboard', 'advanced'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'dashboard.customize',
    name: 'Customize Dashboard',
    description: 'Ability to customize dashboard layout and widgets',
    category: PermissionCategory.DASHBOARD,
    action: 'customize',
    resource: 'dashboard',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['dashboard', 'customize'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'dashboard.export',
    name: 'Export Dashboard',
    description: 'Export dashboard data and reports',
    category: PermissionCategory.DASHBOARD,
    action: 'export',
    resource: 'dashboard',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['dashboard', 'export'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },

  // API Permissions
  {
    id: 'api.personal',
    name: 'Personal API Access',
    description: 'Access to personal API endpoints',
    category: PermissionCategory.API,
    action: 'access',
    resource: 'api.personal',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['api', 'personal'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'api.company',
    name: 'Company API Access',
    description: 'Access to company-wide API endpoints',
    category: PermissionCategory.API,
    action: 'access',
    resource: 'api.company',
    scope: PermissionScope.COMPANY,
    isSystem: true,
    tags: ['api', 'company'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'api.partner',
    name: 'Partner API Access',
    description: 'Access to partner-level API endpoints',
    category: PermissionCategory.API,
    action: 'access',
    resource: 'api.partner',
    scope: PermissionScope.PARTNER,
    isSystem: true,
    tags: ['api', 'partner'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'api.unlimited',
    name: 'Unlimited API Access',
    description: 'Unlimited API access without rate limits',
    category: PermissionCategory.API,
    action: 'access',
    resource: 'api.unlimited',
    scope: PermissionScope.GLOBAL,
    isSystem: true,
    tags: ['api', 'unlimited'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },

  // Data Permissions
  {
    id: 'data.view',
    name: 'View Data',
    description: 'View accessible data based on scope',
    category: PermissionCategory.DATA,
    action: 'view',
    resource: 'data',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['data', 'view'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'data.export',
    name: 'Export Data',
    description: 'Export data in various formats',
    category: PermissionCategory.DATA,
    action: 'export',
    resource: 'data',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['data', 'export'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'data.modify',
    name: 'Modify Data',
    description: 'Edit and update existing data',
    category: PermissionCategory.DATA,
    action: 'modify',
    resource: 'data',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['data', 'modify'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'data.delete',
    name: 'Delete Data',
    description: 'Permanently delete data',
    category: PermissionCategory.DATA,
    action: 'delete',
    resource: 'data',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['data', 'delete'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },

  // Admin Permissions
  {
    id: 'admin.users.view',
    name: 'View Users',
    description: 'View user management interface',
    category: PermissionCategory.ADMIN,
    action: 'view',
    resource: 'admin.users',
    scope: PermissionScope.GLOBAL,
    isSystem: true,
    tags: ['admin', 'users', 'view'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'admin.users.manage',
    name: 'Manage Users',
    description: 'Create, edit, and delete users',
    category: PermissionCategory.ADMIN,
    action: 'manage',
    resource: 'admin.users',
    scope: PermissionScope.GLOBAL,
    isSystem: true,
    tags: ['admin', 'users', 'manage'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'admin.roles.view',
    name: 'View Roles',
    description: 'View role management interface',
    category: PermissionCategory.ADMIN,
    action: 'view',
    resource: 'admin.roles',
    scope: PermissionScope.GLOBAL,
    isSystem: true,
    tags: ['admin', 'roles', 'view'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'admin.roles.manage',
    name: 'Manage Roles',
    description: 'Create, edit, and delete roles',
    category: PermissionCategory.ADMIN,
    action: 'manage',
    resource: 'admin.roles',
    scope: PermissionScope.GLOBAL,
    isSystem: true,
    tags: ['admin', 'roles', 'manage'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'admin.permissions.view',
    name: 'View Permissions',
    description: 'View permission management interface',
    category: PermissionCategory.ADMIN,
    action: 'view',
    resource: 'admin.permissions',
    scope: PermissionScope.GLOBAL,
    isSystem: true,
    tags: ['admin', 'permissions', 'view'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'admin.permissions.manage',
    name: 'Manage Permissions',
    description: 'Assign and revoke permissions',
    category: PermissionCategory.ADMIN,
    action: 'manage',
    resource: 'admin.permissions',
    scope: PermissionScope.GLOBAL,
    isSystem: true,
    tags: ['admin', 'permissions', 'manage'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'admin.billing.view',
    name: 'View Billing',
    description: 'View billing and subscription information',
    category: PermissionCategory.ADMIN,
    action: 'view',
    resource: 'admin.billing',
    scope: PermissionScope.GLOBAL,
    isSystem: true,
    tags: ['admin', 'billing', 'view'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'admin.billing.manage',
    name: 'Manage Billing',
    description: 'Manage billing and subscriptions',
    category: PermissionCategory.ADMIN,
    action: 'manage',
    resource: 'admin.billing',
    scope: PermissionScope.GLOBAL,
    isSystem: true,
    tags: ['admin', 'billing', 'manage'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },

  // Analytics Permissions
  {
    id: 'analytics.basic',
    name: 'Basic Analytics',
    description: 'Access to basic analytics and reports',
    category: PermissionCategory.ANALYTICS,
    action: 'view',
    resource: 'analytics.basic',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['analytics', 'basic'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'analytics.advanced',
    name: 'Advanced Analytics',
    description: 'Access to advanced analytics and custom reports',
    category: PermissionCategory.ANALYTICS,
    action: 'view',
    resource: 'analytics.advanced',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['analytics', 'advanced'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'analytics.custom',
    name: 'Custom Analytics',
    description: 'Create and customize analytics reports',
    category: PermissionCategory.ANALYTICS,
    action: 'customize',
    resource: 'analytics',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['analytics', 'custom'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },

  // Integration Permissions
  {
    id: 'integration.webhook',
    name: 'Webhook Integration',
    description: 'Configure and manage webhooks',
    category: PermissionCategory.INTEGRATION,
    action: 'manage',
    resource: 'integration.webhook',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['integration', 'webhook'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'integration.api_key',
    name: 'API Key Management',
    description: 'Generate and manage API keys',
    category: PermissionCategory.INTEGRATION,
    action: 'manage',
    resource: 'integration.api_key',
    scope: PermissionScope.OWN,
    isSystem: true,
    tags: ['integration', 'api-key'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  },
  {
    id: 'integration.partner',
    name: 'Partner Integration',
    description: 'Access to partner integration features',
    category: PermissionCategory.INTEGRATION,
    action: 'access',
    resource: 'integration.partner',
    scope: PermissionScope.PARTNER,
    isSystem: true,
    tags: ['integration', 'partner'],
    deprecated: false,
    metadata: {
      createdAt: new Date(),
      updatedAt: new Date(),
      version: 1
    }
  }
];
