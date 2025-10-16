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

// User Info Response (from /api/v1/auth/userinfo)
export interface UserInfoResponse {
  sub: string; // Wallet address
  wallet_address: string; // Web3 wallet address
  tier_level: string; // User tier
  auth_method: string; // "web3_siwe"
  permissions: string[]; // Backend-determined permissions (display only)
  email?: string; // Optional email (for compatibility)
  access?: string; // JWT access token for SSE authentication
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
  // TOKEN STORAGE (cookies)
  // ============================================================================

  private loadTokensFromStorage(): void {
    if (typeof window === 'undefined') return;

    try {
      // Access token is HttpOnly, so we can't access it directly
      // We'll check client-side cookies for expiry and user data
      const expiry = getClientCookie(COOKIES.expires_at);
      this.tokenExpiry = expiry ? parseInt(expiry, 10) : null;

      // Restore user object from cookies
      const storedUser = getClientCookieJSON<UserInfoResponse>(COOKIES.user);
      if (storedUser) {
        this.user = storedUser;
      }
    } catch (error) {
      console.warn('Failed to load tokens from cookies', { error });
    }
  }

  private saveTokensToStorage(): void {
    if (typeof window === 'undefined') return;

    try {
      // Access token is set by server as HttpOnly cookie
      // Save expiry and user data to client-side cookies
      if (this.tokenExpiry) {
        setClientCookie(COOKIES.expires_at, this.tokenExpiry.toString());
      }

      // Save user object to cookies
      if (this.user) {
        setClientCookieJSON(COOKIES.user, this.user);
        setClientCookie(COOKIES.auth_time, Date.now().toString());
      }
    } catch (error) {
      console.warn('Failed to save tokens to cookies', { error });
    }
  }

  private clearTokensFromStorage(): void {
    if (typeof window === 'undefined') return;

    try {
      // Clear client-side cookies
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
      console.log('🔄 Reusing existing challenge request');
      return cached.promise;
    }

    const challengeUrl = `${this.backendUrl}/api/auth/web3/challenge`;

    console.log('🔑 Requesting Web3 challenge', {
      url: challengeUrl,
      wallet_address: walletAddress,
      client_id: this.clientId,
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
            client_id: this.clientId,
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

        try {
          const errorData = await response.json();
          errorMessage = `Challenge request failed: ${errorData.message || errorMessage}`;
        } catch {
          const errorText = await response.text();
          errorMessage = `Challenge request failed: ${response.status} ${response.statusText}. ${errorText}`;
        }

        // Enhanced error logging with troubleshooting hints
        console.error('❌ Challenge request failed', {
          url: challengeUrl,
          status: response.status,
          statusText: response.statusText,
          troubleshooting: this.getTroubleshootingHints(response.status),
        });

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

        if (error instanceof TypeError && error.message.includes('fetch')) {
          const enhancedError = new Error(
            `Network error: Cannot connect to backend at ${challengeUrl}. ` +
              `Please ensure the backend server is running on port 8080. ` +
              `Original error: ${error.message}`
          );
          console.error('🌐 Network connectivity error', {
            url: challengeUrl,
            backend_url: this.backendUrl,
            error: error.message,
            hint: 'Backend server may not be running or CORS may be misconfigured',
          });
          throw enhancedError;
        }
        throw error;
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
        const errorData = await response.json();
        throw new Error(errorData.message || 'Verification failed');
      }

      const result = await response.json();

      // Check if authentication was successful
      if (!result.success || !result.authenticated) {
        throw new Error(result.message || 'Authentication failed');
      }

      // Store access token
      this.accessToken = result.access_token;
      // Default to 24 hour expiry if not provided
      this.tokenExpiry = Date.now() + 24 * 60 * 60 * 1000;

      // Create user object from response
      const user: UserInfoResponse = {
        sub: result.wallet_address,
        wallet_address: result.wallet_address,
        tier_level: 'standard', // TODO: Derive from permissions
        auth_method: 'web3_siwe',
        permissions: result.permissions || [],
        access: result.access_token,
      };

      this.user = user;
      this.saveTokensToStorage();
      this.notifyListeners();

      return { success: true, user };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Authentication failed',
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

  // Web3-first system: No refresh tokens, users re-authenticate with wallet
  private async refreshTokens(): Promise<boolean> {
    console.log(
      'Web3 tokens expired, user needs to re-authenticate with wallet'
    );
    this.clearTokens();
    return false;
  }

  private clearTokens(): void {
    this.accessToken = null;
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
      // Web3-first: No refresh tokens, clear session and require re-authentication
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
    }>('/api/auth/session/verify', {
      method: 'POST',
      body: JSON.stringify({ admin_context: false }),
    });

    if (response.success && response.data?.authenticated) {
      // Convert session verification response to UserInfoResponse format
      return {
        sub: response.data.wallet_address,
        wallet_address: response.data.wallet_address,
        tier_level: 'standard', // TODO: Derive from permissions
        auth_method: 'web3_siwe',
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

  getUserTier(): string {
    return this.user?.tier_level || 'free';
  }

  getUserPermissions(): string[] {
    return this.user?.permissions || [];
  }

  // Simple display helper - NOT for authorization
  hasPermissionForDisplay(permission: string): boolean {
    return this.user?.permissions.includes(permission) || false;
  }
}

// Factory functions for creating client instances
export function createFrontendClient(): SharedWeb3AuthClient {
  return new SharedWeb3AuthClient(
    'epsx-frontend',
    process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
  );
}

export function createAdminClient(): SharedWeb3AuthClient {
  return new SharedWeb3AuthClient(
    'epsx-admin',
    process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
  );
}
