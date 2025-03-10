import { Injectable, CanActivate, ExecutionContext, UnauthorizedException, ForbiddenException, SetMetadata } from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { AuthService } from '../auth';
import { PERMISSIONS, hasPermission, PermissionKey } from '../permissions';
import { Request } from 'express';
import { UserRole } from '../types/roles.enum';

export const ROLES_KEY = 'roles';
export const PERMISSIONS_KEY = 'permissions';

export const extractTokenFromCookie = (req: Request): string | null => {
  if (req.cookies && req.cookies.__session) {
    return req.cookies.__session;
  }
  return null;
};

export const RequirePermissions = (...permissions: PermissionKey[]) => SetMetadata(PERMISSIONS_KEY, permissions);

@Injectable()
export class FirebaseAuthGuard implements CanActivate {
  constructor(
    private reflector: Reflector,
    private authService: AuthService
  ) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const requiredRoles = this.reflector.get<UserRole[]>(ROLES_KEY, context.getHandler());
    const requiredPermissions = this.reflector.get<PermissionKey[]>(PERMISSIONS_KEY, context.getHandler());
    
    if (!requiredRoles && !requiredPermissions) {
      return true; // No roles or permissions specified means the endpoint is public
    }

    const request = context.switchToHttp().getRequest();
    const token = extractTokenFromCookie(request);

    if (!token) {
      if (requiredRoles?.includes(UserRole.GUEST)) {
        request.userRole = UserRole.GUEST;
        return true;
      }
      throw new UnauthorizedException('No authentication token provided');
    }

    try {
      const decodedToken = await this.authService.verifyToken(token);
      const userSnapshot = await this.authService.getUser(decodedToken.uid);
      
      // Get user's custom claims which contain their role
      const customClaims = userSnapshot.customClaims || {};
      let userRole = UserRole.GUEST;

      // Check roles in descending order of privilege
      if (customClaims.admin) {
        userRole = UserRole.ADMINISTRATOR;
      } else if (customClaims.token_holder) {
        userRole = UserRole.TOKEN_HOLDER;
      } else if (customClaims.premium) {
        userRole = UserRole.PREMIUM_USER;
      } else if (customClaims.basic) {
        userRole = UserRole.REGISTERED_USER;
      }

      // Attach the role to the request for use in the controller
      request.userRole = userRole;

      // Check if user has sufficient role
      // Check role-based access
      if (requiredRoles && !requiredRoles.includes(userRole)) {
        throw new ForbiddenException('Insufficient role permissions');
      }

      // Check permission-based access
      if (requiredPermissions) {
        const hasAllPermissions = requiredPermissions.every(permissionKey => 
          hasPermission(userRole, PERMISSIONS[permissionKey])
        );
        if (!hasAllPermissions) {
          throw new ForbiddenException('Insufficient permissions');
        }
      }

      return true;
    } catch (error) {
      if (error instanceof ForbiddenException) {
        throw error;
      }
      throw new UnauthorizedException('Invalid authentication token');
    }
  }
}
