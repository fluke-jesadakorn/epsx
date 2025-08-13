/**
 * OIDC-Only Provider Detection Service
 * Simplified auth provider detection for single OIDC solution
 */

export interface ProviderDetectionResult {
  provider: string;
  metadata?: {
    issuer?: string;
    clientId?: string;
    scope?: string[];
  };
}

/**
 * Simple provider detector that always returns OIDC
 * since we only use OIDC authentication
 */
export default class ProviderDetector {
  /**
   * Detect provider from JWT token
   * Always returns 'oidc' since we only support OIDC
   */
  static detectFromToken(token: string): ProviderDetectionResult {
    if (!token || token.trim() === '') {
      return { provider: 'unknown' };
    }

    // For OIDC-only implementation, always return oidc
    return {
      provider: 'oidc',
      metadata: {
        issuer: process.env.NEXT_PUBLIC_OIDC_ISSUER || 'http://localhost:8080',
        clientId: process.env.NEXT_PUBLIC_OIDC_CLIENT_ID || 'epsx-frontend',
        scope: ['openid', 'profile', 'email']
      }
    };
  }

  /**
   * Validate if token is from supported provider
   */
  static isValidProvider(provider: string): boolean {
    return provider === 'oidc';
  }

  /**
   * Get supported providers list
   */
  static getSupportedProviders(): string[] {
    return ['oidc'];
  }
}