import { NextRequest, NextResponse } from 'next/server';
import { AdminService } from '@/services/adminService';
import { AdminRole } from '@/types/admin/roles';

export function withAdminAccess(resource: string, action: string) {
  return async (req: NextRequest) => {
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

export function withSuperAdminAccess() {
  return async (req: NextRequest) => {
    try {
      // Get admin user's role from session
      const adminRole = AdminRole.ADMIN; // This should come from auth

      if (adminRole !== AdminRole.SUPER_ADMIN) {
        return NextResponse.json(
          { 
            error: 'Super admin access required',
            userRole: adminRole
          },
          { status: 403 }
        );
      }

      return NextResponse.next();
    } catch (error) {
      return NextResponse.json(
        { error: 'Super admin verification failed' },
        { status: 500 }
      );
    }
  };
}

export function withMinimumAdminRole(minimumRole: AdminRole) {
  return async (req: NextRequest) => {
    try {
      // Get admin user's role from session
      const adminRole = AdminRole.ADMIN; // This should come from auth

      const rolePriority = AdminService.getRolePriority(adminRole);
      const requiredPriority = AdminService.getRolePriority(minimumRole);

      if (rolePriority < requiredPriority) {
        return NextResponse.json(
          { 
            error: 'Insufficient admin role',
            required: minimumRole,
            current: adminRole
          },
          { status: 403 }
        );
      }

      return NextResponse.next();
    } catch (error) {
      return NextResponse.json(
        { error: 'Admin role verification failed' },
        { status: 500 }
      );
    }
  };
}
