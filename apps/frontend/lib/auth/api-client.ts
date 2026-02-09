/**
 * FRONTEND AUTH API CLIENT
 * Wrapper around SharedWeb3AuthClient
 */

import { SharedWeb3AuthClient } from '@/shared/auth/client';

// Keep interfaces for backward compatibility
export type {
  Web3TokenResponse as OpenIDTokenResponse, UserInfoResponse, Web3AuthRequest
} from '@/shared/auth/client';

export class OpenIDApiClient extends SharedWeb3AuthClient {
  private static instance: OpenIDApiClient;

  private constructor() {
    super(
      process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID ?? 'epsx-frontend',
      process.env.NEXT_PUBLIC_BACKEND_URL ?? 'http://localhost:8080'
    );
  }

  static getInstance(): OpenIDApiClient {
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    if (!OpenIDApiClient.instance) {
      OpenIDApiClient.instance = new OpenIDApiClient();
    }
    return OpenIDApiClient.instance;
  }

  // Compatibility method
  async authenticateWithWeb3(request: any) {
    return this.authenticateWithSignature(request);
  }

  // Compatibility method
  async revokeTokens() {
    return this.logout();
  }
}

// Export singleton instance
export const openidApiClient = OpenIDApiClient.getInstance();
export default openidApiClient;
