import { Injectable, Logger } from '@nestjs/common';
import { FirebaseAdminService } from '../../../shared/firebase-admin';

interface TokenMetadata {
  userId: string;
  issuedAt: number;
  expiresAt: number;
  lastRefreshed?: number;
}

interface RefreshTokenResult {
  token: string;
  expiresAt: number;
}

@Injectable()
export class TokenService {
  private readonly tokenMetadataCache: Map<string, TokenMetadata> = new Map();
  private readonly revokedTokens: Set<string> = new Set();
  
  constructor(private readonly firebaseAdmin: FirebaseAdminService) {}

  async createSessionToken(idToken: string, expiresIn: number = 60 * 60 * 24 * 5 * 1000): Promise<string> {
    const decodedToken = await this.firebaseAdmin.auth.verifyIdToken(idToken);
    const sessionCookie = await this.firebaseAdmin.auth.createSessionCookie(idToken, { expiresIn });

    this.tokenMetadataCache.set(sessionCookie, {
      userId: decodedToken.uid,
      issuedAt: Date.now(),
      expiresAt: Date.now() + expiresIn
    });

    return sessionCookie;
  }

  async refreshSessionToken(currentToken: string): Promise<RefreshTokenResult> {
    // Verify current token is still valid
    const metadata = this.tokenMetadataCache.get(currentToken);
    if (!metadata) {
      throw new Error('Token metadata not found');
    }

    if (this.revokedTokens.has(currentToken)) {
      throw new Error('Token has been revoked');
    }

    // Check if token is within refresh window (e.g., last 24 hours of validity)
    const refreshWindowStart = metadata.expiresAt - (24 * 60 * 60 * 1000);
    if (Date.now() < refreshWindowStart) {
      throw new Error('Token not eligible for refresh yet');
    }

    try {
      // Verify the token with Firebase
      const decodedClaims = await this.firebaseAdmin.auth.verifySessionCookie(currentToken);
      
      // Create new session token
      const newExpiresIn = 60 * 60 * 24 * 5 * 1000; // 5 days
      const newToken = await this.firebaseAdmin.auth.createSessionCookie(
        (await this.firebaseAdmin.auth.createCustomToken(decodedClaims.uid)),
        { expiresIn: newExpiresIn }
      );

      // Update metadata for new token
      this.tokenMetadataCache.set(newToken, {
        userId: decodedClaims.uid,
        issuedAt: Date.now(),
        expiresAt: Date.now() + newExpiresIn,
        lastRefreshed: Date.now()
      });

      // Revoke old token
      this.revokeToken(currentToken);

      return {
        token: newToken,
        expiresAt: Date.now() + newExpiresIn
      };
    } catch (error: unknown) {
      Logger.error('Error refreshing token:', error);
      throw new Error(`Token refresh failed: ${error}`);
    }
  }

  async revokeToken(token: string): Promise<void> {
    const metadata = this.tokenMetadataCache.get(token);
    if (metadata) {
      this.revokedTokens.add(token);
      this.tokenMetadataCache.delete(token);
      
      // Revoke with Firebase
      try {
        await this.firebaseAdmin.auth.revokeRefreshTokens(metadata.userId);
      } catch (error: unknown) {
        const errorMessage = error instanceof Error ? error.message : 'Unknown error';
        Logger.error('Error revoking Firebase tokens:', errorMessage);
      }
    }
  }

  async revokeAllUserTokens(userId: string): Promise<void> {
    // Find all tokens for this user
    for (const [token, metadata] of this.tokenMetadataCache.entries()) {
      if (metadata.userId === userId) {
        await this.revokeToken(token);
      }
    }

    // Revoke all refresh tokens for the user in Firebase
    try {
      await this.firebaseAdmin.auth.revokeRefreshTokens(userId);
    } catch (error: unknown) {
      Logger.error('Error revoking Firebase refresh tokens:', error);
      throw new Error(`Failed to revoke refresh tokens: ${error}`);
    }
  }

  async isTokenRevoked(token: string): Promise<boolean> {
    return this.revokedTokens.has(token);
  }

  async getTokenMetadata(token: string): Promise<TokenMetadata | null> {
    return this.tokenMetadataCache.get(token) || null;
  }
}
