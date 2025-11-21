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

// Challenge request/response types
export interface ChallengeRequest {
  wallet_address: string;
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
  error?: string;
}

/**
 * Direct Web3 API Client
 * Handles wallet authentication without OpenID flow
 */
class DirectWeb3ApiClient {
  private baseUrl: string;

  constructor() {
    // Enhanced backend URL resolution
    this.baseUrl =
      typeof window !== 'undefined'
        ? process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
        : process.env.BACKEND_URL || 'http://localhost:8080';

    console.log('🔧 DirectWeb3ApiClient initialized', {
      baseUrl: this.baseUrl,
    });
  }

  /**
   * Request a SIWE challenge from the backend
   */
  async requestChallenge(walletAddress: string): Promise<ChallengeResponse> {
    const url = `${this.baseUrl}/api/v1/auth/web3/challenge`;

    console.log('📝 Requesting SIWE challenge', {
      wallet_address: walletAddress,
      url,
    });

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
        },
        body: JSON.stringify({
          wallet_address: walletAddress,
        }),
      });

      if (!response.ok) {
        const errorData = await response
          .json()
          .catch(() => ({ error: 'Unknown error' }));
        console.error('❌ Challenge request failed', {
          status: response.status,
          statusText: response.statusText,
          error: errorData,
        });
        throw new Error(
          `Challenge request failed: ${errorData.error || response.statusText}`
        );
      }

      const challengeData: ChallengeResponse = await response.json();

      if (!challengeData.success) {
        console.error('❌ Challenge generation failed', {
          error: challengeData.error,
        });
        throw new Error(challengeData.error || 'Challenge generation failed');
      }

      console.log('✅ SIWE challenge received successfully', {
        wallet_address: challengeData.wallet_address,
        nonce: challengeData.nonce,
        expires_at: challengeData.expires_at,
        expires_at_iso: new Date(challengeData.expires_at * 1000).toISOString(),
      });

      return challengeData;
    } catch (error) {
      console.error('❌ Challenge request error', { error });
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
    const url = `${this.baseUrl}/api/v1/auth/web3/verify`;

    console.log('🔐 Verifying wallet signature', {
      wallet_address: request.wallet_address,
      url,
    });

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const errorData = await response
          .json()
          .catch(() => ({ error: 'Unknown error' }));
        console.error('❌ Signature verification failed', {
          status: response.status,
          statusText: response.statusText,
          error: errorData,
        });
        throw new Error(
          `Signature verification failed: ${errorData.error || response.statusText}`
        );
      }

      const verificationData: SignatureVerificationResponse =
        await response.json();

      if (!verificationData.success) {
        console.error('❌ Signature verification rejected', {
          error: verificationData.error,
        });
        throw new Error(
          verificationData.error || 'Signature verification failed'
        );
      }

      console.log('✅ Signature verification successful', {
        wallet_address: verificationData.wallet_address,
        permissions_count: verificationData.permissions?.length || 0,
        is_new_user: verificationData.is_new_user,
      });

      return verificationData;
    } catch (error) {
      console.error('❌ Signature verification error', { error });
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
      console.log('🚀 Starting complete wallet authentication flow', {
        wallet_address: walletAddress,
      });

      // Step 1: Request challenge
      const challenge = await this.requestChallenge(walletAddress);

      // Step 2: Sign message
      console.log('🖊️ Requesting signature for SIWE message...');
      const signature = await signMessage(challenge.message);
      console.log('✅ Signature received from wallet');

      // Step 3: Verify signature
      const verification = await this.verifySignature({
        wallet_address: walletAddress,
        signature,
        message: challenge.message,
        nonce: challenge.nonce,
      });

      console.log('🎉 Wallet authentication completed successfully!', {
        wallet_address: verification.wallet_address,
        permissions: verification.permissions?.length || 0,
        is_new_user: verification.is_new_user,
      });

      return verification;
    } catch (error) {
      console.error('❌ Wallet authentication failed', { error });
      throw error instanceof Error
        ? error
        : new Error('Wallet authentication failed');
    }
  }
}

// Export singleton instance
export const directWeb3Api = new DirectWeb3ApiClient();

// Export convenience functions
export const requestWalletChallenge = (walletAddress: string) =>
  directWeb3Api.requestChallenge(walletAddress);

export const verifyWalletSignature = (request: SignatureVerificationRequest) =>
  directWeb3Api.verifySignature(request);

export const authenticateWallet = (
  walletAddress: string,
  signMessage: (message: string) => Promise<string>
) => directWeb3Api.authenticateWallet(walletAddress, signMessage);

export default directWeb3Api;
