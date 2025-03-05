import { Injectable, CanActivate, ExecutionContext, ForbiddenException } from '@nestjs/common';
import { auth } from '../../../shared';
import { AuthenticatedRequest } from '../types/request';
import { UserRole } from '../../../shared/guards/role.guard';

@Injectable()
export class AdminGuard implements CanActivate {
  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest<AuthenticatedRequest>();
    const token = request.cookies?.token;

    if (!token) {
      throw new ForbiddenException('No authentication token provided');
    }

    try {
      const decodedToken = await auth.verifyToken(token);
      const userSnapshot = await auth.getUser(decodedToken.uid);
      const customClaims = userSnapshot.customClaims || {};

      // Only users with admin role can access protected routes
      if (!customClaims.admin) {
        throw new ForbiddenException('Requires admin privileges');
      }

      // Attach user info to request
      request.user = {
        uid: userSnapshot.uid,
        email: userSnapshot.email,
        role: customClaims.admin ? UserRole.PREMIUM : UserRole.BASIC
      };

      return true;
    } catch (error) {
      if (error instanceof ForbiddenException) {
        throw error;
      }
      throw new ForbiddenException('Invalid authentication token');
    }
  }
}
