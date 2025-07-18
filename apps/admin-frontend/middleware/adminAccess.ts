import { NextRequest, NextResponse } from 'next/server';
import { AdminService } from '@/services/adminService';
import { AdminRole } from '@/types/admin/roles';

export function withAdminAccess(resource: string, action: string) {
  return async (_req: NextRequest) => {
    try {
      // Get admin user's role from session
      // This would be replaced with actual admin auth implementation
      const adminRole = AdminRole.ADMIN; // This should come from auth

      if (!AdminService.hasPermission(adminRole, resource, action)) {
        return NextResponse.json(
          { 
            error: 'Insufficient admin permissions',
            required: { resource, action },
            userRole: adminRole
          },
          { status: 403 }
        );
      }

      return NextResponse.next();
    } catch (error) {
      return NextResponse.json(
        { error: 'Admin permission verification failed' },
        { status: 500 }
      );
    }
  };
}

export function requireSuperAdmin() {
  return async (_req: NextRequest) => {
    try {
      // Get admin user's role from session
      const adminRole = AdminRole.ADMIN; // This would be replaced with actual session check
      
      if (adminRole !== AdminRole.ADMIN) {
        return NextResponse.json(
          { error: 'Super admin access required' },
          { status: 403 }
        );
      }
      
      return null; // Allow access
    } catch (error) {
      console.error('Admin access check failed:', error);
      return NextResponse.json(
        { error: 'Unauthorized' },
        { status: 401 }
      );
    }
  };
}

export function requireMinimumRole(minimumRole: AdminRole) {
  return async (_req: NextRequest) => {
    try {
      // Get admin user's role from session
      const adminRole = AdminRole.ADMIN; // This would be replaced with actual session check
      
      const rolePriority = AdminService.getRolePriority(adminRole);
      const requiredPriority = AdminService.getRolePriority(minimumRole);
      
      if (rolePriority < requiredPriority) {
        return NextResponse.json(
          { error: 'Insufficient permissions' },
          { status: 403 }
        );
      }
      
      return null; // Allow access
    } catch (error) {
      console.error('Admin access check failed:', error);
      return NextResponse.json(
        { error: 'Unauthorized' },
        { status: 401 }
      );
    }
  };
}
