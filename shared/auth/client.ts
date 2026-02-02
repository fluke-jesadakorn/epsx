// ============================================================================
// SHARED WEB3 AUTHENTICATION CLIENT
// Unified Web3-first authentication system for both frontend and admin-frontend
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Same Web3-first authentication for frontend and admin-frontend
 * - SIWE (Sign-In With Ethereum) standard for wallet authentication
 * - No permission logic in frontends - backend decides everything
 * - Bearer token authentication for all API calls
 * - Display exactly what backend tells us to display
 */

import {
  clearClientSideCookies,
  COOKIES,
  getClientCookie,
  getClientCookieJSON,
  removeClientCookie,
  setClientCookie,
  setClientCookieJSON
} from './cookies';

// Web3 JWT Token Response (from backend)
export interface Web3TokenResponse {
  access_token: string; // Web3 JWT Bearer token for API access
  token_type: string; // Always "Bearer"
  expires_in: number; // Seconds until expiration
  refresh_token?: string; // Optional refresh token
  wallet_address: string; // Authenticated wallet address
  permissions: string[]; // User permissions from backend
  user_id: string; // User identifier
}

// Web3 Authentication Request
export interface Web3AuthRequest {
  wallet_address: string;
  signature: string;
  message: string;
  nonce: string;
  client_id: string; // "epsx-frontend" or "epsx-admin"
}

// Web3 Challenge Response
export interface Web3ChallengeResponse {
  nonce: string;
  message: string;
  wallet_address: string;
}

// User Info Response (from /api/auth/userinfo)
export interface UserInfoResponse {
  sub: string; // Wallet address
  wallet_address: string; // Web3 wallet address
  tier_level: string; // User tier
  auth_method: string; // "web3_siwe"
  permissions: string[]; // Backend-determined permissions (display only)
  email?: string; // Optional email (for compatibility)
  access?: string; // JWT access token for SSE authentication
  packageTier?: string; // Package tier for display (alias of tier_level)
  group?: string; // User group (for permission display)
  is_admin?: boolean; // Admin status flag
}

// Unified API Response Structure (from backend)
export interface UnifiedApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: {
    code: number;
    message: string;
    reason: string;
  };
  meta?: {
    timestamp: string;
    permissions?: {
      user_tier: string;
      available_actions: string[];
      restricted_actions?: Array<{
        action: string;
        reason: string;
        required_tier?: string;
      }>;
    };
  };
}

/**
 * Shared Web3 Authentication Client
 * Used by both frontend and admin-frontend applications
 */
export class SharedWeb3AuthClient {
  private accessToken: string | null = null;
  private refreshToken: string | null = null;
  private tokenExpiry: number | null = null;
  private user: UserInfoResponse | null = null;
  private listeners: Set<(user: UserInfoResponse | null) => void> = new Set();
  private clientId: string;
  private backendUrl: string;
  private challengeCache: Map<string, { promise: Promise<Web3ChallengeResponse>; timestamp: number }> = new Map();
  private authInProgress: boolean = false;

  constructor(clientId: string, backendUrl: string) {
    this.clientId = clientId;
    this.backendUrl = backendUrl;
    this.loadTokensFromStorage();
  }

  // ============================================================================
  // AUTHENTICATION STATE MANAGEMENT
  // ============================================================================

  isAuthenticated(): boolean {
    return !!(
      this.accessToken &&
      this.tokenExpiry &&
      this.tokenExpiry > Date.now()
    );
  }

  getCurrentUser(): UserInfoResponse | null {
    return this.user;
  }

  subscribe(callback: (user: UserInfoResponse | null) => void): () => void {
    this.listeners.add(callback);
    return () => this.listeners.delete(callback);
  }

  private notifyListeners(): void {
    this.listeners.forEach(callback => callback(this.user));
  }

  // ============================================================================
  // TOKEN STORAGE (cookies only)
  // ============================================================================

  private loadTokensFromStorage(): void {
    if (typeof window === 'undefined') return;

    try {
      // Load access token from cookies
      const accessToken = getClientCookie(COOKIES.access_token);
      if (accessToken) {
        this.accessToken = accessToken;
      }

      // Load refresh token from cookies
      this.refreshToken = getClientCookie(COOKIES.refresh_token);

      // Load expiry from cookies
      const expiry = getClientCookie(COOKIES.expires_at);
      this.tokenExpiry = expiry ? parseInt(expiry, 10) : null;

      // Load user from cookies
      let storedUser = getClientCookieJSON<UserInfoResponse>(COOKIES.user);

      // Fallback: decode user from JWT if cookie missing but token exists
      if (!storedUser && this.accessToken) {
        try {
          const payloadPart = this.accessToken.split('.')[1];
          if (payloadPart) {
            const payload = JSON.parse(atob(payloadPart));
            if (payload.sub && payload.sub.startsWith('0x')) {
              console.log('[AUTH] Decoded user identity from Access Token');
              storedUser = {
                sub: payload.sub,
                wallet_address: payload.sub,
                tier_level: payload.package_tier || payload.tier_level || 'basic',
                auth_method: 'web3_siwe',
                permissions: payload.permissions || [],
                access: this.accessToken
              };
            }
          }
        } catch (e) {
          console.warn('Failed to decode access token for user recovery', e);
        }
      }

      if (storedUser) {
        this.user = storedUser;
      }

      console.log('[AUTH] SharedWeb3AuthClient: Initial cookie state loaded', {
        clientId: this.clientId,
        hasAccessToken: !!this.accessToken,
        hasUser: !!this.user,
        wallet: this.user?.wallet_address?.slice(0, 8),
        isExpired: this.isExpired(),
        source: storedUser ? 'cookie' : (this.accessToken ? 'jwt' : 'none')
      });
    } catch (error) {
      console.warn('Failed to load tokens from cookies', { error });
    }
  }

  private saveTokensToStorage(): void {
    if (typeof window === 'undefined') return;

    try {
      if (this.accessToken) {
        setClientCookie(COOKIES.access_token, this.accessToken, this.tokenExpiry ? Math.floor((this.tokenExpiry - Date.now()) / 1000) : 3600);
      }

      if (this.refreshToken) {
        setClientCookie(COOKIES.refresh_token, this.refreshToken, 2592000); // 30 days
      }

      if (this.tokenExpiry) {
        setClientCookie(COOKIES.expires_at, this.tokenExpiry.toString(), 2592000);
      }

      if (this.user) {
        setClientCookieJSON(COOKIES.user, this.user, 2592000);
      }

      console.log('[AUTH] Client: Session state updated and cookies set', {
        clientId: this.clientId,
        hasUser: !!this.user,
      });
    } catch (error) {
      console.warn('Failed to save tokens to cookies', { error });
    }
  }

  private clearTokensFromStorage(): void {
    if (typeof window === 'undefined') return;

    try {
      // Clear auth tokens specific to this client
      removeClientCookie(COOKIES.access_token);
      removeClientCookie(COOKIES.refresh_token);
      removeClientCookie(COOKIES.expires_at);
      removeClientCookie(COOKIES.user);

      // Also clear shared cookies
      clearClientSideCookies();
    } catch (error) {
      console.warn('Failed to clear tokens from cookies', { error });
    }
  }



  // ============================================================================
  // WEB3 AUTHENTICATION FLOW
  // ============================================================================

  async requestChallenge(
    walletAddress: string
  ): Promise<Web3ChallengeResponse> {
    // Deduplicate concurrent challenge requests
    const cacheKey = `${walletAddress}_${this.clientId}`;
    const now = Date.now();
    const cached = this.challengeCache.get(cacheKey);

    // Return cached promise if request is in-flight or recent (within 60s)
    if (cached && (now - cached.timestamp) < 60000) {
      console.log('[AUTH] Reusing existing challenge request');
      return cached.promise;
    }

    const challengeUrl = `${this.backendUrl}/api/auth/web3/challenge`;

    console.log('[AUTH] Requesting Web3 challenge', {
      url: challengeUrl,
      wallet_address: walletAddress,
      backend_url: this.backendUrl,
    });

    const challengePromise = (async () => {
      try {
        const response = await fetch(challengeUrl, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Accept: 'application/json',
          },
          body: JSON.stringify({
            wallet_address: walletAddress,
          }),
          signal: AbortSignal.timeout(10000), // 10s timeout
        });

        console.log('[AUTH] Challenge response received', {
          status: response.status,
          ok: response.ok,
          statusText: response.statusText,
          headers: Object.fromEntries(response.headers.entries()),
        });

        if (!response.ok) {
          let errorMessage = `Challenge request failed: ${response.status} ${response.statusText}`;
          let errorData: any = null;

          // Read the response body only once
          const contentType = response.headers.get('content-type');
          try {
            if (response.status === 404) {
              // Specific error for 404 - endpoint not found
              errorMessage = `Authentication endpoint not found. The backend may need to be updated with Web3 authentication routes.`;
            } else if (contentType && contentType.includes('application/json')) {
              errorData = await response.json();
              errorMessage = errorData.message || errorMessage;
            } else {
              const errorText = await response.text();
              errorMessage = `Challenge request failed: ${response.status} ${response.statusText}. ${errorText}`;
              errorData = { text: errorText };
            }
          } catch (bodyReadError) {
            console.warn('Failed to read error response body:', bodyReadError);
            // Use only status information if body reading fails
            if (response.status === 404) {
              errorMessage = `Authentication endpoint not found. The backend may need to be updated with Web3 authentication routes.`;
            }
          }

          // Special handling for 404 - likely route configuration issue
          let errorDetails: any;
          if (response.status === 404) {
            errorDetails = {
              url: challengeUrl,
              status: response.status,
              statusText: response.statusText,
              headers: Object.fromEntries(response.headers.entries()),
              errorData,
              troubleshooting: 'Authentication endpoint not found. The backend may need to be updated with the correct Web3 authentication routes (/api/auth/web3/*).',
              requestBody: {
                wallet_address: walletAddress,
              },
              backendUrl: this.backendUrl,
            };

            console.error('[AUTH] Error: Web3 challenge endpoint not found (404)', errorDetails);
          } else {
            errorDetails = {
              url: challengeUrl,
              status: response.status,
              statusText: response.statusText,
              headers: Object.fromEntries(response.headers.entries()),
              errorData,
              troubleshooting: this.getTroubleshootingHints(response.status),
              requestBody: {
                wallet_address: walletAddress,
              },
              backendUrl: this.backendUrl,
            };

            console.error('[AUTH] Error: Challenge request failed with full details:', errorDetails);
          }

          throw new Error(errorMessage);
        }

        const challengeData = await response.json();
        console.log('[AUTH] Challenge request successful', challengeData);

        // Clear cache after success
        setTimeout(() => this.challengeCache.delete(cacheKey), 60000);

        return challengeData;
      } catch (error) {
        // Clear cache on error
        this.challengeCache.delete(cacheKey);

        let errorMessage = 'Challenge request failed';

        if (error instanceof TypeError && error.message.includes('fetch')) {
          errorMessage = `Network error: Cannot connect to backend at ${this.backendUrl}. Please check your internet connection and ensure the backend service is running.`;
        } else if (error instanceof Error) {
          if (error.name === 'TimeoutError' || error.name === 'AbortError') {
            errorMessage = 'Request timed out. Please check your connection.';
          } else {
            errorMessage = error.message;
          }
        } else {
          errorMessage = String(error);
        }

        const enhancedError = new Error(errorMessage);
        console.error('[AUTH] Error: Challenge request', {
          url: challengeUrl,
          backend_url: this.backendUrl,
          original_error: error,
          wallet_address: walletAddress,
          error_type: typeof error
        });

        throw enhancedError;
      }
    })();

    // Cache the promise
    this.challengeCache.set(cacheKey, { promise: challengePromise, timestamp: now });

    return challengePromise;
  }

  /**
   * Get troubleshooting hints based on HTTP status code
   */
  private getTroubleshootingHints(status: number): string {
    switch (status) {
      case 404:
        return 'The challenge endpoint was not found. Check if the backend routes are properly configured.';
      case 405:
        return 'Method not allowed. The endpoint may not accept POST requests.';
      case 500:
        return 'Backend server error. Check backend logs and database connectivity.';
      case 503:
        return 'Backend service unavailable. The server may be starting up or overloaded.';
      case 0:
        return 'Network error. Backend server may not be running or CORS may be blocking the request.';
      default:
        return 'Unexpected error. Check backend logs for more details.';
    }
  }

  async authenticateWithSignature(request: {
    wallet_address: string;
    signature: string;
    message: string;
    nonce: string;
  }): Promise<{ success: boolean; user?: UserInfoResponse; error?: string }> {
    // Prevent parallel authentication attempts
    if (this.authInProgress) {
      return {
        success: false,
        error: 'Authentication already in progress. Please wait.',
      };
    }

    this.authInProgress = true;

    try {
      // Call backend verify endpoint directly
      const response = await fetch(`${this.backendUrl}/api/auth/web3/verify`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
        signal: AbortSignal.timeout(15000), // 15s timeout for verification
      });

      if (!response.ok) {
        let errorData: any = null;
        let errorMessage = 'Verification failed';

        try {
          errorData = await response.json();
          errorMessage = errorData.message || errorData.error || `Verification failed: ${response.status} ${response.statusText}`;
        } catch (parseError) {
          errorMessage = `Verification failed: ${response.status} ${response.statusText}`;
        }

        // Special handling for 404 - likely route configuration issue
        if (response.status === 404) {
          errorMessage = `Authentication endpoint not found. The backend may need to be updated with the correct Web3 authentication routes.`;
          console.error('[AUTH] Error: Web3 authentication endpoint not found (404)', {
            status: response.status,
            statusText: response.statusText,
            url: `${this.backendUrl}/api/auth/web3/verify`,
            wallet_address: request.wallet_address,
            troubleshooting: 'Check if backend has correct Web3 authentication routes deployed'
          });
        } else {
          console.error('Web3 verification HTTP error', {
            status: response.status,
            statusText: response.statusText,
            errorData,
            wallet_address: request.wallet_address
          });
        }

        throw new Error(errorMessage);
      }

      const result = await response.json();

      // Check if authentication was successful
      if (!result.success || !result.authenticated) {
        const errorMsg = result.message || result.error || 'Authentication failed';
        console.error('Web3 authentication failed in backend', {
          success: result.success,
          authenticated: result.authenticated,
          message: result.message,
          error: result.error,
          wallet_address: request.wallet_address
        });
        throw new Error(errorMsg);
      }

      // Store access token
      console.log('[AUTH] Auth result from backend:', {
        hasAccessToken: !!result.access_token,
        accessTokenLength: result.access_token?.length || 0,
        accessTokenPreview: result.access_token?.substring(0, 50) + '...',
      });
      this.accessToken = result.access_token;
      this.refreshToken = result.refresh_token;
      // Use backend-provided expiry, default to 1 hour if not specified
      const expiresInSeconds = result.expires_in || 3600;
      this.tokenExpiry = Date.now() + expiresInSeconds * 1000;

      // Create user object from response
      const user: UserInfoResponse = {
        sub: result.wallet_address,
        wallet_address: result.wallet_address,
        tier_level: result.tier_level || 'free', // Default tier if not provided
        auth_method: 'web3_siwe',
        permissions: result.permissions || [],
        access: result.access_token,
      };

      this.user = user;
      console.log('[AUTH] Saving tokens to storage, accessToken is:', this.accessToken ? 'SET' : 'EMPTY');
      this.saveTokensToStorage();
      this.notifyListeners();

      return { success: true, user };
    } catch (error) {
      let errorMessage = 'Authentication failed';

      if (error instanceof TypeError && error.message.includes('fetch')) {
        errorMessage = `Network error: Cannot connect to backend at ${this.backendUrl}. Please check your internet connection and ensure the backend service is running.`;
        console.error('Web3 authentication network error', {
          backend_url: this.backendUrl,
          wallet_address: request.wallet_address,
          error: error.message
        });
      } else if (error instanceof Error) {
        if (error.name === 'TimeoutError' || error.name === 'AbortError') {
          errorMessage = 'Verification timed out. Please try again.';
        } else {
          errorMessage = error.message;
        }
        console.error('Web3 authentication error', {
          error: error.message,
          wallet_address: request.wallet_address
        });
      } else {
        console.error('Web3 authentication unknown error', {
          error: String(error),
          wallet_address: request.wallet_address
        });
      }

      return {
        success: false,
        error: errorMessage,
      };
    } finally {
      this.authInProgress = false;
    }
  }

  private async getWeb3Tokens(
    request: Web3AuthRequest
  ): Promise<Web3TokenResponse> {
    const response = await fetch(`${this.backendUrl}/api/auth/web3/verify`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(
        `Web3 verification failed: ${errorData.error_description || errorData.message || 'Unknown error'}`
      );
    }

    return response.json();
  }

  // ============================================================================
  // TOKEN MANAGEMENT
  // ============================================================================

  private isExpired(): boolean {
    if (!this.tokenExpiry) return true;
    return this.tokenExpiry <= Date.now();
  }

  // Refresh tokens using backend refresh endpoint
  async refreshTokens(): Promise<boolean> {
    if (!this.refreshToken) {
      // refresh_token is HttpOnly, so we can't read it from JS
      // Don't clear cookies here - just return false and let the server handle it
      console.log('No refresh token available in JS (may be HttpOnly)');
      return false;
    }

    try {
      const response = await fetch(`${this.backendUrl}/api/auth/session/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          refresh_token: this.refreshToken,
        }),
      });

      if (!response.ok) {
        console.warn('Token refresh failed', response.status);
        this.clearTokens();
        return false;
      }

      const result = await response.json();
      if (result.success && result.access_token) {
        this.accessToken = result.access_token;
        if (result.refresh_token) {
          this.refreshToken = result.refresh_token;
        }
        this.tokenExpiry = Date.now() + (result.expires_in || 3600) * 1000;
        this.saveTokensToStorage();
        return true;
      }

      this.clearTokens();
      return false;
    } catch (error) {
      console.error('Token refresh request failed', error);
      // Do NOT clear tokens on network error (might be temporary)
      // Only clear if we know for sure it's invalid (handled above in !response.ok)
      return false;
    }
  }

  private clearTokens(): void {
    this.accessToken = null;
    this.refreshToken = null;
    this.tokenExpiry = null;
    this.user = null;
    this.clearTokensFromStorage();
    this.notifyListeners();
  }

  // ============================================================================
  // API REQUESTS WITH BEARER AUTHENTICATION
  // ============================================================================

  async makeAuthenticatedRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<UnifiedApiResponse<T>> {
    // Ensure we have valid tokens
    if (!this.isAuthenticated()) {
      // Web3-first: No refresh tokens, user must re-authenticate
      return {
        success: false,
        error: {
          code: 401,
          message: 'Authentication required',
          reason: 'No valid authentication tokens',
        },
      };
    }

    // Web3-first: No auto-refresh, token expiry handled by re-authentication

    const url = endpoint.startsWith('http')
      ? endpoint
      : `${this.backendUrl}${endpoint}`;

    const response = await fetch(url, {
      ...options,
      headers: {
        ...options.headers,
        Authorization: `Bearer ${this.accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    // Handle 401 with token refresh retry
    if (response.status === 401) {
      // Attempt to refresh tokens
      const refreshed = await this.refreshTokens();

      if (refreshed) {
        // Retry request with new token
        return this.makeAuthenticatedRequest(endpoint, options);
      }

      // If refresh failed, do NOT clear session immediately if it's a background request (avoid logout loop)
      // Only clear if we really want to enforce logout. Current logic was clearing cookies aggressively.
      console.warn('Authentication failed (401) and refresh failed. Request:', endpoint);

      return {
        success: false,
        error: {
          code: 401,
          message: 'Authentication expired',
          reason: 'Please sign in with your wallet again',
        },
      };
    }

    return this.parseApiResponse(response);
  }

  private async parseApiResponse<T>(
    response: Response
  ): Promise<UnifiedApiResponse<T>> {
    try {
      const data = await response.json();

      // Backend returns unified response format
      if (response.ok) {
        return data; // Already in UnifiedApiResponse format
      } else {
        return data; // Error response in unified format
      }
    } catch (error) {
      return {
        success: false,
        error: {
          code: response.status,
          message: 'Request failed',
          reason: `Failed to parse response: ${error}`,
        },
      };
    }
  }

  // ============================================================================
  // USER MANAGEMENT
  // ============================================================================

  /**
   * Manually set the current user (e.g. from server-side hydration)
   * @param user User info or null
   */
  setCurrentUser(user: UserInfoResponse | null): void {
    this.user = user;

    // If user object contains an access token, hydrate that too
    if (user && user.access) {
      this.accessToken = user.access;
      // If we don't have an expiry yet, assume it's valid for at least a bit (hydration scenario)
      if (!this.tokenExpiry) {
        this.tokenExpiry = Date.now() + 3600 * 1000; // 1 hour safety buffer
      }
    }

    this.notifyListeners();
  }

  async loadCurrentUser(): Promise<UserInfoResponse | null> {
    try {
      if (!this.isAuthenticated()) {
        this.user = null;
        this.notifyListeners();
        return null;
      }

      const user = await this.fetchCurrentUser();
      this.user = user;

      // Sync user to storage
      if (user) {
        setClientCookieJSON(COOKIES.user, user);
      }

      this.notifyListeners();
      return user;
    } catch (error) {
      console.warn('[AUTH] Failed to load current user', error);
      // Don't auto-logout on network error, but do if 401
      if (error instanceof Error && error.message.includes('401')) {
        this.clearTokens();
      }
      return null;
    }
  }

  private async fetchCurrentUser(): Promise<UserInfoResponse | null> {
    // If we already have the user object in memory, return it (optimistic)
    if (this.user) return this.user;

    // Otherwise try to decode from token first (fastest)
    if (this.accessToken) {
      try {
        const payloadPart = this.accessToken.split('.')[1];
        if (payloadPart) {
          const payload = JSON.parse(atob(payloadPart));
          if (payload.sub) {
            return {
              sub: payload.sub,
              wallet_address: payload.sub, // JWT 'sub' is wallet address
              tier_level: payload.tier_level || 'free',
              auth_method: 'web3_siwe',
              permissions: payload.permissions || [],
              access: this.accessToken
            };
          }
        }
      } catch (e) {
        // Ignore decode errors
      }
    }

    // Fallback: This would be where you call /api/auth/me if needed
    // For now we rely on the token payload as the source of truth
    return null;
  }

  async logout(): Promise<void> {
    const wasAuthenticated = !!this.accessToken;
    this.clearTokens();

    if (wasAuthenticated) {
      try {
        // Attempt backend logout (best effort)
        await fetch(`${this.backendUrl}/api/auth/session/logout`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
        });
      } catch (e) {
        // Ignore network errors during logout
      }
    }
  }

  // ============================================================================
  // SIMPLE DISPLAY HELPERS (NOT FOR AUTHORIZATION)
  // ============================================================================

  /**
   * These methods are for UI display only
   * Backend makes all authorization decisions
   */

  getClientId(): string {
    return this.clientId;
  }

  getBackendUrl(): string {
    return this.backendUrl;
  }

  getWalletAddress(): string | null {
    return this.user?.wallet_address || null;
  }

  getUserTier(): string {
    return this.user?.tier_level || 'free';
  }

  getUserPermissions(): string[] {
    return this.user?.permissions || [];
  }


  // ============================================================================
  // OPENID CONNECT DISCOVERY (Compatibility)
  // ============================================================================


  async getDiscoveryDocument(): Promise<any> {
    const url = `${this.backendUrl}/.well-known/openid-configuration`;
    const response = await fetch(url);
    return response.json();
  }

  async getJwks(): Promise<any> {
    const url = `${this.backendUrl}/.well-known/jwks.json`;
    const response = await fetch(url);
    return response.json();
  }
}

// Factory functions for creating client instances
export function createFrontendClient(): SharedWeb3AuthClient {
  return new SharedWeb3AuthClient(
    'epsx-frontend',
    process.env['NEXT_PUBLIC_BACKEND_URL'] || 'http://127.0.0.1:8080'
  );
}

export function createAdminClient(): SharedWeb3AuthClient {
  return new SharedWeb3AuthClient(
    'epsx-admin',
    process.env['NEXT_PUBLIC_BACKEND_URL'] || 'http://127.0.0.1:8080'
  );
}
