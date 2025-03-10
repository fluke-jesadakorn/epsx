import { Injectable, CanActivate, ExecutionContext, ForbiddenException } from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { AUTH_REQUIREMENTS_KEY, AuthRequirements } from '../decorators/auth-requirements.decorator';
import { TokenClaims } from '../types/token-claims';
import { UserRole } from '../types/roles.enum';
import { hasRequiredAccessLevel } from '../types/roles.enum';

@Injectable()
export class AuthorizationGuard implements CanActivate {
  constructor(private reflector: Reflector) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const requirements = this.reflector.getAllAndOverride<AuthRequirements>(
      AUTH_REQUIREMENTS_KEY,
      [context.getHandler(), context.getClass()]
    );

    if (!requirements?.roles && !requirements?.permissions) {
      return true;
    }

    const request = context.switchToHttp().getRequest();
    const user = request.user as TokenClaims;
    
    if (!user) {
      throw new ForbiddenException('User not authenticated');
    }

    // Administrator has access to everything
    if (user.role === UserRole.ADMINISTRATOR) {
      return true;
    }

    // Check role requirements
    if (requirements.roles) {
      const minRequiredRole = requirements.roles.reduce((min, role) => {
        const minLevel = hasRequiredAccessLevel(min, role) ? min : role;
        return minLevel;
      }, requirements.roles[0]);

      if (!hasRequiredAccessLevel(user.role, minRequiredRole)) {
        throw new ForbiddenException(
          `Insufficient role level. Required: ${minRequiredRole}, Current: ${user.role}`
        );
      }
    }

    // Check permission requirements - using permissions from token claims
    if (requirements.permissions) {
      const hasAllPermissions = requirements.permissions.every(permission =>
        user.permissions.includes(permission)
      );

      if (!hasAllPermissions) {
        const missingPermissions = requirements.permissions.filter(
          permission => !user.permissions.includes(permission)
        );
        throw new ForbiddenException(
          `Missing required permissions: ${missingPermissions.join(', ')}`
        );
      }
    }

    return true;
  }
}
