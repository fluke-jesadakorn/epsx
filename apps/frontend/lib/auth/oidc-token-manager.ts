// OIDC Token Manager (Server-Side)
// Secure JWT token management with validation, refresh, and storage

import { jwtVerify, importJWK, JWTPayload } from 'jose';
import { getOIDCDiscoveryClient, type OIDCConfiguration } from './oidc-discovery-client';

export interface TokenSet {
  access_token: string;
  id_token: string;
  refresh_token?: string;
  expires_at: number;
  token_type: string;
  scope?: string;
}

export interface DecodedToken extends JWTPayload {
  sub: string;
  email?: string;
  name?: string;
  role?: string;
  permissions?: string[];
  tenant_id?: string;
  iat: number;
  exp: number;
  aud: string | string[];
  iss: string;
}

export interface TokenValidationResult {
  valid: boolean;
  expired: boolean;
  payload?: DecodedToken;
  error?: string;
}

export interface TokenRefreshResult {
  success: boolean;
  tokens?: TokenSet;
  error?: string;
}

/**
 * OIDC Token Manager for server-side token operations
 */
export class OIDCTokenManager {
  private discoveryClient = getOIDCDiscoveryClient();
  private tokenCache = new Map<string, { config: OIDCConfiguration; jwks: any; expires: number }>();

  /**
   * Validate JWT token
   */
  async validateToken(token: string, tenantId?: string): Promise<TokenValidationResult> {
    try {
      if (!token) {
        return { valid: false, expired: false, error: 'Token is required' };
      }

      // Get OIDC configuration and JWKS
      const { config, jwks } = await this.getConfigAndJWKS(tenantId);
      
      // Find appropriate key for verification
      const key = this.findSigningKey(token, jwks);
      if (!key) {
        return { valid: false, expired: false, error: 'No matching signing key found' };
      }

      const publicKey = await importJWK(key);
      
      // Verify token
      const { payload } = await jwtVerify(token, publicKey, {
        issuer: config.issuer,
        audience: process.env.OIDC_CLIENT_ID || 'epsx-frontend',
        clockTolerance: 30, // Allow 30 second clock skew
      });

      const decodedToken = payload as DecodedToken;
      
      return {
        valid: true,
        expired: false,
        payload: decodedToken
      };

    } catch (error) {
      if (error instanceof Error) {
        // Check if token is expired
        if (error.message.includes('exp') || error.message.includes('expired')) {
          return { valid: false, expired: true, error: 'Token expired' };
        }
        
        return { valid: false, expired: false, error: error.message };
      }
      
      return { valid: false, expired: false, error: 'Token validation failed' };
    }
  }

  /**
   * Refresh access token using refresh token
   */
  async refreshTokens(refreshToken: string, tenantId?: string): Promise<TokenRefreshResult> {
    try {
      if (!refreshToken) {
        return { success: false, error: 'Refresh token is required' };
      }

      const config = await this.discoveryClient.discoverConfiguration(tenantId);
      
      const response = await fetch(config.token_endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
          'Accept': 'application/json',
        },
        body: new URLSearchParams({
          grant_type: 'refresh_token',
          refresh_token: refreshToken,
          client_id: process.env.OIDC_CLIENT_ID || 'epsx-frontend',
        }),
      });

      if (!response.ok) {
        const errorData = await response.text();
        return {
          success: false,
          error: `Token refresh failed: ${response.status} ${errorData}`
        };
      }

      const tokenData = await response.json();
      
      const tokens: TokenSet = {
        access_token: tokenData.access_token,
        id_token: tokenData.id_token,
        refresh_token: tokenData.refresh_token || refreshToken, // Keep old refresh token if not renewed
        expires_at: Date.now() + (tokenData.expires_in * 1000),
        token_type: tokenData.token_type || 'Bearer',
        scope: tokenData.scope,
      };

      return { success: true, tokens };

    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Token refresh failed'
      };
    }
  }

  /**
   * Exchange authorization code for tokens
   */
  async exchangeCodeForTokens(
    code: string,
    codeVerifier: string,
    redirectUri: string,
    tenantId?: string
  ): Promise<TokenRefreshResult> {
    try {
      const config = await this.discoveryClient.discoverConfiguration(tenantId);
      
      const response = await fetch(config.token_endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
          'Accept': 'application/json',
        },
        body: new URLSearchParams({
          grant_type: 'authorization_code',
          code,
          redirect_uri: redirectUri,
          client_id: process.env.OIDC_CLIENT_ID || 'epsx-frontend',
          code_verifier: codeVerifier,
        }),
      });

      if (!response.ok) {
        const errorData = await response.text();
        return {
          success: false,
          error: `Code exchange failed: ${response.status} ${errorData}`
        };
      }

      const tokenData = await response.json();
      
      const tokens: TokenSet = {
        access_token: tokenData.access_token,
        id_token: tokenData.id_token,
        refresh_token: tokenData.refresh_token,
        expires_at: Date.now() + (tokenData.expires_in * 1000),
        token_type: tokenData.token_type || 'Bearer',
        scope: tokenData.scope,
      };

      // Validate the received tokens
      const validation = await this.validateToken(tokens.id_token, tenantId);
      if (!validation.valid) {
        return {
          success: false,
          error: `Invalid ID token: ${validation.error}`
        };
      }

      return { success: true, tokens };

    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Code exchange failed'
      };
    }
  }

  /**
   * Get user info from access token
   */
  async getUserInfo(accessToken: string, tenantId?: string): Promise<any> {
    try {
      const config = await this.discoveryClient.discoverConfiguration(tenantId);
      
      const response = await fetch(config.userinfo_endpoint, {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${accessToken}`,
          'Accept': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`UserInfo request failed: ${response.status} ${response.statusText}`);
      }

      return await response.json();

    } catch (error) {
      console.error('Failed to get user info:', error);
      throw error;
    }
  }

  /**
   * Check if token is expired
   */
  isTokenExpired(tokenOrExpiry: string | number): boolean {
    let expiryTime: number;
    
    if (typeof tokenOrExpiry === 'string') {
      try {
        // Decode JWT without verification to get expiry
        const [, payload] = tokenOrExpiry.split('.');
        const decodedPayload = JSON.parse(atob(payload));
        expiryTime = decodedPayload.exp * 1000; // Convert to milliseconds
      } catch {
        return true; // Consider malformed tokens as expired
      }
    } else {
      expiryTime = tokenOrExpiry;
    }
    
    // Add 30 second buffer for clock skew
    return Date.now() >= (expiryTime - 30000);
  }

  /**
   * Decode JWT payload without verification (for inspection only)
   */
  decodeTokenPayload(token: string): DecodedToken | null {
    try {
      const [, payload] = token.split('.');
      return JSON.parse(atob(payload)) as DecodedToken;
    } catch {
      return null;
    }
  }

  /**
   * Clear token cache
   */
  clearCache(): void {
    this.tokenCache.clear();
    console.log('🧹 Token manager cache cleared');
  }

  /**
   * Get OIDC configuration and JWKS with caching
   */
  private async getConfigAndJWKS(tenantId?: string): Promise<{ config: OIDCConfiguration; jwks: any }> {
    const cacheKey = tenantId || 'default';
    const cached = this.tokenCache.get(cacheKey);
    
    if (cached && cached.expires > Date.now()) {
      return { config: cached.config, jwks: cached.jwks };
    }

    const config = await this.discoveryClient.discoverConfiguration(tenantId);
    const jwks = await this.discoveryClient.getJWKS(tenantId);
    
    // Cache for 1 hour
    this.tokenCache.set(cacheKey, {
      config,
      jwks,
      expires: Date.now() + 60 * 60 * 1000
    });
    
    return { config, jwks };
  }

  /**
   * Find appropriate signing key from JWKS
   */
  private findSigningKey(token: string, jwks: any): any {
    if (!jwks.keys || jwks.keys.length === 0) {
      return null;
    }

    // Try to extract key ID from JWT header
    try {
      const [header] = token.split('.');
      const decodedHeader = JSON.parse(atob(header));
      
      if (decodedHeader.kid) {
        const key = jwks.keys.find((k: any) => k.kid === decodedHeader.kid);
        if (key) return key;
      }
    } catch {
      // Ignore header parsing errors
    }

    // Fallback to first available key
    return jwks.keys[0];
  }

  /**
   * Extract token from Authorization header
   */
  static extractTokenFromHeader(authHeader: string): string | null {
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return null;
    }
    
    return authHeader.substring(7); // Remove 'Bearer ' prefix
  }

  /**
   * Create Authorization header from token
   */
  static createAuthHeader(token: string): string {
    return `Bearer ${token}`;
  }
}

// Singleton instance
let tokenManager: OIDCTokenManager | null = null;

export function getOIDCTokenManager(): OIDCTokenManager {
  if (!tokenManager) {
    tokenManager = new OIDCTokenManager();
  }
  return tokenManager;
}