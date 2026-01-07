// Direct Web3 API Client for Backend Communication
// Bypasses Bearer token requirements for public authentication routes

import { UnifiedApiClient } from '@/shared/utils/api-client';

/**
 * Direct backend API communication for Web3 wallet authentication
 * Uses public routes that don't require Bearer tokens
 */

// Types matching backend API responses
export interface ChallengeResponse {
  success: boolean;
  nonce: string;
  message: string;
  expires_at: string;
  wallet_address: string;
  error?: string;
}

export interface VerifyResponse {
  success: boolean;
  wallet_address: string;
  permissions: string[];
  access_token: string;
  is_new_user: boolean;
  error?: string;
}

export interface ChallengeRequest {
  wallet_address: string;
}

export interface VerifyRequest {
  wallet_address: string;
  signature: string;
  message: string;
  nonce: string;
}

/**
 * Direct Web3 API client for authentication flows
 */
export class DirectWeb3Api {
  private backendUrl: string;
  private client: UnifiedApiClient;

  constructor(backendUrl?: string) {
    // Enhanced backend URL resolution
    this.backendUrl =
      backendUrl ||
      (typeof window !== 'undefined'
        ? process.env.NEXT_PUBLIC_BACKEND_URL ||
        window.location.origin.replace(/:300[0-9]/, ':8080')
        : process.env.BACKEND_URL || 'http://localhost:8080');

    // Initialize the unified API client
    this.client = new UnifiedApiClient({
      baseURL: this.backendUrl,
      platform: 'frontend',
      serverSide: typeof window === 'undefined',
    });
  }

  /**
   * Request SIWE challenge from backend
   * Uses public route: POST /api/auth/web3/challenge
   */
  async requestChallenge(walletAddress: string): Promise<ChallengeResponse> {
    try {
      const response = await this.client.post<ChallengeResponse>('/api/auth/web3/challenge', {
        wallet_address: walletAddress,
      });

      if (!response.success || !response.data) {
        throw new Error(response.error || response.message || 'Challenge generation failed');
      }

      return response.data;
    } catch (error) {
      console.error('❌ Challenge request failed:', error);
      throw error;
    }
  }

  /**
   * Verify wallet signature and create/save wallet user
   * Uses public route: POST /api/auth/web3/verify
   */
  async verifySignature(request: VerifyRequest): Promise<VerifyResponse> {
    try {
      const response = await this.client.post<VerifyResponse>('/api/auth/web3/verify', request);

      if (!response.success || !response.data) {
        throw new Error(response.error || response.message || 'Signature verification failed');
      }

      return {
        success: response.data.success,
        wallet_address: response.data.wallet_address,
        permissions: response.data.permissions || [],
        access_token: response.data.access_token,
        is_new_user: response.data.is_new_user,
      };
    } catch (error) {
      console.error('❌ Signature verification failed:', error);
      throw error;
    }
  }

  /**
   * Complete Web3 authentication flow
   * 1. Request challenge
   * 2. Sign message with wallet
   * 3. Verify signature and save wallet user
   */
  async authenticateWallet(
    walletAddress: string,
    signMessage: (message: string) => Promise<string>
  ): Promise<VerifyResponse> {
    try {
      // Step 1: Request challenge
      const challenge = await this.requestChallenge(walletAddress);

      // Step 2: Sign SIWE message
      const signature = await signMessage(challenge.message);

      // Step 3: Verify signature and save wallet
      const result = await this.verifySignature({
        wallet_address: walletAddress,
        signature,
        message: challenge.message,
        nonce: challenge.nonce,
      });

      return result;
    } catch (error) {
      console.error('❌ Web3 authentication flow failed:', error);
      throw error;
    }
  }
}

// Default instance for easy importing
export const directWeb3Api = new DirectWeb3Api();

// Convenience functions for direct usage
export const requestWalletChallenge = (walletAddress: string) =>
  directWeb3Api.requestChallenge(walletAddress);

export const verifyWalletSignature = (request: VerifyRequest) =>
  directWeb3Api.verifySignature(request);

export const authenticateWallet = (
  walletAddress: string,
  signMessage: (message: string) => Promise<string>
) => directWeb3Api.authenticateWallet(walletAddress, signMessage);
