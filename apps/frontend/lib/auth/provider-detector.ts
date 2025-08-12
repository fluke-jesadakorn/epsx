// Provider Detection Logic
// Determines authentication provider from tokens, cookies, or context

import { jwtDecode } from 'jwt-decode';

/**
 * Supported authentication providers
 */
export type AuthProvider = 'firebase' | 'oidc' | 'unknown';

/**
 * Provider detection result
 */
export interface ProviderDetectionResult {
  provider: AuthProvider;
  confidence: number; // 0-1 scale
  metadata?: Record<string, any>;
}

/**
 * JWT token claims interface for detection
 */
interface TokenClaims {
  iss?: string;    // Issuer
  aud?: string;    // Audience
  sub?: string;    // Subject
  exp?: number;    // Expiry
  iat?: number;    // Issued at
  firebase?: any;  // Firebase-specific claims
  provider?: string; // Custom provider field
  session_type?: string; // Session type
}

/**
 * Provider detection utility class
 */
export class ProviderDetector {
  private static readonly FIREBASE_ISSUERS = [
    'https://securetoken.google.com/',
    'firebase',
    'google.com'
  ];

  private static readonly OIDC_ISSUERS = [
    'http://localhost:8080',
    'https://api.epsx.com',
    process.env.NEXT_PUBLIC_BACKEND_URL
  ].filter(Boolean);

  /**
   * Detect provider from JWT token
   */
  static detectFromToken(token: string): ProviderDetectionResult {
    if (!token) {
      return { provider: 'unknown', confidence: 0 };
    }

    try {
      // Decode JWT without verification (for detection only)
      const claims = jwtDecode<TokenClaims>(token);
      
      // Check for explicit provider claim
      if (claims.provider) {
        switch (claims.provider.toLowerCase()) {
          case 'firebase':
            return { 
              provider: 'firebase', 
              confidence: 1.0,
              metadata: { issuer: claims.iss, explicit: true }
            };
          case 'oidc':
            return { 
              provider: 'oidc', 
              confidence: 1.0,
              metadata: { issuer: claims.iss, explicit: true }
            };
        }
      }

      // Check for session_type (unified tokens)
      if (claims.session_type === 'unified') {
        return { 
          provider: 'oidc', 
          confidence: 0.9,
          metadata: { issuer: claims.iss, unified: true }
        };
      }

      // Detect by issuer
      if (claims.iss) {
        // Firebase detection
        const isFirebaseIssuer = this.FIREBASE_ISSUERS.some(issuer => 
          claims.iss!.includes(issuer) || issuer.includes(claims.iss!)
        );
        
        if (isFirebaseIssuer || claims.firebase) {
          return {
            provider: 'firebase',
            confidence: 0.9,
            metadata: { issuer: claims.iss, firebase_claims: !!claims.firebase }
          };
        }

        // OIDC detection
        const isOIDCIssuer = this.OIDC_ISSUERS.some(issuer => 
          claims.iss === issuer
        );
        
        if (isOIDCIssuer) {
          return {
            provider: 'oidc',
            confidence: 0.8,
            metadata: { issuer: claims.iss }
          };
        }
      }

      // Fallback detection based on token structure
      if (claims.firebase || (claims.aud && claims.aud.includes('firebase'))) {
        return {
          provider: 'firebase',
          confidence: 0.7,
          metadata: { structure_based: true }
        };
      }

      // If it has standard JWT claims but no Firebase indicators
      if (claims.sub && claims.exp && claims.iat) {
        return {
          provider: 'oidc',
          confidence: 0.6,
          metadata: { structure_based: true }
        };
      }

      return { provider: 'unknown', confidence: 0.3 };

    } catch (error) {
      console.debug('Token detection failed:', error);
      return { provider: 'unknown', confidence: 0 };
    }
  }

  /**
   * Detect provider from browser cookies
   */
  static detectFromCookies(): ProviderDetectionResult {
    if (typeof window === 'undefined') {
      return { provider: 'unknown', confidence: 0 };
    }

    const cookies = this.parseCookies();
    
    // Check for Firebase session cookies
    const firebaseCookies = [
      'next-auth.session-token',
      '__Secure-next-auth.session-token',
      'authjs.session-token',
      '__Secure-authjs.session-token',
      'firebase_session'
    ];

    const hasFirebaseCookie = firebaseCookies.some(name => name in cookies);
    
    if (hasFirebaseCookie) {
      return {
        provider: 'firebase',
        confidence: 0.8,
        metadata: { source: 'cookies', cookies: firebaseCookies.filter(name => name in cookies) }
      };
    }

    // Check for OIDC session cookies
    const oidcCookies = [
      'oidc_session',
      'unified_session',
      'jwt_token'
    ];

    const hasOIDCCookie = oidcCookies.some(name => name in cookies);
    
    if (hasOIDCCookie) {
      return {
        provider: 'oidc',
        confidence: 0.8,
        metadata: { source: 'cookies', cookies: oidcCookies.filter(name => name in cookies) }
      };
    }

    // Check for generic session cookies
    if ('sess_id' in cookies) {
      return {
        provider: 'oidc', // Assume OIDC for generic session
        confidence: 0.5,
        metadata: { source: 'cookies', generic: true }
      };
    }

    return { provider: 'unknown', confidence: 0 };
  }

  /**
   * Detect provider from localStorage
   */
  static detectFromLocalStorage(): ProviderDetectionResult {
    if (typeof window === 'undefined') {
      return { provider: 'unknown', confidence: 0 };
    }

    try {
      const firebaseKeys = [
        'firebase:authUser',
        'firebase:persistence',
        'firebase_auth_token'
      ];

      const hasFirebaseStorage = firebaseKeys.some(key => 
        localStorage.getItem(key) !== null
      );

      if (hasFirebaseStorage) {
        return {
          provider: 'firebase',
          confidence: 0.7,
          metadata: { source: 'localStorage' }
        };
      }

      const oidcKeys = [
        'oidc_token',
        'unified_jwt',
        'access_token'
      ];

      const hasOIDCStorage = oidcKeys.some(key => 
        localStorage.getItem(key) !== null
      );

      if (hasOIDCStorage) {
        return {
          provider: 'oidc',
          confidence: 0.7,
          metadata: { source: 'localStorage' }
        };
      }

      return { provider: 'unknown', confidence: 0 };

    } catch (error) {
      console.debug('localStorage detection failed:', error);
      return { provider: 'unknown', confidence: 0 };
    }
  }

  /**
   * Comprehensive provider detection using multiple sources
   */
  static detectProvider(): ProviderDetectionResult {
    const detectionResults: ProviderDetectionResult[] = [];

    // Try token detection first (if token is available)
    try {
      const token = this.extractTokenFromContext();
      if (token) {
        const tokenResult = this.detectFromToken(token);
        if (tokenResult.confidence > 0.5) {
          return tokenResult;
        }
        detectionResults.push(tokenResult);
      }
    } catch (error) {
      console.debug('Token extraction failed:', error);
    }

    // Cookie detection
    const cookieResult = this.detectFromCookies();
    detectionResults.push(cookieResult);

    // localStorage detection
    const storageResult = this.detectFromLocalStorage();
    detectionResults.push(storageResult);

    // Find the result with highest confidence
    const bestResult = detectionResults.reduce((best, current) => 
      current.confidence > best.confidence ? current : best,
      { provider: 'unknown' as AuthProvider, confidence: 0 }
    );

    return bestResult;
  }

  /**
   * Check if a specific provider is currently active
   */
  static isProviderActive(provider: AuthProvider): boolean {
    const detection = this.detectProvider();
    return detection.provider === provider && detection.confidence > 0.5;
  }

  /**
   * Get provider priority order for authentication attempts
   */
  static getProviderPriority(): AuthProvider[] {
    const detection = this.detectProvider();
    
    if (detection.confidence > 0.7) {
      // High confidence in current provider
      return detection.provider === 'firebase' 
        ? ['firebase', 'oidc']
        : ['oidc', 'firebase'];
    }

    // Default priority order
    return ['firebase', 'oidc'];
  }

  /**
   * Extract token from various contexts (cookies, headers, etc.)
   */
  private static extractTokenFromContext(): string | null {
    if (typeof window === 'undefined') {
      return null;
    }

    // Check for bearer token in any stored location
    const sources = [
      () => localStorage.getItem('access_token'),
      () => localStorage.getItem('firebase_token'),
      () => sessionStorage.getItem('jwt_token'),
      () => this.extractTokenFromCookie()
    ];

    for (const source of sources) {
      try {
        const token = source();
        if (token && token.includes('.')) { // Basic JWT format check
          return token;
        }
      } catch (error) {
        continue;
      }
    }

    return null;
  }

  /**
   * Extract token from cookies
   */
  private static extractTokenFromCookie(): string | null {
    const cookies = this.parseCookies();
    
    // Try various cookie names
    const tokenCookies = [
      'next-auth.session-token',
      'jwt_token',
      'access_token',
      'unified_jwt'
    ];

    for (const cookieName of tokenCookies) {
      if (cookies[cookieName]) {
        return cookies[cookieName];
      }
    }

    return null;
  }

  /**
   * Parse document cookies into key-value pairs
   */
  private static parseCookies(): Record<string, string> {
    if (typeof document === 'undefined') {
      return {};
    }

    return document.cookie
      .split(';')
      .reduce((cookies, cookie) => {
        const [key, value] = cookie.trim().split('=');
        if (key && value) {
          cookies[key] = decodeURIComponent(value);
        }
        return cookies;
      }, {} as Record<string, string>);
  }
}

/**
 * Hook for provider detection in React components
 */
export function useProviderDetection(): {
  provider: AuthProvider;
  confidence: number;
  metadata?: Record<string, any>;
  detect: () => ProviderDetectionResult;
  isActive: (provider: AuthProvider) => boolean;
} {
  const result = ProviderDetector.detectProvider();

  return {
    provider: result.provider,
    confidence: result.confidence,
    metadata: result.metadata,
    detect: () => ProviderDetector.detectProvider(),
    isActive: (provider: AuthProvider) => ProviderDetector.isProviderActive(provider)
  };
}

export default ProviderDetector;