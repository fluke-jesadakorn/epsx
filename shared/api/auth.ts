/**
 * UNIFIED AUTH API CLIENT
 * 
 * Consolidates all authentication-related API calls across EPSX applications.
 * Eliminates proxy routes by providing direct backend communication.
 * 
 * Features:
 * - Web3 wallet authentication (challenge/verify flow)
 * - Session management and validation
 * - Token refresh and revocation
 * - Multi-platform support (frontend/admin)
 * - SIWE (Sign-In with Ethereum) standard compliance
 */

import { API_ROUTES } from '../config/route-constants';
import { UnifiedApiClient } from '../utils/api-client';

// ============================================================================
// AUTH TYPES
// ============================================================================

export interface Web3Challenge {
  nonce: string;
  message: string;
  expires_at: number;
  chain_id?: number;
}

export interface Web3ChallengeRequest {
  wallet_address: string;
  chain_id?: number;
}

export interface Web3ChallengeResponse {
  success: boolean;
  nonce: string;
  message: string;
  expires_at: number;
  api_version?: string;
  access_level?: string;
}

export interface Web3VerifyRequest {
  wallet_address: string;
  signature: string;
  message: string;
  nonce: string;
  chain_id?: number;
}

export interface Web3VerifyResponse {
  success: boolean;
  authenticated: boolean;
  access_token?: string;
  refresh_token?: string;
  permissions: string[];
  user_id?: string;
  wallet_address: string;
  expires_in?: number;
  api_version?: string;
  access_level?: string;
}

export interface SessionInfo {
  valid: boolean;
  user_id?: string;
  wallet_address?: string;
  permissions: string[];
  expires_at?: number;
  tier?: string;
  platform?: string;
}

export interface SessionValidationRequest {
  token?: string;
  wallet_address?: string;
}

export interface SessionValidationResponse {
  success: boolean;
  authenticated: boolean;
  session_valid: boolean;
  user_id?: string;
  wallet_address?: string;
  permissions: string[];
  api_version?: string;
  access_level?: string;
}

export interface TokenRefreshRequest {
  refresh_token: string;
}

export interface TokenRefreshResponse {
  success: boolean;
  access_token: string;
  refresh_token?: string;
  expires_in: number;
  api_version?: string;
}

export interface LogoutResponse {
  success: boolean;
  message: string;
  logged_out: boolean;
  api_version?: string;
}

export interface UserProfile {
  user_id: string;
  wallet_address: string;
  permissions: string[];
  tier: string;
  created_at?: string;
  last_login?: string;
  platform?: string;
}

export interface UserProfileResponse {
  success: boolean;
  data: UserProfile;
  api_version?: string;
  access_level?: string;
}

export interface PermissionsResponse {
  success: boolean;
  data: {
    permissions: string[];
    tier: string;
    expires_at: number | null;
    groups?: string[];
  };
  api_version?: string;
  access_level?: string;
}

// ============================================================================
// AUTH API CLIENT CLASS
// ============================================================================

export class AuthAPIClient {
  constructor(private client: UnifiedApiClient) { }

  // ============================================================================
  // WEB3 AUTHENTICATION FLOW
  // ============================================================================

  /**
   * Request authentication challenge for wallet
   * Route: POST /api/v1/auth/web3/challenge
   */
  async getWeb3Challenge(request: Web3ChallengeRequest): Promise<Web3Challenge> {
    const response = await this.client.post<Web3ChallengeResponse>(
      API_ROUTES.AUTH.WEB3_CHALLENGE,
      request,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to get Web3 challenge: ${response.error}`);
    }

    return {
      nonce: response.data.nonce,
      message: response.data.message,
      expires_at: response.data.expires_at,
    };
  }

  /**
   * Verify signed challenge and authenticate
   * Route: POST /api/v1/auth/web3/verify
   */
  async verifyWeb3Signature(request: Web3VerifyRequest): Promise<Web3VerifyResponse> {
    const response = await this.client.post<Web3VerifyResponse>(
      API_ROUTES.AUTH.WEB3_VERIFY,
      request,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to verify Web3 signature: ${response.error}`);
    }

    // Update client token if authentication successful
    if (response.data.authenticated && response.data.access_token) {
      this.client.setAuthToken(response.data.access_token);
    }

    return response.data;
  }

  /**
   * Complete Web3 authentication flow (challenge + verify)
   * Convenience method that handles the full authentication process
   */
  async authenticateWeb3(
    walletAddress: string,
    signMessageFn: (message: string) => Promise<string>,
    chainId?: number
  ): Promise<Web3VerifyResponse> {
    try {
      // Step 1: Get challenge
      const challenge = await this.getWeb3Challenge({
        wallet_address: walletAddress,
        chain_id: chainId,
      });

      // Step 2: Sign message
      const signature = await signMessageFn(challenge.message);

      if (!signature) {
        throw new Error('Message signing was cancelled');
      }

      // Step 3: Verify signature
      const verificationResult = await this.verifyWeb3Signature({
        wallet_address: walletAddress,
        signature,
        message: challenge.message,
        nonce: challenge.nonce,
        chain_id: chainId,
      });

      return verificationResult;
    } catch (error) {
      console.error('Web3 authentication failed:', error);
      throw error;
    }
  }

  // ============================================================================
  // SESSION MANAGEMENT
  // ============================================================================

  /**
   * Validate current session
   * Route: GET /api/v1/auth/web3/session
   */
  async validateSession(request?: SessionValidationRequest): Promise<SessionValidationResponse> {
    const response = await this.client.get<SessionValidationResponse>(
      API_ROUTES.AUTH.WEB3_SESSION,
      request,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to validate session: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Verify session with detailed information
   * Route: POST /api/v1/auth/session/verify
   */
  async verifySession(token?: string): Promise<SessionValidationResponse> {
    const response = await this.client.post<SessionValidationResponse>(
      API_ROUTES.AUTH.SESSION_VERIFY || API_ROUTES.AUTH.WEB3_SESSION, // Fallback to session route
      { token },
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to verify session: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Refresh authentication token
   * Route: POST /api/v1/auth/session/refresh
   */
  async refreshToken(refreshToken: string): Promise<TokenRefreshResponse> {
    const response = await this.client.post<TokenRefreshResponse>(
      API_ROUTES.AUTH.SESSION_REFRESH,
      { refresh_token: refreshToken },
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to refresh token: ${response.error}`);
    }

    // Update client token
    if (response.data.access_token) {
      this.client.setAuthToken(response.data.access_token);
    }

    return response.data;
  }

  /**
   * Store session data (admin-specific)
   * Route: POST /api/auth/web3/store-session
   * Note: This may need to be moved to standardized endpoint
   */
  async storeSession(sessionData: {
    wallet_address: string;
    access_token: string;
    permissions: string[];
  }): Promise<{ success: boolean; message: string }> {
    const response = await this.client.post(
      API_ROUTES.AUTH.WEB3_SESSION, // Updated to use standard session endpoint
      sessionData,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to store session: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Clear stored session (admin-specific)  
   * Route: POST /api/auth/web3/clear-session
   * Note: This may need to be moved to standardized endpoint
   */
  async clearSession(): Promise<{ success: boolean; message: string }> {
    const response = await this.client.post(
      API_ROUTES.AUTH.WEB3_LOGOUT, // Updated to use standard logout endpoint
      {},
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'admin',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to clear session: ${response.error}`);
    }

    return response.data;
  }

  // ============================================================================
  // LOGOUT
  // ============================================================================

  /**
   * Logout and invalidate session
   * Route: DELETE /api/v1/auth/web3/logout
   */
  async logout(): Promise<LogoutResponse> {
    const response = await this.client.delete<LogoutResponse>(
      API_ROUTES.AUTH.WEB3_LOGOUT,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    // Clear local token regardless of response
    this.client.removeAuthToken();

    if (!this.client.isApiSuccess(response)) {
      // Don't throw error for logout - just log it
      console.warn(`Logout request failed: ${response.error}`);
      return {
        success: false,
        message: response.error || 'Logout request failed',
        logged_out: true, // Still consider locally logged out
      };
    }

    return response.data;
  }

  // ============================================================================
  // USER PROFILE & PERMISSIONS
  // ============================================================================

  /**
   * Get current user profile
   * Route: GET /api/v1/auth/users/profile
   */
  async getUserProfile(): Promise<UserProfile> {
    const response = await this.client.get<UserProfileResponse>(
      API_ROUTES.AUTH.PROFILE,
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to get user profile: ${response.error}`);
    }

    return response.data.data;
  }

  /**
   * Update user profile
   * Route: PUT /api/v1/auth/users/profile
   */
  async updateUserProfile(updates: Partial<UserProfile>): Promise<{ success: boolean; message: string }> {
    const response = await this.client.put(
      API_ROUTES.AUTH.PROFILE,
      updates,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to update user profile: ${response.error}`);
    }

    return response.data;
  }

  /**
   * Get current user permissions
   * Route: GET /api/v1/auth/users/permissions
   */
  async getUserPermissions(): Promise<{
    permissions: string[];
    tier: string;
    expires_at: number | null;
    groups?: string[];
  }> {
    const response = await this.client.get<PermissionsResponse>(
      API_ROUTES.AUTH.PERMISSIONS,
      undefined,
      {
        headers: {
          'X-API-Version': 'v1',
          'X-Access-Level': 'auth',
        },
      }
    );

    if (!this.client.isApiSuccess(response)) {
      throw new Error(`Failed to get user permissions: ${response.error}`);
    }

    return response.data.data;
  }

  // ============================================================================
  // UTILITY METHODS
  // ============================================================================

  /**
   * Check if user has specific permission
   */
  async hasPermission(permission: string): Promise<boolean> {
    try {
      const permissions = await this.getUserPermissions();
      return permissions.permissions.includes(permission) ||
        permissions.permissions.some(p =>
          p === 'admin:*:*' ||
          p.startsWith(permission.split(':').slice(0, 2).join(':') + ':*')
        );
    } catch (error) {
      console.warn(`Permission check failed: ${error}`);
      return false;
    }
  }

  /**
   * Check if user has admin permissions
   */
  async isAdmin(): Promise<boolean> {
    return this.hasPermission('admin:*:*');
  }

  /**
   * Get authentication status
   */
  async getAuthStatus(): Promise<{
    authenticated: boolean;
    wallet_address?: string;
    permissions: string[];
    tier?: string;
  }> {
    try {
      const session = await this.validateSession();
      return {
        authenticated: session.authenticated && session.session_valid,
        wallet_address: session.wallet_address,
        permissions: session.permissions,
      };
    } catch (error) {
      return {
        authenticated: false,
        permissions: [],
      };
    }
  }

  /**
   * Generate SIWE message for wallet signing
   * Following EIP-4361 standard
   */
  static generateSIWEMessage(options: {
    domain: string;
    address: string;
    statement?: string;
    uri: string;
    version: string;
    chainId: number;
    nonce: string;
    issuedAt: string;
    expirationTime?: string;
  }): string {
    const {
      domain,
      address,
      statement = 'Sign in to EPSX',
      uri,
      version,
      chainId,
      nonce,
      issuedAt,
      expirationTime,
    } = options;

    let message = `${domain} wants you to sign in with your Ethereum account:\n${address}\n\n${statement}`;

    message += `\n\nURI: ${uri}`;
    message += `\nVersion: ${version}`;
    message += `\nChain ID: ${chainId}`;
    message += `\nNonce: ${nonce}`;
    message += `\nIssued At: ${issuedAt}`;

    if (expirationTime) {
      message += `\nExpiration Time: ${expirationTime}`;
    }

    return message;
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create auth API client for frontend applications
 */
export function createAuthClient(client: UnifiedApiClient): AuthAPIClient {
  return new AuthAPIClient(client);
}

/**
 * Create auth client with automatic platform detection
 */
export function createPlatformAuthClient(platform: 'frontend' | 'admin' = 'frontend'): AuthAPIClient {
  if (platform === 'admin') {
    const { createAdminApiClient } = require('../utils/api-client');
    return new AuthAPIClient(createAdminApiClient());
  } else {
    const { createFrontendApiClient } = require('../utils/api-client');
    return new AuthAPIClient(createFrontendApiClient());
  }
}

// Type alias for backward compatibility with useApiClient
export type AuthApi = AuthAPIClient;