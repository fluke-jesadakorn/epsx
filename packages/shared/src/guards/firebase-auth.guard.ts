import { Injectable, CanActivate, ExecutionContext, UnauthorizedException } from '@nestjs/common';
import { FirebaseAdmin } from '../services/firebase-admin';
import { AuthenticatedRequest } from '../interfaces/request';

@Injectable()
export class FirebaseAuthGuard implements CanActivate {
  constructor(private readonly firebaseAdmin: FirebaseAdmin) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest<AuthenticatedRequest>();
    const token = this.extractTokenFromCookie(request);

    if (!token) {
      throw new UnauthorizedException('No authentication token provided');
    }

    try {
      const decodedToken = await this.firebaseAdmin.verifyIdToken(token);
      const userRecord = await this.firebaseAdmin.getUser(decodedToken.uid);
      
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
