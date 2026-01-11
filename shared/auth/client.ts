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
  setClientCookie,
  setClientCookieJSON,
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
  // TOKEN STORAGE (cookies + localStorage fallback)
  // ============================================================================

  private loadTokensFromStorage(): void {
    if (typeof window === 'undefined') return;

    try {
      // 1. Try cookies first (primary storage)
      this.cleanupLegacyStorage();

      // Access token is HttpOnly usually, but we sync it to session_id for Server Actions & Refresh
      let accessToken = getClientCookie(COOKIES.session_id);

      // 2. Fallback to localStorage (epsx.access_token)
      // This handles cases where cookies fail (e.g. size limits, browser restrictions)
      if (!accessToken) {
        accessToken = localStorage.getItem('epsx.access_token');
        if (accessToken) {
          console.log('🔄 Recovered access token from localStorage fallback');
        }
      }

      if (accessToken) {
        this.accessToken = accessToken;
      }

      this.refreshToken = getClientCookie(COOKIES.refresh_token) || localStorage.getItem('epsx.refresh_token');

      // Access token is HttpOnly, so we can't access it directly
      // We'll check client-side cookies for expiry and user data
      const expiry = getClientCookie(COOKIES.expires_at) || localStorage.getItem('epsx.expires_at');
      this.tokenExpiry = expiry ? parseInt(expiry, 10) : null;

      // Restore user object from cookies or localStorage
      let storedUser = getClientCookieJSON<UserInfoResponse>(COOKIES.user);
      if (!storedUser) {
        const storedUserStr = localStorage.getItem('epsx.user');
        if (storedUserStr) {
          try {
            storedUser = JSON.parse(storedUserStr);
          } catch (e) {
            // Invalid JSON
          }
        }
      }

      if (storedUser) {
        this.user = storedUser;
      }
    } catch (error) {
      console.warn('Failed to load tokens from storage', { error });
    }
  }

  private saveTokensToStorage(): void {
    if (typeof window === 'undefined') return;

    try {
      console.log('🍪 saveTokensToStorage called:', {
        hasTokenExpiry: !!this.tokenExpiry,
        hasAccessToken: !!this.accessToken,
        accessTokenLength: this.accessToken?.length || 0,
        hasRefreshToken: !!this.refreshToken,
        hasUser: !!this.user,
        cookieClientSession: COOKIES.session_id,
      });

      // Access token is set by server as HttpOnly cookie
      // Save expiry and user data to client-side cookies AND localStorage

      if (this.tokenExpiry) {
        setClientCookie(COOKIES.expires_at, this.tokenExpiry.toString());
        localStorage.setItem('epsx.expires_at', this.tokenExpiry.toString());
        console.log('🍪 Set expires_at cookie and localStorage');
      }

      // Sync access token to session_id for Server Components and persistence
      if (this.accessToken) {
        console.log('🍪 Setting session_id cookie with token of length:', this.accessToken.length);
        setClientCookie(COOKIES.session_id, this.accessToken);
        localStorage.setItem('epsx.access_token', this.accessToken);
        console.log('🍪 session_id cookie and localStorage SET');
      } else {
        console.warn('⚠️ No access token to save to session_id!');
      }

      if (this.refreshToken) {
        setClientCookie(COOKIES.refresh_token, this.refreshToken);
        localStorage.setItem('epsx.refresh_token', this.refreshToken);
      }

      // Save user object to cookies and localStorage
      if (this.user) {
        setClientCookieJSON(COOKIES.user, this.user);
        localStorage.setItem('epsx.user', JSON.stringify(this.user));

        const now = Date.now().toString();
        setClientCookie(COOKIES.auth_time, now);
        localStorage.setItem('epsx.auth_time', now);

        console.log('🍪 Set user and auth_time cookies and localStorage');
      }
    } catch (error) {
      console.warn('Failed to save tokens to storage', { error });
    }
  }

  private clearTokensFromStorage(): void {
    if (typeof window === 'undefined') return;

    try {
      // Clear client-side cookies
      clearClientSideCookies();
      // Explicitly clear session_id
      setClientCookie(COOKIES.session_id, '', 0);

      // Clear localStorage
      localStorage.removeItem('epsx.access_token');
      localStorage.removeItem('epsx.refresh_token');
      localStorage.removeItem('epsx.expires_at');
      localStorage.removeItem('epsx.user');
      localStorage.removeItem('epsx.auth_time');
    } catch (error) {
      console.warn('Failed to clear tokens from cookies', { error });
    }
  }

  private cleanupLegacyStorage(): void {
    if (typeof window === 'undefined') return;
    try {
      const legacyKeys = [
        'oidc.access_token', 'oidc.refresh_token', 'oidc.expires_at',
        'oidc.user', 'oidc.auth_time', 'oidc.id_token'
      ];
      let cleaned = false;
      legacyKeys.forEach(key => {
        if (localStorage.getItem(key)) {
          localStorage.removeItem(key);
          cleaned = true;
        }
      });
      if (cleaned) {
        console.log('🧹 Cleaned up legacy OIDC storage keys');
      }
    } catch (e) {
      // Ignore errors during cleanup
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
      console.log('🔄 Reusing existing challenge request');
      return cached.promise;
    }

    const challengeUrl = `${this.backendUrl}/api/auth/web3/challenge`;

    console.log('🔑 Requesting Web3 challenge', {
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
        });

        console.log('🔑 Challenge response received', {
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

            console.error('❌ Web3 challenge endpoint not found (404)', errorDetails);
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

            console.error('❌ Challenge request failed with full details:', errorDetails);
          }

          throw new Error(errorMessage);
        }

        const challengeData = await response.json();
        console.log('✅ Challenge request successful', challengeData);

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
          errorMessage = error.message;
        } else {
          errorMessage = String(error);
        }

        const enhancedError = new Error(errorMessage);
        console.error('🌐 Challenge request error', {
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
          console.error('❌ Web3 authentication endpoint not found (404)', {
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
      console.log('🔐 Auth result from backend:', {
        hasAccessToken: !!result.access_token,
        accessTokenLength: result.access_token?.length || 0,
        accessTokenPreview: result.access_token?.substring(0, 50) + '...',
      });
      this.accessToken = result.access_token;
      this.refreshToken = result.refresh_token;
      // Default to 30 days expiry if not provided
      this.tokenExpiry = Date.now() + 30 * 24 * 60 * 60 * 1000;

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
      console.log('🍪 Saving tokens to storage, accessToken is:', this.accessToken ? 'SET' : 'EMPTY');
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
        errorMessage = error.message;
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
      console.log('No refresh token available, user needs to re-authenticate');
      this.clearTokens();
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
      this.clearTokens();
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

      // If refresh failed, clear session and require re-authentication
      this.clearTokens();

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

  async loadCurrentUser(): Promise<UserInfoResponse | null> {
    try {
      if (!this.isAuthenticated()) {
        this.user = null;
        this.notifyListeners();
        return null;
      }

      const user = await this.fetchCurrentUser();
      this.user = user;
      this.notifyListeners();
      return user;
    } catch (error) {
      this.user = null;
      this.notifyListeners();
      return null;
    }
  }

  private async fetchCurrentUser(): Promise<UserInfoResponse | null> {
    // Use session verification endpoint to get current user info
    const response = await this.makeAuthenticatedRequest<{
      success: boolean;
      authenticated: boolean;
      wallet_address: string;
      user_id: string;
      permissions: string[];
      is_admin: boolean;
      expires: string;
    }>('/api/auth/web3/session', {
      method: 'POST',
      body: JSON.stringify({ admin_context: false }),
    });

    if (response.success && response.data?.authenticated) {
      // Convert session verification response to UserInfoResponse format
      return {
        sub: response.data.wallet_address,
        wallet_address: response.data.wallet_address,
        auth_method: 'web3_siwe',
        tier_level: 'basic', // Default tier level
        permissions: response.data.permissions || [],
      };
    }

    return null;
  }

  async logout(): Promise<void> {
    try {
      // Web3-first: No server-side token revocation needed
      console.log('🚪 Web3 logout - clearing local session only');
    } catch (error) {
      // Continue with local cleanup even if server revocation fails
    }

    // Clear local state
    this.clearTokens();
    this.user = null;
    this.notifyListeners();

    // Trigger wallet disconnect event
    if (typeof window !== 'undefined') {
      const event = new CustomEvent('epsx:disconnect-wallet');
      window.dispatchEvent(event);
    }
  }

  // ============================================================================
  // SIMPLE DISPLAY HELPERS (NOT FOR AUTHORIZATION)
  // ============================================================================

  /**
   * These methods are for UI display only
   * Backend makes all authorization decisions
   */

  getWalletAddress(): string | null {
    return this.user?.wallet_address || null;
  }

  getUserPermissions(): string[] {
    return this.user?.permissions || [];
  }

  // Simple display helper - NOT for authorization
  hasPermissionForDisplay(permission: string): boolean {
    return this.user?.permissions.includes(permission) || false;
  }

  // Public getters for debugging
  getBackendUrl(): string {
    return this.backendUrl;
  }

  getClientId(): string {
    return this.clientId;
  }

  getUserTier(): string {
    return this.user?.tier_level || 'free';
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
    process.env['NEXT_PUBLIC_BACKEND_URL'] || 'http://localhost:8080'
  );
}

export function createAdminClient(): SharedWeb3AuthClient {
  return new SharedWeb3AuthClient(
    'epsx-admin',
    process.env['NEXT_PUBLIC_BACKEND_URL'] || 'http://localhost:8080'
  );
}
