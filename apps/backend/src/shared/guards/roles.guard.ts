import { Injectable, CanActivate, ExecutionContext } from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { FirebaseAdminService, PERMISSIONS_KEY } from '..';

@Injectable()
export class RolesGuard implements CanActivate {
  constructor(
    private reflector: Reflector,
    private firebaseAdmin: FirebaseAdminService
  ) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const requiredPermissions = this.reflector.get<string[]>(
      PERMISSIONS_KEY,
      context.getHandler()
    );

    if (!requiredPermissions) {
      return true;
    }

    const request = context.switchToHttp().getRequest();
    const authHeader = request.headers.authorization;
    
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return false;
    }

    const token = authHeader.split(' ')[1];
    
    try {
      const decodedToken = await this.firebaseAdmin.verifyIdToken(token);
      const userRoles = decodedToken.roles || [];
      const userPermissions = decodedToken.permissions || [];

      // Check if user has all required permissions
      return requiredPermissions.every(permission => 
        userPermissions.includes(permission)
      );
    } catch (error) {
      return false;
    }
  }
}
