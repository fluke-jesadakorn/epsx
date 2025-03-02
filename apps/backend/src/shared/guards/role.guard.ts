import { Injectable, CanActivate, ExecutionContext, UnauthorizedException, ForbiddenException } from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { auth } from '../firebase-admin';
import { Request } from 'express';

export enum UserRole {
  PUBLIC = 'public',
  BASIC = 'basic',
  PREMIUM = 'premium',
}

export const ROLES_KEY = 'roles';
export const Roles = (...roles: UserRole[]) => {
  return (target: any, key?: string | symbol, descriptor?: any) => {
    if (descriptor) {
      Reflect.defineMetadata(ROLES_KEY, roles, descriptor.value);
      return descriptor;
    }
    Reflect.defineMetadata(ROLES_KEY, roles, target);
    return target;
  };
};

export const extractTokenFromCookie = (req: Request): string | null => {
  if (req.cookies && req.cookies.token) {
    return req.cookies.token;
  }
  return null;
};

@Injectable()
export class RolesGuard implements CanActivate {
  constructor(private reflector: Reflector) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const roles = this.reflector.get<UserRole[]>(ROLES_KEY, context.getHandler());
    
    if (!roles) {
      return true; // No roles specified means the endpoint is public
    }

    const request = context.switchToHttp().getRequest();
    const token = extractTokenFromCookie(request);

    if (!token) {
      if (roles.includes(UserRole.PUBLIC)) {
        request.userRole = UserRole.PUBLIC;
        return true;
      }
      throw new UnauthorizedException('No authentication token provided');
    }

    try {
      const decodedToken = await auth().verifyIdToken(token);
      const userSnapshot = await auth().getUser(decodedToken.uid);
      
      // Get user's custom claims which contain their role
      const customClaims = userSnapshot.customClaims || {};
      let userRole = UserRole.PUBLIC;

      if (customClaims.premium) {
        userRole = UserRole.PREMIUM;
      } else if (customClaims.basic) {
        userRole = UserRole.BASIC;
      }

      // Attach the role to the request for use in the controller
      request.userRole = userRole;

      // Check if user has sufficient role
      const hasRole = roles.includes(userRole);
      if (!hasRole) {
        throw new ForbiddenException('Insufficient role permissions');
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
