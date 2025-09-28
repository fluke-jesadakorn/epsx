// ============================================================================
// OPENID CONNECT API CLIENT - FRONTEND BEARER TOKEN MANAGEMENT
// Standard OpenID Connect client with Web3 wallet authentication trigger
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Frontend is "dumb" - no permission logic whatsoever
 * - Backend is the source of truth for ALL authorization decisions
 * - Only Bearer token management in Authorization header
 * - Web3 wallet signing triggers OpenID token issuance
 * - Display exactly what backend tells us to display
 */

import { logger } from '@/lib/shared';

// OpenID Connect Token Response (from backend)
export interface OpenIDTokenResponse {
  access_token: string;      // JWT Bearer token for API access
  token_type: string;        // Always "Bearer"
  expires_in: number;        // Seconds until expiration
  refresh_token: string;     // For token renewal
  id_token: string;          // OpenID identity token
  scope: string;             // "openid profile permissions"
}

// Web3 Authentication Request
export interface Web3AuthRequest {
  wallet_address: string;
  signature: string;
  message: string;
  nonce: string;
  client_id: string;        // "epsx-frontend" or "epsx-admin"
}

// OpenID Token Refresh Request
export interface TokenRefreshRequest {
  refresh_token: string;
  client_id: string;
}

// OpenID Error Response
export interface OpenIDErrorResponse {
  error: string;
  error_description: string;
  error_uri?: string;
}

// User Info Response (from /api/v1/auth/userinfo)
export interface UserInfoResponse {
  sub: string;                    // Wallet address
  wallet_address: string;         // Web3 wallet address
  tier_level: string;            // User tier
  auth_method: string;           // "web3_siwe"
  permissions: string[];         // Backend-determined permissions
}

// API Response Structure (unified from backend)
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: {
    code: number;
    message: string;
    details?: any;
  };
}

/**
 * OpenID Connect API Client
 * Handles all Bearer token authentication with backend OpenID endpoints
 */
export class OpenIDApiClient {
  private static instance: OpenIDApiClient;
  private accessToken: string | null = null;
  private refreshToken: string | null = null;
  private tokenExpiry: number | null = null;
  private clientId: string;

  private constructor() {
    this.clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend';
    this.loadTokensFromStorage();
  }

  static getInstance(): OpenIDApiClient {
    if (!OpenIDApiClient.instance) {
      OpenIDApiClient.instance = new OpenIDApiClient();
    }
    return OpenIDApiClient.instance;
  }

  // ============================================================================
  // TOKEN STORAGE MANAGEMENT
  // ============================================================================

  private loadTokensFromStorage(): void {
    if (typeof window === 'undefined') return;
    
    try {
      this.accessToken = localStorage.getItem('openid_access_token');
      this.refreshToken = localStorage.getItem('openid_refresh_token');
      const expiry = localStorage.getItem('openid_token_expiry');
      this.tokenExpiry = expiry ? parseInt(expiry, 10) : null;
    } catch (error) {
      logger.warn('Failed to load tokens from storage', { error });
    }
  }

  private saveTokensToStorage(): void {
    if (typeof window === 'undefined') return;
    
    try {
      if (this.accessToken) {
        localStorage.setItem('openid_access_token', this.accessToken);
      }
      if (this.refreshToken) {
        localStorage.setItem('openid_refresh_token', this.refreshToken);
      }
      if (this.tokenExpiry) {
        localStorage.setItem('openid_token_expiry', this.tokenExpiry.toString());
      }
    } catch (error) {
      logger.warn('Failed to save tokens to storage', { error });
    }
  }

  private clearTokensFromStorage(): void {
    if (typeof window === 'undefined') return;
    
    try {
      localStorage.removeItem('openid_access_token');
      localStorage.removeItem('openid_refresh_token');
      localStorage.removeItem('openid_token_expiry');
    } catch (error) {
      logger.warn('Failed to clear tokens from storage', { error });
    }
  }

  // ============================================================================
  // OPENID CONNECT TOKEN MANAGEMENT
  // ============================================================================

  /**
   * Authenticate with Web3 wallet and get OpenID tokens
   * Main entry point: Web3 signature → OpenID Connect tokens
   */
  async authenticateWithWeb3(request: Web3AuthRequest): Promise<OpenIDTokenResponse> {
    const url = `${process.env.NEXT_PUBLIC_BACKEND_URL}/api/v1/auth/web3/token`;
    
    logger.info('Authenticating with Web3 wallet for OpenID tokens', {
      wallet_address: request.wallet_address,
      client_id: request.client_id
    });

    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const errorData: OpenIDErrorResponse = await response.json();
      logger.error('Web3 OpenID authentication failed', { 
        error: errorData.error,
        description: errorData.error_description
      });
      throw new Error(`Authentication failed: ${errorData.error_description}`);
    }

    const tokenResponse: OpenIDTokenResponse = await response.json();
    
    // Store tokens securely
    this.accessToken = tokenResponse.access_token;
    this.refreshToken = tokenResponse.refresh_token;
    this.tokenExpiry = Date.now() + (tokenResponse.expires_in * 1000);
    this.saveTokensToStorage();

    logger.info('OpenID tokens received successfully', {
      token_type: tokenResponse.token_type,
      expires_in: tokenResponse.expires_in,
      scope: tokenResponse.scope
    });

    return tokenResponse;
  }

  /**
   * Refresh OpenID Connect tokens
   */
  async refreshTokens(): Promise<boolean> {
    if (!this.refreshToken) {
      logger.warn('No refresh token available');
      return false;
    }

    const url = `${process.env.NEXT_PUBLIC_BACKEND_URL}/api/v1/auth/token/refresh`;
    const request: TokenRefreshRequest = {
      refresh_token: this.refreshToken,
      client_id: this.clientId,
    };

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const errorData: OpenIDErrorResponse = await response.json();
        logger.error('Token refresh failed', { 
          error: errorData.error,
          description: errorData.error_description
        });
        
        // If refresh fails, clear tokens and require re-authentication
        this.clearTokens();
        return false;
      }

      const tokenResponse: OpenIDTokenResponse = await response.json();
      
      // Update tokens
      this.accessToken = tokenResponse.access_token;
      this.refreshToken = tokenResponse.refresh_token;
      this.tokenExpiry = Date.now() + (tokenResponse.expires_in * 1000);
      this.saveTokensToStorage();

      logger.info('Tokens refreshed successfully');
      return true;

    } catch (error) {
      logger.error('Token refresh request failed', { error });
      this.clearTokens();
      return false;
    }
  }

  /**
   * Revoke refresh token (logout)
   */
  async revokeTokens(): Promise<void> {
    if (!this.refreshToken) {
      this.clearTokens();
      return;
    }

    const url = `${process.env.NEXT_PUBLIC_BACKEND_URL}/api/v1/auth/token/revoke`;
    const request: TokenRefreshRequest = {
      refresh_token: this.refreshToken,
      client_id: this.clientId,
    };

    try {
      await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });
    } catch (error) {
      logger.warn('Token revocation failed', { error });
      // Continue with local cleanup even if server revocation fails
    }

    this.clearTokens();
    logger.info('Tokens revoked and cleared');
  }

  private clearTokens(): void {
    this.accessToken = null;
    this.refreshToken = null;
    this.tokenExpiry = null;
    this.clearTokensFromStorage();
  }

  // ============================================================================
  // BEARER TOKEN API CALLS
  // ============================================================================

  /**
   * Check if we have valid access token
   */
  isAuthenticated(): boolean {
    return !!(this.accessToken && this.tokenExpiry && this.tokenExpiry > Date.now());
  }

  /**
   * Check if token needs refresh (expires in next 5 minutes)
   */
  needsRefresh(): boolean {
    if (!this.tokenExpiry) return false;
    const fiveMinutesFromNow = Date.now() + (5 * 60 * 1000);
    return this.tokenExpiry <= fiveMinutesFromNow;
  }

  /**
   * Get current user info from backend
   */
  async getCurrentUser(): Promise<UserInfoResponse | null> {
    const response = await this.makeAuthenticatedRequest('/api/v1/auth/userinfo');
    return response.success ? (response.data as UserInfoResponse) : null;
  }

  /**
   * Make authenticated API request with Bearer token
   * Automatically handles token refresh if needed
   */
  async makeAuthenticatedRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    // Ensure we have valid tokens
    if (!this.isAuthenticated()) {
      if (this.refreshToken && await this.refreshTokens()) {
        // Token refreshed successfully, continue
      } else {
        // No valid tokens, require re-authentication
        return {
          success: false,
          error: {
            code: 401,
            message: 'Authentication required'
          }
        };
      }
    }

    // Refresh token if it expires soon
    if (this.needsRefresh() && this.refreshToken) {
      await this.refreshTokens();
    }

    const url = endpoint.startsWith('http') 
      ? endpoint 
      : `${process.env.NEXT_PUBLIC_BACKEND_URL}${endpoint}`;

    const response = await fetch(url, {
      ...options,
      headers: {
        ...options.headers,
        'Authorization': `Bearer ${this.accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    if (response.status === 401) {
      // Token might be invalid, try refreshing once
      if (this.refreshToken && await this.refreshTokens()) {
        // Retry with new token
        const retryResponse = await fetch(url, {
          ...options,
          headers: {
            ...options.headers,
            'Authorization': `Bearer ${this.accessToken}`,
            'Content-Type': 'application/json',
          },
        });

        return this.handleApiResponse(retryResponse);
      } else {
        // Refresh failed, clear tokens
        this.clearTokens();
        return {
          success: false,
          error: {
            code: 401,
            message: 'Authentication expired'
          }
        };
      }
    }

    return this.handleApiResponse(response);
  }

  private async handleApiResponse<T>(response: Response): Promise<ApiResponse<T>> {
    try {
      const data = await response.json();

      if (response.ok) {
        return {
          success: true,
          data
        };
      } else {
        return {
          success: false,
          error: {
            code: response.status,
            message: data.error_description || data.message || 'Request failed',
            details: data
          }
        };
      }
    } catch (error) {
      return {
        success: false,
        error: {
          code: response.status,
          message: `Request failed: ${error}`
        }
      };
    }
  }

  // ============================================================================
  // OPENID CONNECT DISCOVERY
  // ============================================================================

  /**
   * Get OpenID Connect discovery document
   */
  async getDiscoveryDocument(): Promise<any> {
    const url = `${process.env.NEXT_PUBLIC_BACKEND_URL}/.well-known/openid_configuration`;
    const response = await fetch(url);
    return response.json();
  }

  /**
   * Get JWT public keys for token validation
   */
  async getJwks(): Promise<any> {
    const url = `${process.env.NEXT_PUBLIC_BACKEND_URL}/.well-known/jwks.json`;
    const response = await fetch(url);
    return response.json();
  }
}

// Export singleton instance
export const openidApiClient = OpenIDApiClient.getInstance();

// Convenience export
export default openidApiClient;