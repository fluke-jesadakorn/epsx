// ============================================================================
// WEB3 OPENID AUTHENTICATION SERVICE
// Web3 wallet signing that triggers OpenID Connect token issuance
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Web3 wallet signing triggers OpenID token issuance
 * - Frontend only handles wallet connection and signing
 * - Backend validates signatures and issues OpenID tokens
 * - No permission logic in frontend - backend decides everything
 */

import { logger } from '@/lib/shared';
import { openidApiClient, Web3AuthRequest, UserInfoResponse } from './openid-api-client';

// Web3 Challenge Response
export interface Web3ChallengeResponse {
  nonce: string;
  message: string;
  wallet_address: string;
}

// Web3 Authentication Result
export interface Web3AuthResult {
  success: boolean;
  user?: UserInfoResponse;
  error?: string;
}

// Web3 Signature Request
export interface Web3SignatureRequest {
  wallet_address: string;
  signature: string;
  message: string;
  nonce: string;
}

/**
 * Web3 OpenID Authentication Service
 * Handles Web3 wallet authentication and OpenID token management
 */
export class Web3OpenIDService {
  private static instance: Web3OpenIDService;
  private currentUser: UserInfoResponse | null = null;
  private listeners: Set<(user: UserInfoResponse | null) => void> = new Set();

  private constructor() {}

  static getInstance(): Web3OpenIDService {
    if (!Web3OpenIDService.instance) {
      Web3OpenIDService.instance = new Web3OpenIDService();
    }
    return Web3OpenIDService.instance;
  }

  // ============================================================================
  // USER STATE MANAGEMENT
  // ============================================================================

  getCurrentUser(): UserInfoResponse | null {
    return this.currentUser;
  }

  isAuthenticated(): boolean {
    return this.currentUser !== null && openidApiClient.isAuthenticated();
  }

  subscribe(callback: (user: UserInfoResponse | null) => void): () => void {
    this.listeners.add(callback);
    return () => this.listeners.delete(callback);
  }

  private notifyListeners(): void {
    this.listeners.forEach(callback => callback(this.currentUser));
  }

  // ============================================================================
  // WEB3 AUTHENTICATION FLOW
  // ============================================================================

  /**
   * Request Web3 challenge from backend
   */
  async requestChallenge(walletAddress: string): Promise<Web3ChallengeResponse> {
    const url = `${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/web3/challenge`;
    
    logger.info('Requesting Web3 challenge', { wallet_address: walletAddress });

    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        wallet_address: walletAddress,
        client_id: process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend'
      }),
    });

    if (!response.ok) {
      const errorData = await response.json();
      logger.error('Challenge request failed', { error: errorData });
      throw new Error(`Challenge request failed: ${errorData.message || 'Unknown error'}`);
    }

    const challengeData: Web3ChallengeResponse = await response.json();
    
    logger.info('Web3 challenge received successfully', {
      wallet_address: challengeData.wallet_address,
      nonce: challengeData.nonce
    });

    return challengeData;
  }

  /**
   * Authenticate with Web3 signature and get OpenID tokens
   */
  async authenticateWithSignature(signatureRequest: Web3SignatureRequest): Promise<Web3AuthResult> {
    try {
      logger.info('Authenticating with Web3 signature', {
        wallet_address: signatureRequest.wallet_address
      });

      // Create Web3 auth request for OpenID token issuance
      const web3AuthRequest: Web3AuthRequest = {
        wallet_address: signatureRequest.wallet_address,
        signature: signatureRequest.signature,
        message: signatureRequest.message,
        nonce: signatureRequest.nonce,
        client_id: process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend'
      };

      // Get OpenID tokens from backend
      const tokenResponse = await openidApiClient.authenticateWithWeb3(web3AuthRequest);
      
      logger.info('OpenID tokens received, fetching user info');

      // Get user information
      const user = await openidApiClient.getCurrentUser();
      
      if (!user) {
        logger.error('Failed to fetch user info after authentication');
        return {
          success: false,
          error: 'Failed to fetch user information'
        };
      }

      this.currentUser = user;
      this.notifyListeners();

      logger.info('Web3 authentication successful', {
        wallet_address: user.wallet_address,
        tier_level: user.tier_level,
        permissions_count: user.permissions.length
      });

      return {
        success: true,
        user
      };

    } catch (error) {
      logger.error('Web3 authentication failed', { error });
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Authentication failed'
      };
    }
  }

  /**
   * Load current user from existing tokens
   */
  async loadCurrentUser(): Promise<UserInfoResponse | null> {
    try {
      if (!openidApiClient.isAuthenticated()) {
        logger.info('No valid tokens available');
        this.currentUser = null;
        this.notifyListeners();
        return null;
      }

      logger.info('Loading current user from existing tokens');
      
      const user = await openidApiClient.getCurrentUser();
      
      if (user) {
        this.currentUser = user;
        this.notifyListeners();
        
        logger.info('Current user loaded successfully', {
          wallet_address: user.wallet_address,
          tier_level: user.tier_level
        });
      } else {
        this.currentUser = null;
        this.notifyListeners();
        logger.info('No current user found');
      }

      return user;

    } catch (error) {
      logger.error('Failed to load current user', { error });
      this.currentUser = null;
      this.notifyListeners();
      return null;
    }
  }

  /**
   * Logout user and revoke tokens
   */
  async logout(): Promise<void> {
    try {
      logger.info('Logging out user');
      
      // Revoke OpenID tokens
      await openidApiClient.revokeTokens();
      
      // Clear user state
      this.currentUser = null;
      this.notifyListeners();
      
      logger.info('User logged out successfully');
      
      // Trigger wallet disconnect event for Web3 components
      if (typeof window !== 'undefined') {
        const event = new CustomEvent('epsx:disconnect-wallet');
        window.dispatchEvent(event);
      }

    } catch (error) {
      logger.error('Logout failed', { error });
      throw new Error('Logout failed');
    }
  }

  /**
   * Refresh user data
   */
  async refreshUser(): Promise<void> {
    try {
      logger.info('Refreshing user data');
      
      const user = await openidApiClient.getCurrentUser();
      
      if (user) {
        this.currentUser = user;
        this.notifyListeners();
        logger.info('User data refreshed successfully');
      } else {
        // If we can't get user data, tokens might be invalid
        this.currentUser = null;
        this.notifyListeners();
        logger.warn('Failed to refresh user data - tokens may be invalid');
      }

    } catch (error) {
      logger.error('Failed to refresh user data', { error });
      throw new Error('Failed to refresh user data');
    }
  }

  // ============================================================================
  // CONVENIENCE METHODS
  // ============================================================================

  /**
   * Check if user has admin access (display helper only)
   * Note: This is NOT for authorization - backend makes all security decisions
   */
  isAdminUser(): boolean {
    if (!this.currentUser) return false;
    return this.currentUser.permissions.some(p => p.startsWith('admin:'));
  }

  /**
   * Get user tier (display helper only)
   */
  getUserTier(): string {
    return this.currentUser?.tier_level || 'free';
  }

  /**
   * Get wallet address
   */
  getWalletAddress(): string | null {
    return this.currentUser?.wallet_address || null;
  }

  /**
   * Get user permissions (display helper only)
   * Note: These are for display purposes only - backend validates all permissions
   */
  getUserPermissions(): string[] {
    return this.currentUser?.permissions || [];
  }

  /**
   * Make authenticated API request
   */
  async makeApiRequest<T>(endpoint: string, options?: RequestInit) {
    return openidApiClient.makeAuthenticatedRequest<T>(endpoint, options);
  }
}

// Export singleton instance
export const web3OpenidService = Web3OpenIDService.getInstance();

// Convenience export
export default web3OpenidService;