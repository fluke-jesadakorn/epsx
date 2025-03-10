import { Injectable, CanActivate, ExecutionContext, ForbiddenException } from '@nestjs/common';
import { AuthService } from '../../../shared';
import { AuthenticatedRequest } from '../types/request';
import { UserRole } from '@epsx/shared';

@Injectable()
export class AdminGuard implements CanActivate {
  constructor(private readonly authService: AuthService) {}
  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest<AuthenticatedRequest>();
    const token = request.cookies?.token;

    if (!token) {
      throw new ForbiddenException('No authentication token provided');
    }

    try {
      const decodedToken = await this.authService.verifyToken(token);
      const userSnapshot = await this.authService.getUser(decodedToken.uid);
      const customClaims = userSnapshot.customClaims || {};

      // Only users with administrator role can access protected routes
      if (!customClaims.admin) {
        throw new ForbiddenException('Requires administrator privileges');
      }

      // Attach user info to request
      request.user = {
        uid: userSnapshot.uid,
        email: userSnapshot.email,
        role: customClaims.admin ? UserRole.ADMINISTRATOR : UserRole.PREMIUM_USER
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
