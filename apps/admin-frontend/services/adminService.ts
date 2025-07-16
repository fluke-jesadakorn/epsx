import { AdminRole, ADMIN_PERMISSIONS } from '@/types/admin/roles';

export class AdminService {
  static hasPermission(
    userRole: AdminRole,
    resource: string,
    action: string
  ): boolean {
    const permissions = ADMIN_PERMISSIONS[userRole];
    
    // Super admin has all permissions
    if (userRole === AdminRole.SUPER_ADMIN) {
      return true;
    }

    return permissions.some(permission => {
      const resourceMatch = permission.resource === '*' || permission.resource === resource;
      const actionMatch = permission.actions.includes('*') || permission.actions.includes(action);
      return resourceMatch && actionMatch;
    });
  }

  static getAvailableActions(userRole: AdminRole, resource: string): string[] {
    const permissions = ADMIN_PERMISSIONS[userRole];
    const resourcePermission = permissions.find(p => 
      p.resource === resource || p.resource === '*'
    );
    
    return resourcePermission?.actions || [];
  }

  static canManageUsers(userRole: AdminRole): boolean {
    return this.hasPermission(userRole, 'users', 'write');
  }

  static canViewPayments(userRole: AdminRole): boolean {
    return this.hasPermission(userRole, 'payments', 'read');
  }

  static canManageSystem(userRole: AdminRole): boolean {
    return this.hasPermission(userRole, 'system', 'write');
  }

  static canViewAnalytics(userRole: AdminRole): boolean {
    return this.hasPermission(userRole, 'analytics', 'read');
  }

  static getRolePriority(role: AdminRole): number {
    const priority = {
      [AdminRole.SUPPORT]: 0,
      [AdminRole.MODERATOR]: 1,
      [AdminRole.ADMIN]: 2,
      [AdminRole.SUPER_ADMIN]: 3
    };
    return priority[role] || 0;
  }
}
