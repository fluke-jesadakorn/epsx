// Firebase Token Validation Service
// Exchanges Firebase tokens for unified backend JWTs

import { getFirebaseAuth } from './firebase-auth';

/**
 * Unified JWT response from backend
 */
export interface UnifiedJWT {
  access_token: string;
  token_type: string;
  expires_at: string;
  expires_in: number;
  session_id: string;
  jti: string;
  refresh_token?: string;
}

/**
 * Token exchange request payload
 */
interface TokenExchangeRequest {
  firebase_token: string;
  provider_hint?: 'firebase';
  client_id?: string;
}

/**
 * Token exchange response
 */
interface TokenExchangeResponse {
  success: boolean;
  data?: UnifiedJWT;
  error?: {
    code: string;
    message: string;
    details?: any;
  };
}

/**
 * Firebase token validation and exchange service
 */
export class FirebaseTokenValidator {
  private backendUrl: string;
  private clientId: string;

  constructor(backendUrl?: string, clientId?: string) {
    this.backendUrl = backendUrl || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    this.clientId = clientId || 'frontend-client';
  }

  /**
   * Validate Firebase token and exchange for unified JWT
   * This is the main integration point with our multi-provider backend
   */
  async validateAndExchange(firebaseToken: string): Promise<UnifiedJWT> {
    const request: TokenExchangeRequest = {
      firebase_token: firebaseToken,
      provider_hint: 'firebase',
      client_id: this.clientId
    };

    try {
      const response = await fetch(`${this.backendUrl}/api/auth/token/exchange`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json'
        },
        body: JSON.stringify(request)
      });

      if (!response.ok) {
        const errorData = await response.text();
        throw new TokenValidationError(
          `Token exchange failed: HTTP ${response.status}`,
          response.status,
          errorData
        );
      }

      const result: TokenExchangeResponse = await response.json();

      if (!result.success || !result.data) {
        throw new TokenValidationError(
          result.error?.message || 'Token exchange failed',
          400,
          result.error
        );
      }

      return result.data;
    } catch (error) {
      if (error instanceof TokenValidationError) {
        throw error;
      }

      // Network or parsing errors
      throw new TokenValidationError(
        `Network error during token exchange: ${error instanceof Error ? error.message : 'Unknown error'}`,
        0,
        error
      );
    }
  }

  /**
   * Get current Firebase token and exchange it
   */
  async getCurrentUserToken(): Promise<UnifiedJWT | null> {
    const firebaseAuth = getFirebaseAuth();
    const firebaseToken = await firebaseAuth.getIdToken();

    if (!firebaseToken) {
      return null;
    }

    return this.validateAndExchange(firebaseToken);
  }

  /**
   * Refresh current Firebase token and exchange it
   */
  async refreshCurrentUserToken(): Promise<UnifiedJWT | null> {
    const firebaseAuth = getFirebaseAuth();
    const firebaseToken = await firebaseAuth.getIdToken(true); // Force refresh

    if (!firebaseToken) {
      return null;
    }

    return this.validateAndExchange(firebaseToken);
  }

  /**
   * Check if Firebase user is authenticated and get unified token
   */
  async getValidatedToken(): Promise<UnifiedJWT | null> {
    const firebaseAuth = getFirebaseAuth();
    
    if (!firebaseAuth.isAuthenticated()) {
      return null;
    }

    try {
      return await this.getCurrentUserToken();
    } catch (error) {
      console.error('Token validation failed:', error);
      
      // Try to refresh the Firebase token once
      try {
        return await this.refreshCurrentUserToken();
      } catch (refreshError) {
        console.error('Token refresh failed:', refreshError);
        return null;
      }
    }
  }

  /**
   * Validate token and check if it's still valid
   */
  async isTokenValid(unifiedJwt: UnifiedJWT): Promise<boolean> {
    try {
      const expiresAt = new Date(unifiedJwt.expires_at);
      const now = new Date();
      const bufferMinutes = 5; // 5-minute buffer before expiry

      // Check if token will expire in the next 5 minutes
      if (expiresAt.getTime() - now.getTime() < bufferMinutes * 60 * 1000) {
        return false;
      }

      // TODO: Optionally validate against backend
      return true;
    } catch (error) {
      console.error('Token validation error:', error);
      return false;
    }
  }

  /**
   * Auto-refresh token if needed
   */
  async ensureValidToken(currentToken?: UnifiedJWT): Promise<UnifiedJWT | null> {
    if (currentToken && await this.isTokenValid(currentToken)) {
      return currentToken;
    }

    // Token is invalid or expired, get a fresh one
    return this.getValidatedToken();
  }
}

/**
 * Token validation error
 */
export class TokenValidationError extends Error {
  public readonly statusCode: number;
  public readonly details: any;

  constructor(message: string, statusCode: number = 400, details?: any) {
    super(message);
    this.name = 'TokenValidationError';
    this.statusCode = statusCode;
    this.details = details;
  }

  /**
   * Get user-friendly error message
   */
  getUserFriendlyMessage(): string {
    switch (this.statusCode) {
      case 401:
        return 'Authentication expired. Please sign in again.';
      case 403:
        return 'Access denied. You may not have the required permissions.';
      case 404:
        return 'Authentication service not available.';
      case 429:
        return 'Too many requests. Please try again later.';
      case 500:
        return 'Server error. Please try again later.';
      default:
        return this.message;
    }
  }
}

/**
 * Singleton instance
 */
let firebaseTokenValidator: FirebaseTokenValidator | null = null;

/**
 * Get Firebase token validator instance
 */
export function getFirebaseTokenValidator(): FirebaseTokenValidator {
  if (!firebaseTokenValidator) {
    firebaseTokenValidator = new FirebaseTokenValidator();
  }
  return firebaseTokenValidator;
}

/**
 * Initialize Firebase token validator with custom configuration
 */
export function initializeFirebaseTokenValidator(
  backendUrl?: string,
  clientId?: string
): FirebaseTokenValidator {
  firebaseTokenValidator = new FirebaseTokenValidator(backendUrl, clientId);
  return firebaseTokenValidator;
}

export default FirebaseTokenValidator;