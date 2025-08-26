// UNIFIED ADMIN PERMISSION SYSTEM - Matches backend Rust implementation
// Uses unified format: "domain:action:scope" for all admin operations

// Define shared types locally for admin frontend
export interface UnifiedPermission {
  domain: PermissionDomain;
  action: PermissionAction;
  scope: PermissionScope;
}

export enum PermissionDomain {
  Users = 'users',
  Roles = 'roles', 
  Permissions = 'permissions',
  Analytics = 'analytics',
  Packages = 'packages',
  Payments = 'payments',
  Dashboard = 'dashboard',
  AdminUsers = 'admin-users',
  AdminRoles = 'admin-roles',
  AdminSecurity = 'admin-security',
  AdminSystem = 'admin-system',
  AdminAudit = 'admin-audit',
  AdminAnalytics = 'admin-analytics',
  AdminFinance = 'admin-finance',
  AdminContent = 'admin-content',
  AdminSupport = 'admin-support',
  System = 'system',
  Security = 'security',
  Audit = 'audit',
}

export enum PermissionAction {
  Read = 'read',
  Write = 'write',
  Create = 'create',
  Update = 'update',
  Delete = 'delete',
  View = 'view',
  Admin = 'admin',
  Manage = 'manage',
  Execute = 'execute',
  Export = 'export',
  Import = 'import',
  Grant = 'grant',
  Revoke = 'revoke',
  Audit = 'audit',
  All = 'all',
}

export enum PermissionScope {
  Own = 'own',
  Team = 'team',
  Org = 'org',
  System = 'system',
  All = 'all',
}

// Admin-specific unified permissions using domain:action:scope format
export enum AdminModule {
  UserManagement = 'admin-users',
  AnalyticsAccess = 'admin-analytics', 
  SystemConfiguration = 'admin-system',
  AuditLogs = 'admin-audit',
  FinancialOversight = 'admin-finance',
  ContentManagement = 'admin-content',
  SupportAccess = 'admin-support',
  SecurityManagement = 'admin-security'
}

// Unified admin permission constants
export const AdminPermissions = {
  // User Management Module - admin-users:action:scope
  USER_READ_SYSTEM: 'admin-users:read:system',
  USER_CREATE_SYSTEM: 'admin-users:create:system', 
  USER_UPDATE_SYSTEM: 'admin-users:update:system',
  USER_DELETE_SYSTEM: 'admin-users:delete:system',
  USER_MANAGE_ALL: 'admin-users:manage:all',

  // Analytics Access Module - admin-analytics:action:scope
  ANALYTICS_READ_SYSTEM: 'admin-analytics:read:system',
  ANALYTICS_EXPORT_SYSTEM: 'admin-analytics:export:system',
  ANALYTICS_CREATE_SYSTEM: 'admin-analytics:create:system',
  ANALYTICS_MANAGE_ALL: 'admin-analytics:manage:all',

  // System Configuration Module - admin-system:action:scope
  SYSTEM_READ_SYSTEM: 'admin-system:read:system',
  SYSTEM_UPDATE_SYSTEM: 'admin-system:update:system',
  SYSTEM_MANAGE_SYSTEM: 'admin-system:manage:system',
  SYSTEM_ADMIN_ALL: 'admin-system:admin:all',

  // Audit Logs Module - admin-audit:action:scope
  AUDIT_READ_SYSTEM: 'admin-audit:read:system',
  AUDIT_EXPORT_SYSTEM: 'admin-audit:export:system',
  AUDIT_MANAGE_SYSTEM: 'admin-audit:manage:system',
  AUDIT_READ_ALL: 'admin-audit:read:all',

  // Security Management Module - admin-security:action:scope
  SECURITY_READ_SYSTEM: 'admin-security:read:system',
  SECURITY_MANAGE_ALL: 'admin-security:manage:all',
  SECURITY_ADMIN_ALL: 'admin-security:admin:all',

  // Financial Oversight Module - admin-finance:action:scope
  FINANCE_READ_SYSTEM: 'admin-finance:read:system',
  FINANCE_MANAGE_SYSTEM: 'admin-finance:manage:system',

  // Content Management Module - admin-content:action:scope
  CONTENT_READ_SYSTEM: 'admin-content:read:system',
  CONTENT_WRITE_SYSTEM: 'admin-content:write:system',
  CONTENT_DELETE_SYSTEM: 'admin-content:delete:system',

  // Support Access Module - admin-support:action:scope  
  SUPPORT_READ_SYSTEM: 'admin-support:read:system',
  SUPPORT_WRITE_SYSTEM: 'admin-support:write:system',
} as const;

// Unified admin user interface
export interface UnifiedAdminUser {
  id: string;
  email: string;
  permissions: string[]; // Array of unified permission strings
  modules: AdminModule[]; // Deprecated: use permissions instead
  assignedBy: string;
  assignedAt: Date;
  isActive: boolean;
}

// Module access levels (mapped to unified actions)
export enum AdminAccessLevel {
  Read = 'read',      // Can view module data
  Write = 'write',    // Can modify module data
  Create = 'create',  // Can create new items
  Update = 'update',  // Can update existing items
  Delete = 'delete',  // Can delete items
  Manage = 'manage',  // Can manage module settings
  Admin = 'admin',    // Full administrative control
}

// Permission utility functions
export class AdminPermissionUtils {
  /**
   * Check if user has specific permission
   */
  static hasPermission(userPermissions: string[], requiredPermission: string): boolean {
    // Check for exact match
    if (userPermissions.includes(requiredPermission)) {
      return true;
    }
    
    // Check for wildcard matches
    return userPermissions.some(permission => {
      if (permission === '*:*:*') return true; // Full admin
      
      const [userDomain, userAction, userScope] = permission.split(':');
      const [reqDomain, reqAction, reqScope] = requiredPermission.split(':');
      
      const domainMatch = userDomain === '*' || userDomain === reqDomain;
      const actionMatch = userAction === '*' || userAction === reqAction || 
                         this.actionIncludes(userAction, reqAction);
      const scopeMatch = userScope === '*' || userScope === reqScope ||
                        this.scopeIncludes(userScope, reqScope);
      
      return domainMatch && actionMatch && scopeMatch;
    });
  }
  
  /**
   * Check if user has access to admin module
   */
  static hasModuleAccess(userPermissions: string[], module: AdminModule, level: AdminAccessLevel = AdminAccessLevel.Read): boolean {
    const requiredPermission = `${module}:${level}:system`;
    return this.hasPermission(userPermissions, requiredPermission);
  }
  
  /**
   * Get all modules user has access to
   */
  static getUserModules(userPermissions: string[]): AdminModule[] {
    const modules: AdminModule[] = [];
    
    for (const module of Object.values(AdminModule)) {
      if (this.hasModuleAccess(userPermissions, module)) {
        modules.push(module);
      }
    }
    
    return modules;
  }
  
  /**
   * Get user's access level for a specific module
   */
  static getUserAccessLevel(userPermissions: string[], module: AdminModule): AdminAccessLevel | null {
    const levels = [AdminAccessLevel.Admin, AdminAccessLevel.Manage, AdminAccessLevel.Delete, 
                   AdminAccessLevel.Update, AdminAccessLevel.Create, AdminAccessLevel.Write, AdminAccessLevel.Read];
    
    for (const level of levels) {
      if (this.hasModuleAccess(userPermissions, module, level)) {
        return level;
      }
    }
    
    return null;
  }
  
  /**
   * Convert legacy admin role to unified permissions
   */
  static legacyRoleToPermissions(role: string): string[] {
    switch (role.toUpperCase()) {
      case 'ADMIN':
        return ['*:*:*']; // Full access
      case 'MODERATOR':
        return [
          AdminPermissions.USER_READ_SYSTEM,
          AdminPermissions.USER_UPDATE_SYSTEM,
          AdminPermissions.CONTENT_READ_SYSTEM,
          AdminPermissions.CONTENT_WRITE_SYSTEM,
          AdminPermissions.CONTENT_DELETE_SYSTEM,
          AdminPermissions.ANALYTICS_READ_SYSTEM
        ];
      case 'SUPPORT':
        return [
          AdminPermissions.USER_READ_SYSTEM,
          AdminPermissions.SUPPORT_READ_SYSTEM,
          AdminPermissions.SUPPORT_WRITE_SYSTEM,
          AdminPermissions.FINANCE_READ_SYSTEM
        ];
      default:
        return [];
    }
  }
  
  /**
   * Convert legacy permission actions to unified permissions
   */
  static legacyActionsToPermissions(resource: string, actions: string[]): string[] {
    const domain = this.resourceToAdminDomain(resource);
    
    return actions.map(action => {
      const unifiedAction = this.mapLegacyAction(action);
      const scope = action === 'manage' ? 'all' : 'system';
      return `${domain}:${unifiedAction}:${scope}`;
    });
  }
  
  private static actionIncludes(parentAction: string, childAction: string): boolean {
    const hierarchy = {
      'admin': ['manage', 'write', 'read', 'view', 'create', 'update', 'delete'],
      'manage': ['write', 'read', 'view', 'create', 'update'],
      'write': ['read', 'view'],
      'create': ['read', 'view'],
      'update': ['read', 'view']
    };
    
    return hierarchy[parentAction]?.includes(childAction) || false;
  }
  
  private static scopeIncludes(parentScope: string, childScope: string): boolean {
    const hierarchy = {
      'all': ['system', 'org', 'team', 'own'],
      'system': ['org', 'team', 'own'], 
      'org': ['team', 'own'],
      'team': ['own']
    };
    
    return hierarchy[parentScope]?.includes(childScope) || false;
  }
  
  private static resourceToAdminDomain(resource: string): string {
    const mapping: Record<string, string> = {
      'users': 'admin-users',
      'content': 'admin-content',
      'reports': 'admin-analytics',
      'tickets': 'admin-support',
      'payments': 'admin-finance',
      'system': 'admin-system',
      'security': 'admin-security',
      'audit': 'admin-audit'
    };
    
    return mapping[resource] || 'admin-users';
  }
  
  private static mapLegacyAction(action: string): string {
    const mapping: Record<string, string> = {
      'read': 'read',
      'write': 'write', 
      'delete': 'delete',
      'manage': 'manage',
      '*': 'admin'
    };
    
    return mapping[action] || 'read';
  }
}

// Permission validation for admin interfaces
export class AdminPermissionValidator {
  /**
   * Validate unified permission format
   */
  static isValidPermission(permission: string): boolean {
    const parts = permission.split(':');
    
    if (parts.length !== 3) return false;
    
    const [domain, action, scope] = parts;
    
    // Check domain
    const validDomains = [...Object.values(AdminModule), 'system', 'security', 'audit'];
    if (!validDomains.includes(domain) && domain !== '*') return false;
    
    // Check action
    const validActions = Object.values(AdminAccessLevel);
    if (!validActions.includes(action as AdminAccessLevel) && action !== '*') return false;
    
    // Check scope
    const validScopes = ['own', 'team', 'org', 'system', 'all'];
    if (!validScopes.includes(scope) && scope !== '*') return false;
    
    return true;
  }
  
  /**
   * Get permission display name
   */
  static getPermissionDisplayName(permission: string): string {
    const [domain, action, scope] = permission.split(':');
    
    const domainName = domain.replace('admin-', '').replace('-', ' ').toUpperCase();
    const actionName = action.charAt(0).toUpperCase() + action.slice(1);
    const scopeName = scope.charAt(0).toUpperCase() + scope.slice(1);
    
    return `${domainName} - ${actionName} (${scopeName})`;
  }
}

// Style checking for admin permissions
export class AdminStyleChecker {
  /**
   * Check admin permission format consistency
   */
  static checkPermissionStyle(permission: string): { valid: boolean; errors: string[] } {
    const errors: string[] = [];
    
    // Basic format check
    if (!/^[a-z-]+:[a-z]+:[a-z]+$/.test(permission)) {
      errors.push('Permission must follow format "domain:action:scope" with lowercase letters and hyphens');
    }
    
    // Admin domain check
    const [domain] = permission.split(':');
    if (!domain.startsWith('admin-') && !['system', 'security', 'audit'].includes(domain)) {
      errors.push('Admin permissions should use admin-prefixed domains');
    }
    
    // Validation
    if (!AdminPermissionValidator.isValidPermission(permission)) {
      errors.push('Invalid permission format or values');
    }
    
    return {
      valid: errors.length === 0,
      errors
    };
  }
}

// Legacy compatibility types - DEPRECATED, use unified types instead
export enum AdminRole {
  ADMIN = 'ADMIN',
  MODERATOR = 'MODERATOR', 
  SUPPORT = 'SUPPORT'
}

export interface AdminPermission {
  resource: string;
  actions: string[];
}

// DEPRECATED: Legacy permission mapping - use AdminPermissionUtils.legacyRoleToPermissions instead
export const ADMIN_PERMISSIONS: Record<AdminRole, AdminPermission[]> = {
  [AdminRole.ADMIN]: [
    { resource: '*', actions: ['*'] }
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