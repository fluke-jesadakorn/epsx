// Unified permission format system for frontend - matches backend Rust implementation
// Format: "domain:action:scope" (e.g., "users:read:own", "admin:write:system")

export interface UnifiedPermission {
  domain: PermissionDomain;
  action: PermissionAction;
  scope: PermissionScope;
}

// Permission domains (resources/entities) - matching backend
export enum PermissionDomain {
  // User domains
  Users = 'users',
  Roles = 'roles', 
  Permissions = 'permissions',
  
  // Business domains
  Analytics = 'analytics',
  Packages = 'packages',
  Payments = 'payments',
  Dashboard = 'dashboard',
  
  // Admin domains
  AdminUsers = 'admin-users',
  AdminRoles = 'admin-roles',
  AdminSecurity = 'admin-security',
  AdminSystem = 'admin-system',
  AdminAudit = 'admin-audit',
  AdminAnalytics = 'admin-analytics',
  AdminFinance = 'admin-finance',
  AdminContent = 'admin-content',
  AdminSupport = 'admin-support',
  
  // System domains
  System = 'system',
  Security = 'security',
  Audit = 'audit',
}

// Permission actions (what can be done) - matching backend
export enum PermissionAction {
  // Basic CRUD
  Read = 'read',
  Write = 'write',
  Create = 'create',
  Update = 'update',
  Delete = 'delete',
  
  // Special actions
  View = 'view',
  Admin = 'admin',
  Manage = 'manage',
  Execute = 'execute',
  Export = 'export',
  Import = 'import',
  
  // Admin-specific
  Grant = 'grant',
  Revoke = 'revoke',
  Audit = 'audit',
  
  // Wildcard
  All = 'all',
}

// Permission scopes (where/to what extent) - matching backend
export enum PermissionScope {
  /** Own resources only (user's own data) */
  Own = 'own',
  /** Team/group resources */
  Team = 'team',
  /** Organization resources */
  Org = 'org',
  /** System-wide access */
  System = 'system',
  /** All resources (admin-level) */
  All = 'all',
}

// Standard permission patterns - matching backend constants
export const PermissionPatterns = {
  // Standard user permissions
  USER_READ_OWN: 'users:read:own',
  USER_WRITE_OWN: 'users:write:own',
  USER_READ_TEAM: 'users:read:team',
  
  // Admin module permissions  
  ADMIN_USER_READ_ALL: 'admin-users:read:all',
  ADMIN_USER_WRITE_ALL: 'admin-users:write:all',
  ADMIN_SYSTEM_MANAGE_ALL: 'admin-system:manage:all',
  
  // Analytics permissions
  ANALYTICS_VIEW_OWN: 'analytics:view:own',
  ANALYTICS_EXPORT_OWN: 'analytics:export:own',
  ANALYTICS_ADMIN_ALL: 'analytics:admin:all',
  
  // Dashboard permissions
  DASHBOARD_VIEW_OWN: 'dashboard:view:own',
  DASHBOARD_ADMIN_SYSTEM: 'dashboard:admin:system',
  
  // Package tier permissions
  PACKAGES_VIEW_OWN: 'packages:view:own',
  PACKAGES_MANAGE_ALL: 'packages:manage:all',
  
  // Security permissions
  SECURITY_VIEW_SYSTEM: 'security:view:system',
  SECURITY_MANAGE_SYSTEM: 'security:manage:system',
} as const;

// Permission format converter for backward compatibility
export class PermissionConverter {
  /**
   * Convert legacy dot notation to unified format
   * "users.view" -> "users:read:own"
   */
  static fromDotNotation(permission: string): UnifiedPermission {
    const parts = permission.split('.');
    if (parts.length !== 2) {
      throw new Error(`Invalid dot notation format: ${permission}`);
    }
    
    const domain = this.mapStringToDomain(parts[0]);
    const action = this.mapLegacyActionToAction(parts[1]);
    const scope = parts[1] === 'admin' ? PermissionScope.System : PermissionScope.Own;
    
    return { domain, action, scope };
  }
  
  /**
   * Convert legacy admin actions to unified format
   * ["read", "write"] on "users" -> ["admin-users:read:team", "admin-users:write:team"]
   */
  static fromAdminActions(resource: string, actions: string[]): UnifiedPermission[] {
    const domain = this.mapResourceToAdminDomain(resource);
    
    return actions.map(action => {
      const actionEnum = this.mapStringToAction(action);
      const scope = resource === '*' 
        ? PermissionScope.All 
        : action === 'manage' 
          ? PermissionScope.System 
          : PermissionScope.Team;
          
      return { domain, action: actionEnum, scope };
    });
  }
  
  /**
   * Convert admin module format to unified format
   * "user-management:view" -> "admin-users:read:system"
   */
  static fromAdminModule(permission: string): UnifiedPermission {
    const parts = permission.split(':');
    if (parts.length !== 2) {
      throw new Error(`Invalid admin module format: ${permission}`);
    }
    
    const domain = this.mapModuleStringToDomain(parts[0]);
    const action = parts[1] === 'view' ? PermissionAction.Read : this.mapStringToAction(parts[1]);
    const scope = PermissionScope.System; // Admin modules always have system scope
    
    return { domain, action, scope };
  }
  
  private static mapStringToDomain(domainStr: string): PermissionDomain {
    switch (domainStr.toLowerCase()) {
      case 'users': return PermissionDomain.Users;
      case 'roles': return PermissionDomain.Roles;
      case 'analytics': return PermissionDomain.Analytics;
      case 'packages': return PermissionDomain.Packages;
      case 'payments': return PermissionDomain.Payments;
      case 'dashboard': return PermissionDomain.Dashboard;
      default: return PermissionDomain.Users;
    }
  }
  
  private static mapResourceToAdminDomain(resource: string): PermissionDomain {
    switch (resource) {
      case 'users': return PermissionDomain.AdminUsers;
      case 'content': return PermissionDomain.AdminContent;
      case 'reports': return PermissionDomain.AdminAnalytics;
      case 'tickets': return PermissionDomain.AdminSupport;
      case 'payments': return PermissionDomain.AdminFinance;
      case '*': return PermissionDomain.System;
      default: return PermissionDomain.AdminUsers;
    }
  }
  
  private static mapModuleStringToDomain(moduleStr: string): PermissionDomain {
    switch (moduleStr) {
      case 'user-management': return PermissionDomain.AdminUsers;
      case 'analytics-access': return PermissionDomain.AdminAnalytics;
      case 'system-configuration': return PermissionDomain.AdminSystem;
      case 'audit-logs': return PermissionDomain.AdminAudit;
      case 'financial-oversight': return PermissionDomain.AdminFinance;
      case 'content-management': return PermissionDomain.AdminContent;
      case 'support-access': return PermissionDomain.AdminSupport;
      case 'security-management': return PermissionDomain.AdminSecurity;
      default: return PermissionDomain.AdminUsers;
    }
  }
  
  private static mapStringToAction(actionStr: string): PermissionAction {
    switch (actionStr.toLowerCase()) {
      case 'read': return PermissionAction.Read;
      case 'write': return PermissionAction.Write;
      case 'create': return PermissionAction.Create;
      case 'update': return PermissionAction.Update;
      case 'delete': return PermissionAction.Delete;
      case 'view': return PermissionAction.View;
      case 'admin': return PermissionAction.Admin;
      case 'manage': return PermissionAction.Manage;
      case '*': case 'all': return PermissionAction.All;
      default: return PermissionAction.Read;
    }
  }
  
  private static mapLegacyActionToAction(actionStr: string): PermissionAction {
    switch (actionStr) {
      case 'view': return PermissionAction.View;
      case 'create': return PermissionAction.Create;
      case 'update': return PermissionAction.Update;
      case 'delete': return PermissionAction.Delete;
      case 'admin': return PermissionAction.Admin;
      default: return PermissionAction.View;
    }
  }
}

// Unified Permission class for parsing and validation
export class UnifiedPermissionParser {
  /**
   * Parse permission string in unified format "domain:action:scope"
   */
  static parse(permission: string): UnifiedPermission {
    const parts = permission.split(':');
    if (parts.length !== 3) {
      throw new Error(`Invalid unified permission format: ${permission}. Expected "domain:action:scope"`);
    }
    
    const domain = this.parseDomain(parts[0]);
    const action = this.parseAction(parts[1]);
    const scope = this.parseScope(parts[2]);
    
    return { domain, action, scope };
  }
  
  /**
   * Convert UnifiedPermission back to string format
   */
  static stringify(permission: UnifiedPermission): string {
    return `${permission.domain}:${permission.action}:${permission.scope}`;
  }
  
  /**
   * Check if this permission matches another (with wildcard support)
   */
  static matches(permission: UnifiedPermission, target: UnifiedPermission): boolean {
    const domainMatch = permission.domain === target.domain;
    const actionMatch = permission.action === PermissionAction.All || 
                       target.action === PermissionAction.All || 
                       permission.action === target.action;
    const scopeMatch = permission.scope === PermissionScope.All || 
                      target.scope === PermissionScope.All || 
                      permission.scope === target.scope;
    
    return domainMatch && actionMatch && scopeMatch;
  }
  
  /**
   * Check if this permission includes/subsumes another
   */
  static includes(parent: UnifiedPermission, child: UnifiedPermission): boolean {
    // Domain must match exactly
    if (parent.domain !== child.domain) return false;
    
    // Check action hierarchy
    const actionIncludes = this.actionIncludes(parent.action, child.action);
    
    // Check scope hierarchy  
    const scopeIncludes = this.scopeIncludes(parent.scope, child.scope);
    
    return actionIncludes && scopeIncludes;
  }
  
  private static actionIncludes(parent: PermissionAction, child: PermissionAction): boolean {
    if (parent === PermissionAction.All) return true;
    if (parent === PermissionAction.Admin) {
      return [PermissionAction.Admin, PermissionAction.Manage, PermissionAction.Write, PermissionAction.Read, PermissionAction.View].includes(child);
    }
    if (parent === PermissionAction.Manage) {
      return [PermissionAction.Write, PermissionAction.Read, PermissionAction.View].includes(child);
    }
    if (parent === PermissionAction.Write) {
      return [PermissionAction.Read, PermissionAction.View].includes(child);
    }
    return parent === child;
  }
  
  private static scopeIncludes(parent: PermissionScope, child: PermissionScope): boolean {
    if (parent === PermissionScope.All) return true;
    if (parent === PermissionScope.System) {
      return [PermissionScope.System, PermissionScope.Org, PermissionScope.Team, PermissionScope.Own].includes(child);
    }
    if (parent === PermissionScope.Org) {
      return [PermissionScope.Team, PermissionScope.Own].includes(child);
    }
    if (parent === PermissionScope.Team) {
      return child === PermissionScope.Own;
    }
    return parent === child;
  }
  
  private static parseDomain(domainStr: string): PermissionDomain {
    const domain = Object.values(PermissionDomain).find(d => d === domainStr);
    if (!domain) {
      throw new Error(`Invalid permission domain: ${domainStr}`);
    }
    return domain;
  }
  
  private static parseAction(actionStr: string): PermissionAction {
    const action = Object.values(PermissionAction).find(a => a === actionStr);
    if (!action) {
      throw new Error(`Invalid permission action: ${actionStr}`);
    }
    return action;
  }
  
  private static parseScope(scopeStr: string): PermissionScope {
    const scope = Object.values(PermissionScope).find(s => s === scopeStr);
    if (!scope) {
      throw new Error(`Invalid permission scope: ${scopeStr}`);
    }
    return scope;
  }
}

// Validation utilities
export class PermissionValidator {
  /**
   * Validate permission format
   */
  static validate(permission: string): boolean {
    try {
      UnifiedPermissionParser.parse(permission);
      return true;
    } catch {
      return false;
    }
  }
  
  /**
   * Check if permission is administrative
   */
  static isAdminPermission(permission: string): boolean {
    try {
      const parsed = UnifiedPermissionParser.parse(permission);
      return parsed.domain.startsWith('admin-') || 
             parsed.domain === PermissionDomain.System ||
             parsed.domain === PermissionDomain.Security ||
             parsed.domain === PermissionDomain.Audit;
    } catch {
      return false;
    }
  }
  
  /**
   * Extract domain from permission
   */
  static extractDomain(permission: string): PermissionDomain | null {
    try {
      const parsed = UnifiedPermissionParser.parse(permission);
      return parsed.domain;
    } catch {
      return null;
    }
  }
}

// Style checking utilities for consistent permission patterns
export class PermissionStyleChecker {
  private static readonly REQUIRED_FORMAT_REGEX = /^[a-z-]+:[a-z]+:[a-z]+$/;
  
  /**
   * Check if permission follows unified format style
   */
  static checkFormat(permission: string): { valid: boolean; errors: string[] } {
    const errors: string[] = [];
    
    // Check basic format
    if (!this.REQUIRED_FORMAT_REGEX.test(permission)) {
      errors.push('Permission must follow format "domain:action:scope" with lowercase letters and hyphens only');
    }
    
    // Check parts count
    const parts = permission.split(':');
    if (parts.length !== 3) {
      errors.push('Permission must have exactly 3 parts: domain, action, and scope');
    }
    
    // Check each part is not empty
    if (parts.some(part => !part)) {
      errors.push('All parts (domain, action, scope) must be non-empty');
    }
    
    // Check valid enums
    try {
      UnifiedPermissionParser.parse(permission);
    } catch (error) {
      errors.push(`Invalid permission values: ${error}`);
    }
    
    return {
      valid: errors.length === 0,
      errors
    };
  }
  
  /**
   * Get style recommendations for better permission naming
   */
  static getRecommendations(permission: string): string[] {
    const recommendations: string[] = [];
    
    try {
      const parsed = UnifiedPermissionParser.parse(permission);
      
      // Recommend more specific scopes
      if (parsed.scope === PermissionScope.All) {
        recommendations.push('Consider using a more specific scope than "all" for better security');
      }
      
      // Recommend using "read" instead of "view" for consistency
      if (parsed.action === PermissionAction.View) {
        recommendations.push('Consider using "read" action instead of "view" for consistency');
      }
      
      // Recommend admin domains for administrative actions
      if (parsed.action === PermissionAction.Admin && !parsed.domain.startsWith('admin-')) {
        recommendations.push('Consider using admin-prefixed domain for administrative actions');
      }
      
    } catch {
      recommendations.push('Fix permission format errors first');
    }
    
    return recommendations;
  }
}