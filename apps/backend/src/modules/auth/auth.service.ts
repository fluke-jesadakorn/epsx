import {
  Injectable,
  NotFoundException,
  BadRequestException,
  UnauthorizedException,
} from "@nestjs/common";
import * as admin from "firebase-admin";
import { FirebaseAdminService } from "../../shared/firebase-admin";
import { UserRole } from "../../shared/guards/role.guard";
import * as crypto from "crypto";
import { OAuth2Client } from "google-auth-library";

const googleClient = new OAuth2Client(process.env.GOOGLE_CLIENT_ID);

@Injectable()
export class AuthService {
  private readonly stateMap = new Map<string, { timestamp: number }>();
  private readonly STATE_EXPIRY = 10 * 60 * 1000; // 10 minutes

  constructor(private readonly firebaseAdmin: FirebaseAdminService) {
    // Cleanup expired states periodically
    setInterval(() => this.cleanupExpiredStates(), 5 * 60 * 1000);
  }

  private cleanupExpiredStates() {
    const now = Date.now();
    for (const [state, data] of this.stateMap.entries()) {
      if (now - data.timestamp > this.STATE_EXPIRY) {
        this.stateMap.delete(state);
      }
    }
  }

  generateOAuthURL(provider: string): { authUrl: string; state: string } {
    if (provider !== "google") {
      throw new BadRequestException("Unsupported provider");
    }

    const state = crypto.randomBytes(32).toString("hex");
    this.stateMap.set(state, { timestamp: Date.now() });

    // Construct and validate the redirect URI
    const redirectUri = new URL('/api/v1/auth/oauth/callback', process.env.BACKEND_URL).toString();
    console.log('OAuth Configuration:', {
      clientId: process.env.GOOGLE_CLIENT_ID?.substring(0, 10) + '...',
      redirectUri,
      state: state.substring(0, 10) + '...',
    });

    // Only request minimum required scopes with clear justification:
    // - email: Required for user identification and account management
    // - profile.name: Required for displaying user's name in the UI
    const params = new URLSearchParams({
      client_id: process.env.GOOGLE_CLIENT_ID!,
      redirect_uri: redirectUri,
      response_type: "code",
      // Only request minimum required scopes with clear justification:
      // - openid: Required for OpenID Connect authentication
      // - userinfo.email: Required for user identification and account management
      scope: "openid https://www.googleapis.com/auth/userinfo.email",
      access_type: "offline", // Request refresh token for long-term access
      prompt: "consent", // Always show consent screen to ensure user awareness
      state,
    });

    return {
      authUrl: `https://accounts.google.com/o/oauth2/v2/auth?${params.toString()}`,
      state,
    };
  }

  async signInWithGoogle(
    idToken: string
  ): Promise<{ token: string; user: admin.auth.UserRecord }> {
    try {
      // Verify the Google ID token
      const ticket = await googleClient.verifyIdToken({
        idToken,
        audience: process.env.GOOGLE_CLIENT_ID,
      });
      const payload = ticket.getPayload();

      if (!payload) {
        throw new BadRequestException("Invalid Google token");
      }

      // Check if user exists in Firebase
      let user: admin.auth.UserRecord;
      try {
        user = await this.firebaseAdmin.getUserByEmail(payload.email!);
      } catch (error) {
        // Create new user if not exists
        user = await this.firebaseAdmin.auth.createUser({
          email: payload.email,
          emailVerified: payload.email_verified,
          displayName: payload.name,
          photoURL: payload.picture,
        });

        // Set default role claims
        await this.assignUserRole(user.uid, UserRole.PUBLIC);
      }

      // Create custom token for Firebase authentication
      const customToken = await this.firebaseAdmin.auth.createCustomToken(
        user.uid
      );

      return {
        token: customToken,
        user,
      };
    } catch (error: any) {
      if (error?.code === "auth/id-token-expired") {
        throw new BadRequestException("Token expired");
      }
      console.error("Google sign in error:", error);
      throw new BadRequestException("Failed to authenticate with Google");
    }
  }

  async handleOAuthCallback(
    code: string,
    state: string
  ): Promise<{ token: string; user: admin.auth.UserRecord }> {
    const stateData = this.stateMap.get(state);
    if (!stateData) {
      throw new UnauthorizedException("Invalid state parameter");
    }

    if (Date.now() - stateData.timestamp > this.STATE_EXPIRY) {
      this.stateMap.delete(state);
      throw new UnauthorizedException("State parameter expired");
    }

    this.stateMap.delete(state);

    // Exchange code for tokens
    const tokenResponse = await fetch("https://oauth2.googleapis.com/token", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({
        code,
        client_id: process.env.GOOGLE_CLIENT_ID!,
        client_secret: process.env.GOOGLE_CLIENT_SECRET!,
        redirect_uri: new URL('/api/v1/auth/oauth/callback', process.env.BACKEND_URL).toString(),
        grant_type: "authorization_code",
      }),
    });

    if (!tokenResponse.ok) {
      const errorData = await tokenResponse.json();
      console.error("Token exchange error:", errorData);
      throw new BadRequestException(
        `Failed to exchange authorization code: ${JSON.stringify(errorData)}`
      );
    }

    const tokens = await tokenResponse.json();

    // Get user info from Google
    const userInfoResponse = await fetch(
      "https://www.googleapis.com/oauth2/v3/userinfo",
      {
        headers: { Authorization: `Bearer ${tokens.access_token}` },
      }
    );

    if (!userInfoResponse.ok) {
      throw new BadRequestException("Failed to get user info");
    }

    const userInfo = await userInfoResponse.json();

    try {
      // Check if user exists
      let userRecord: admin.auth.UserRecord;
      try {
        userRecord = await this.firebaseAdmin.getUserByEmail(userInfo.email);
        console.log('Found existing user:', userRecord.uid);
      } catch (error) {
        console.log('Creating new user for email:', userInfo.email);
        // Create new user if not exists
        userRecord = await this.firebaseAdmin.auth.createUser({
          email: userInfo.email,
          displayName: userInfo.name,
          photoURL: userInfo.picture,
          emailVerified: true,
        });
        console.log('Created new user:', userRecord.uid);

        // Set default role claims
        await this.assignUserRole(userRecord.uid, UserRole.PUBLIC);
        console.log('Assigned default role to user');
      }

      // Create custom token
      console.log('Creating custom token for user:', userRecord.uid);
      const customToken = await this.firebaseAdmin.auth.createCustomToken(
        userRecord.uid
      );
      console.log('Successfully created custom token');
      // Return both token and user record
      return {
        token: customToken,
        user: userRecord
      };
    } catch (error) {
      console.error('Error processing authentication:', error);
      throw new BadRequestException(`Failed to process authentication: ${error}`);
    }
  }

  async verifyToken(token: string): Promise<admin.auth.DecodedIdToken> {
    try {
      const decodedToken = await this.firebaseAdmin.verifyIdToken(token);

      // Check if the token has been revoked
      const tokenInfo = await this.firebaseAdmin.auth.getUser(decodedToken.uid);
      if (tokenInfo.tokensValidAfterTime) {
        const validSince = new Date(tokenInfo.tokensValidAfterTime).getTime() / 1000;
        if (decodedToken.iat < validSince) {
          throw new BadRequestException("Token has been revoked");
        }
      }

      return decodedToken;
    } catch (error: any) {
      if (error?.code === "auth/id-token-expired") {
        throw new BadRequestException("Token expired");
      }
      if (error?.code === "auth/invalid-id-token") {
        throw new BadRequestException("Invalid token");
      }
      if (error?.code === "auth/user-disabled") {
        throw new BadRequestException("User account has been disabled");
      }
      throw new BadRequestException("Failed to verify token");
    }
  }

  async revokeUserTokens(userId: string): Promise<void> {
    try {
      // This will invalidate all existing tokens for the user
      await this.firebaseAdmin.auth.revokeRefreshTokens(userId);
      console.log('Successfully revoked tokens for user:', userId);
    } catch (error: any) {
      if (error?.code === "auth/user-not-found") {
        throw new NotFoundException("User not found");
      }
      throw new BadRequestException("Failed to revoke user tokens");
    }
  }
  async assignUserRole(userId: string, role: UserRole): Promise<void> {
    try {
      const user = await this.firebaseAdmin.getUser(userId);

      let claims = {
        admin: false,
        premium: false,
        basic: false,
      };

      // Set claims based on role
      switch (role) {
        case UserRole.PREMIUM:
          claims.premium = true;
          break;
        case UserRole.BASIC:
          claims.basic = true;
          break;
        // Public role has no special claims
        default:
          break;
      }

      await this.firebaseAdmin.setCustomUserClaims(userId, claims);
    } catch (error: any) {
      if (error?.code === "auth/user-not-found") {
        throw new NotFoundException("User not found");
      }
      throw new BadRequestException("Failed to assign role");
    }
  }

  async getUserRole(
    userId: string
  ): Promise<{ userId: string; email: string | undefined; role: UserRole }> {
    try {
      const user = await this.firebaseAdmin.getUser(userId);
      const claims = user.customClaims || {};

      let role = UserRole.PUBLIC;
      if (claims.premium) {
        role = UserRole.PREMIUM;
      } else if (claims.basic) {
        role = UserRole.BASIC;
      }

      return {
        userId: user.uid,
        email: user.email,
        role,
      };
    } catch (error: any) {
      if (error?.code === "auth/user-not-found") {
        throw new NotFoundException("User not found");
      }
      throw new BadRequestException("Failed to get user role");
    }
  }

  async listUsers(
    maxResults: number = 1000
  ): Promise<
    Array<{ userId: string; email: string | undefined; role: UserRole }>
  > {
    try {
      const listUsersResult =
        await this.firebaseAdmin.auth.listUsers(maxResults);

      return listUsersResult.users.map((user: admin.auth.UserRecord) => {
        const claims = user.customClaims || {};
        let role = UserRole.PUBLIC;

        if (claims.premium) {
          role = UserRole.PREMIUM;
        } else if (claims.basic) {
          role = UserRole.BASIC;
        }

        return {
          userId: user.uid,
          email: user.email,
          role,
        };
      });
    } catch (error) {
      throw new BadRequestException("Failed to list users");
    }
  }
}
