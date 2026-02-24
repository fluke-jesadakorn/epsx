// ============================================================================
// SHARED DIRECT WEB3 API CLIENT
// Direct backend communication without Bearer tokens for wallet authentication
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Direct API communication for public routes
 * - No Bearer token requirements for authentication routes
 * - Real SIWE signature verification
 * - Database wallet saving and retrieval
 */

import { fetchWithTimeout } from '../utils/fetch-with-timeout';
import { logger } from '../utils/logger';

// Challenge request/response types
export interface ChallengeRequest {
  wallet_address: string;
  /** Cloudflare Turnstile CAPTCHA token */
  turnstile_token?: string;
}

export interface ChallengeResponse {
  success: boolean;
  nonce: string;
  message: string;
  wallet_address: string;
  expires_at: number;
  error?: string;
}

// Signature verification types
export interface SignatureVerificationRequest {
  wallet_address: string;
  signature: string;
  message: string;
  nonce: string;
}

export interface SignatureVerificationResponse {
  success: boolean;
  wallet_address: string;
  permissions: string[];
  is_new_user: boolean;
  access_token?: string;
  refresh_token?: string;
  error?: string;
}

/**
 * Direct Web3 API Client
 * Handles wallet authentication without OpenID flow
 */
class DirectWeb3ApiClient {
  private baseUrl: string;

  constructor() {
    // Enhanced backend URL resolution with dynamic environment detection
    this.baseUrl =
      typeof window !== 'undefined'
        ? // Client-side: Try env var, then dynamic port replacement (3000 -> 8080)
        process.env.NEXT_PUBLIC_BACKEND_URL ??
        window.location.origin.replace(/:300[0-9]/, ':8080')
        : // Server-side: Try server env var, then default
        process.env.BACKEND_URL ?? 'http://127.0.0.1:8080';

    logger.debug('[AUTH] DirectWeb3ApiClient initialized', {
      baseUrl: this.baseUrl,
      context: typeof window !== 'undefined' ? 'browser' : 'server',
      location: typeof window !== 'undefined' ? window.location.origin : 'N/A',
    });
  }

  /**
   * Request a SIWE challenge from the backend
   */
  async requestChallenge(walletAddress: string, turnstileToken?: string): Promise<ChallengeResponse> {
    const url = `${this.baseUrl}/api/auth/web3/challenge`;

    logger.debug('[AUTH] Requesting SIWE challenge', {
      wallet_address: walletAddress,
      url,
      has_turnstile: Boolean(turnstileToken),
    });

    try {
      const body: ChallengeRequest = {
        wallet_address: walletAddress,
      };
      if (turnstileToken) {
        body.turnstile_token = turnstileToken;
      }

      const response = await fetchWithTimeout(url, {
        method: 'POST',
        timeout: 15000,
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
        },
        body: JSON.stringify(body),
      });

      if (!response.ok) {
        const errorData = (await response
          .json()
          .catch(() => ({ error: 'Unknown error' }))) as { error?: string };
        logger.error('[AUTH] Error: Challenge request failed', {
          status: response.status,
          statusText: response.statusText,
          error: errorData,
        });
        throw new Error(
          `Challenge request failed: ${errorData.error ?? response.statusText}`
        );
      }

      const challengeData = (await response.json()) as ChallengeResponse;

      if (!challengeData.success) {
        logger.error('[AUTH] Error: Challenge generation failed', {
          error: challengeData.error,
        });
        throw new Error(challengeData.error ?? 'Challenge generation failed');
      }

      logger.debug('[AUTH] SIWE challenge received successfully', {
        wallet_address: challengeData.wallet_address,
        nonce: challengeData.nonce,
        expires_at: challengeData.expires_at,
        expires_at_iso: new Date(challengeData.expires_at * 1000).toISOString(),
      });

      return challengeData;
    } catch (error) {
      logger.error('[AUTH] Error: Challenge request network/timeout error', {
        name: error instanceof Error ? error.name : 'UnknownError',
        message: error instanceof Error ? error.message : String(error),
        url,
      });
      throw error instanceof Error
        ? error
        : new Error('Failed to request challenge');
    }
  }

  /**
   * Verify wallet signature and authenticate
   */
  async verifySignature(
    request: SignatureVerificationRequest
  ): Promise<SignatureVerificationResponse> {
    const url = `${this.baseUrl}/api/auth/web3/verify`;

    logger.debug('[AUTH] Verifying wallet signature', {
      wallet_address: request.wallet_address,
      url,
    });

    try {
      const response = await fetchWithTimeout(url, {
        method: 'POST',
        timeout: 15000,
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const errorData = (await response
          .json()
          .catch(() => ({ error: 'Unknown error' }))) as { error?: string };
        logger.error('[AUTH] Error: Signature verification failed', {
          status: response.status,
          statusText: response.statusText,
          error: errorData,
        });
        throw new Error(
          `Signature verification failed: ${errorData.error ?? response.statusText}`
        );
      }

      const verificationData =
        (await response.json()) as SignatureVerificationResponse;

      if (!verificationData.success) {
        logger.error('[AUTH] Error: Signature verification rejected', {
          error: verificationData.error,
        });
        throw new Error(
          verificationData.error ?? 'Signature verification failed'
        );
      }

      logger.info('[AUTH] Signature verification successful', {
        wallet_address: verificationData.wallet_address,
        permissions_count: verificationData.permissions.length,
        is_new_user: verificationData.is_new_user,
      });

      return verificationData;
    } catch (error) {
      logger.error('[AUTH] Error: Signature verification network/timeout error', {
        name: error instanceof Error ? error.name : 'UnknownError',
        message: error instanceof Error ? error.message : String(error),
        url,
      });
      throw error instanceof Error
        ? error
        : new Error('Failed to verify signature');
    }
  }

  /**
   * Complete authentication flow: request challenge + verify signature
   */
  async authenticateWallet(
    walletAddress: string,
    signMessage: (message: string) => Promise<string>
  ): Promise<SignatureVerificationResponse> {
    try {
      logger.debug('[AUTH] Starting complete wallet authentication flow', {
        wallet_address: walletAddress,
      });

      // Step 1: Request challenge
      const challenge = await this.requestChallenge(walletAddress);

      // Step 2: Sign message
      logger.debug('[AUTH] Requesting signature for SIWE message...');
      const signature = await signMessage(challenge.message);
      logger.debug('[AUTH] Signature received from wallet');

      // Step 3: Verify signature
      const verification = await this.verifySignature({
        wallet_address: walletAddress,
        signature,
        message: challenge.message,
        nonce: challenge.nonce,
      });

      logger.info('[AUTH] Wallet authentication completed successfully!', {
        wallet_address: verification.wallet_address,
        permissions: verification.permissions.length,
        is_new_user: verification.is_new_user,
      });

      return verification;
    } catch (error) {
      logger.error('[AUTH] Error: Wallet authentication failed', { error });
      throw error instanceof Error
        ? error
        : new Error('Wallet authentication failed');
    }
  }
}

// Export singleton instance
export const directWeb3Api = new DirectWeb3ApiClient();

// Export convenience functions
export const requestWalletChallenge = (walletAddress: string, turnstileToken?: string) =>
  directWeb3Api.requestChallenge(walletAddress, turnstileToken);

export const verifyWalletSignature = (request: SignatureVerificationRequest) =>
  directWeb3Api.verifySignature(request);

export const authenticateWallet = (
  walletAddress: string,
  signMessage: (message: string) => Promise<string>
) => directWeb3Api.authenticateWallet(walletAddress, signMessage);

export default directWeb3Api;
