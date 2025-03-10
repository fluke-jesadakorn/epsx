import { Injectable, CanActivate, ExecutionContext, UnauthorizedException } from '@nestjs/common';
import { FirebaseAdminService } from '../firebase-admin';
import { AuthenticatedRequest } from '../interfaces/authenticated-request.interface';

@Injectable()
export class FirebaseAuthGuard implements CanActivate {
  constructor(private readonly firebaseAdmin: FirebaseAdminService) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest<AuthenticatedRequest>();
    const token = this.extractTokenFromCookie(request);

    if (!token) {
      throw new UnauthorizedException('No authentication token provided');
    }

    try {
      const decodedToken = await this.firebaseAdmin.verifyIdToken(token);
      const userRecord = await this.firebaseAdmin.getUser(decodedToken.uid);
      
      // Attach the user to the request
      request.user = {
        uid: userRecord.uid,
        email: userRecord.email,
        customClaims: userRecord.customClaims
      };

      return true;
    } catch (error) {
      throw new UnauthorizedException('Invalid authentication token');
    }
  }

  private extractTokenFromCookie(req: AuthenticatedRequest): string | null {
    if (req.cookies && req.cookies.__session) {
      return req.cookies.__session;
    }
    return null;
  }
}
