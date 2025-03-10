import { Injectable, UnauthorizedException } from '@nestjs/common';
import { Response } from 'express';
import { FirebaseAdminService } from '../../../shared/firebase-admin';
import { TokenService } from './token.service';
import { SessionService } from './session.service';
import { AuthLoggerService } from './auth-logger.service';
import { RoleService } from './role.service';

@Injectable()
export class AuthService {
  constructor(
    private readonly firebaseAdmin: FirebaseAdminService,
    private readonly tokenService: TokenService,
    private readonly sessionService: SessionService,
    private readonly authLogger: AuthLoggerService,
    private readonly roleService: RoleService
  ) {}

  generateOAuthURL(provider: string) {
    // Implementation of OAuth URL generation
    const authUrl = `https://oauth.provider.com/${provider}/auth`;
    return { authUrl };
  }

  async handleOAuthCallback(code: string, state: string) {
    // Implementation of OAuth callback handling
    // This would include token exchange and user info retrieval
    const token = 'exchange-token-implementation';
    const user = { uid: 'user-id-implementation' };
    
    return { token, user };
  }

  async verifySession(sessionToken: string | undefined) {
    if (!sessionToken) {
      return { valid: false };
    }

    try {
      if (!(await this.sessionService.isSessionActive(sessionToken))) {
        return { valid: false };
      }

      const decodedClaims = await this.firebaseAdmin.verifySessionCookie(sessionToken);
      const userSnapshot = await this.firebaseAdmin.auth.getUser(decodedClaims.uid);
      const customClaims = userSnapshot.customClaims || {};
      
      const userRole = await this.roleService.determineUserRole(customClaims);

      // Update session activity
      await this.sessionService.updateSessionActivity(sessionToken);

      return {
        valid: true,
        user: {
          email: decodedClaims.email,
          role: userRole,
          features: customClaims.features || [],
          permissions: customClaims.permissions || [],
          tokenBalance: customClaims.tokenBalance || 0,
        },
      };
    } catch (error) {
      return { valid: false };
    }
  }

  async handleOAuthCallbackAndCreateSession(
    code: string, 
    state: string, 
    ipAddress: string,
    userAgent: string,
    res: Response
  ) {
    try {
      const { token: idToken, user } = await this.handleOAuthCallback(code, state);
      
      // Create session token
      const sessionToken = await this.tokenService.createSessionToken(idToken);

      // Create session
      await this.sessionService.createSession(
        user.uid,
        sessionToken,
        ipAddress,
        userAgent
      );

      // Set secure session cookie
      const cookieOptions = {
        maxAge: 60 * 60 * 24 * 7 * 1000, // 7 days
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax' as const,
        path: '/',
        domain: process.env.NODE_ENV === 'production' ? process.env.COOKIE_DOMAIN : undefined
      };

      res.cookie('__session', sessionToken, cookieOptions);
      
      const frontendUrl = new URL('/', process.env.FRONTEND_URL);
      return frontendUrl.toString();
    } catch (error) {
      await this.authLogger.logAuthEvent({
        action: 'login',
        status: 'failure',
        errorDetails: error instanceof Error ? error.message : 'Unknown error',
        ipAddress,
        userAgent
      });
      throw new UnauthorizedException('Failed to process OAuth callback');
    }
  }
}
