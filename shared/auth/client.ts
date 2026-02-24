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

import { logger } from '../utils/logger';

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
export interface UnifiedApiResponse<T = unknown> {
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

export interface AuthVerifyResult {
  success: boolean;
  authenticated: boolean;
  message?: string;
  error?: string;
  access_token: string;
  refresh_token?: string;
  expires_in?: number;
  wallet_address: string;
  tier_level?: string;
  permissions?: string[];
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
  private authInProgress = false;

  constructor(clientId: string, backendUrl: string) {
    this.clientId = clientId;
    this.backendUrl = backendUrl;
  }

  // ============================================================================
  // AUTHENTICATION STATE MANAGEMENT
  // ============================================================================

  isAuthenticated(): boolean {
    return Boolean(this.accessToken !== null && this.accessToken !== '' &&
      this.tokenExpiry !== null &&
      this.tokenExpiry > Date.now());
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

  private decodeUserFromToken(token: string): UserInfoResponse | null {
    try {
      const payloadPart = token.split('.')[1];
      if (!payloadPart) { return null; }

      const payload = JSON.parse(atob(payloadPart)) as Record<string, unknown>;
      if (typeof payload.sub === 'string' && payload.sub.startsWith('0x')) {
        logger.info('[AUTH] Decoded user identity from Access Token');
        return {
          sub: payload.sub,
          wallet_address: payload.sub,
          tier_level: (payload.package_tier as string | undefined) ?? (payload.tier_level as string | undefined) ?? 'basic',
          auth_method: 'web3_siwe',
          permissions: Array.isArray(payload.permissions) ? (payload.permissions as string[]) : [],
          access: token
        };
      }
    } catch (e) {
      logger.warn('Failed to decode access token for user recovery', {
        error_message: e instanceof Error ? e.message : String(e),
        error_type: e instanceof Error ? e.constructor.name : typeof e
      });
    }
    return null;
  }

  // ============================================================================
  // WEB3 AUTHENTICATION FLOW
  // ============================================================================

  async requestChallenge(
    walletAddress: string,
    turnstileToken?: string,
  ): Promise<Web3ChallengeResponse> {
    // Deduplicate concurrent challenge requests
    // Normalize wallet address for cache key
    const normalizedAddress = walletAddress.trim().toLowerCase();
    const cacheKey = `${normalizedAddress}_${this.clientId}`;
    const now = Date.now();
    const cached = this.challengeCache.get(cacheKey);

    // Return cached promise if request is in-flight or recent (within 60s)
    if (cached !== undefined && (now - cached.timestamp) < 60000) {
      logger.info('[AUTH] Reusing existing challenge request', { wallet_address: normalizedAddress });
      return cached.promise;
    }

    const challengeUrl = `${this.backendUrl}/api/auth/web3/challenge`;

    logger.info('[AUTH] Requesting Web3 challenge', {
      url: challengeUrl,
      wallet_address: walletAddress,
      backend_url: this.backendUrl,
      has_turnstile: Boolean(turnstileToken),
    });

    const challengePromise = (async () => {
      try {
        const body: Record<string, string> = {
          wallet_address: walletAddress,
        };
        if (turnstileToken !== undefined && turnstileToken !== '') {
          body.turnstile_token = turnstileToken;
        }

        const response = await fetch(challengeUrl, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Accept: 'application/json',
          },
          body: JSON.stringify(body),
          signal: AbortSignal.timeout(10000), // 10s timeout
        });

        logger.info('[AUTH] Challenge response received', {
          status: response.status,
          ok: response.ok,
          statusText: response.statusText,
          headers: Object.fromEntries(response.headers.entries()),
        });

        if (!response.ok) {
          await this.handleChallengeError(response, { url: challengeUrl, walletAddress });
        }

        const challengeData = (await response.json()) as Web3ChallengeResponse;
        logger.info('[AUTH] Challenge request successful', challengeData);

        return challengeData;
      } catch (error) {
        // Clear cache on error
        this.challengeCache.delete(cacheKey);

        let errorMessage: string;

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
        logger.error('[AUTH] Error: Challenge request', {
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

  // eslint-disable-next-line complexity, sonarjs/cognitive-complexity
  private async handleChallengeError(
    response: Response,
    context: { url: string; walletAddress: string; initialErrorData?: unknown }
  ): Promise<never> {
    const { url, walletAddress, initialErrorData } = context;
    let errorData: unknown = initialErrorData ?? null;
    let errorMessage = `Challenge request failed: ${response.status} ${response.statusText}`;

    // Read the response body only once if not provided
    if (errorData === null) {
      const contentType = response.headers.get('content-type');
      try {
        if (response.status === 404) {
          errorMessage =
            'Authentication endpoint not found. The backend may need to be updated with Web3 authentication routes.';
        } else if (contentType?.includes('application/json') === true) {
          errorData = (await response.json()) as unknown;
          const message = (errorData as { message?: string }).message;
          errorMessage = (message !== undefined && message !== '') ? message : errorMessage;
        } else {
          const errorText = await response.text();
          errorMessage = `Challenge request failed: ${response.status} ${response.statusText}. ${errorText}`;
          errorData = { text: errorText };
        }
      } catch (bodyReadError) {
        logger.warn('Failed to read error response body:', {
          error_message: bodyReadError instanceof Error ? bodyReadError.message : String(bodyReadError),
          error_type: bodyReadError instanceof Error ? bodyReadError.constructor.name : typeof bodyReadError
        });
        if (response.status === 404) {
          errorMessage =
            'Authentication endpoint not found. The backend may need to be updated with Web3 authentication routes.';
        }
      }
    }

    const errorDetails = {
      url,
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

    if (response.status === 404) {
      logger.error(
        '[AUTH] Error: Web3 challenge endpoint not found (404)',
        errorDetails
      );
    } else {
      logger.error(
        '[AUTH] Error: Challenge request failed with full details:',
        errorDetails
      );
    }

    throw new Error(errorMessage);
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
    if (this.authInProgress === true) {
      return { success: false, error: 'Authentication already in progress. Please wait.' };
    }

    this.authInProgress = true;

    try {
      logger.info('Sending Web3 verify request', {
        url: `${this.backendUrl}/api/auth/web3/verify`,
        wallet_address: request.wallet_address,
        has_signature: Boolean(request.signature),
        has_message: Boolean(request.message),
        has_nonce: Boolean(request.nonce)
      });

      const response = await fetch(`${this.backendUrl}/api/auth/web3/verify`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request),
        signal: AbortSignal.timeout(15000),
      });

      if (!response.ok) {
        await this.handleAuthResponseError(response, request.wallet_address);
      }

      const result = (await response.json()) as AuthVerifyResult;

      // Clear challenge cache after any verification attempt (success or failure)
      // because the backend handles nonce lifecycle
      const normalizedAddress = request.wallet_address.trim().toLowerCase();
      const cacheKey = `${normalizedAddress}_${this.clientId}`;
      this.challengeCache.delete(cacheKey);

      this.validateAuthResult(result, request.wallet_address);
      this.updateStateFromAuthResult(result);

      return { success: true, user: this.user ?? undefined };
    } catch (error) {
      const errorMessage = this.formatAuthErrorMessage(error, request.wallet_address);
      return { success: false, error: errorMessage };
    } finally {
      this.authInProgress = false;
    }
  }

  private async handleAuthResponseError(response: Response, walletAddress: string): Promise<never> {
    let errorMessage: string;
    let errorData: Record<string, unknown> | null = null;

    try {
      const parsedData = (await response.json()) as Record<string, unknown>;
      errorData = parsedData;
      errorMessage = (errorData.message as string | undefined) ?? (errorData.error as string | undefined) ?? `Verification failed: ${response.status} ${response.statusText}`;
    } catch (_e) {
      errorMessage = `Verification failed: ${response.status} ${response.statusText}`;
    }

    const errorDetails = {
      status: response.status,
      status_text: response.statusText,
      url: `${this.backendUrl}/api/auth/web3/verify`,
      wallet_address: walletAddress,
      error_message: errorMessage,
      error_data: errorData ?? 'No response data',
      error_data_json: errorData !== null ? JSON.stringify(errorData) : 'null'
    };

    logger.error(
      `Web3 authentication request failed - Status: ${errorDetails.status} (${errorDetails.status_text}), URL: ${errorDetails.url}, Wallet: ${errorDetails.wallet_address}, Message: ${errorDetails.error_message}, Response: ${errorDetails.error_data_json}`,
      errorDetails
    );

    throw new Error(errorMessage);
  }

  private validateAuthResult(result: AuthVerifyResult, walletAddress: string): void {
    if (result.success === false || result.authenticated === false) {
      const errorMsg = result.message ?? result.error ?? 'Authentication failed';
      logger.error(
        `Web3 authentication failed in backend - Success: ${String(result.success)}, Authenticated: ${String(result.authenticated)}, Wallet: ${walletAddress}, Message: ${errorMsg}`,
        {
          success: result.success,
          authenticated: result.authenticated,
          wallet_address: walletAddress,
          error_message: errorMsg,
          backend_url: this.backendUrl
        }
      );
      throw new Error(errorMsg);
    }
  }

  private updateStateFromAuthResult(result: AuthVerifyResult): void {
    this.accessToken = result.access_token;
    this.refreshToken = result.refresh_token ?? null;
    this.tokenExpiry = Date.now() + (result.expires_in ?? 3600) * 1000;

    this.user = {
      sub: result.wallet_address,
      wallet_address: result.wallet_address,
      tier_level: result.tier_level ?? 'free',
      auth_method: 'web3_siwe',
      permissions: result.permissions ?? [],
      access: result.access_token,
    };

    this.notifyListeners();
  }

  private formatAuthErrorMessage(error: unknown, walletAddress: string): string {
    if (error instanceof TypeError && error.message.includes('fetch')) {
      logger.error(
        `Web3 authentication network error - Backend: ${this.backendUrl}, Wallet: ${walletAddress}`,
        {
          wallet_address: walletAddress,
          backend_url: this.backendUrl,
          error_type: 'NetworkError'
        }
      );
      return `Network error: Cannot connect to backend at ${this.backendUrl}.`;
    }

    const message = error instanceof Error ? error.message : String(error);
    const errorType = error instanceof Error ? error.constructor.name : typeof error;
    logger.error(
      `Web3 authentication error - Type: ${errorType}, Message: ${message}, Wallet: ${walletAddress}, Backend: ${this.backendUrl}`,
      {
        error_message: message,
        wallet_address: walletAddress,
        error_type: errorType,
        backend_url: this.backendUrl
      }
    );
    return message;
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
      const errorData = (await response.json()) as { error_description?: string; message?: string };
      throw new Error(
        `Web3 verification failed: ${errorData.error_description ?? errorData.message ?? 'Unknown error'}`
      );
    }

    return response.json() as Promise<Web3TokenResponse>;
  }

  // ============================================================================
  // TOKEN MANAGEMENT
  // ============================================================================

  private isExpired(): boolean {
    if (this.tokenExpiry === null) { return true; }
    return this.tokenExpiry <= Date.now();
  }

  // Refresh tokens using backend refresh endpoint
  // eslint-disable-next-line complexity
  async refreshTokens(): Promise<boolean> {
    if (this.refreshToken === null || this.refreshToken === '') {
      // refresh_token is HttpOnly, so we can't read it from JS
      // Don't clear cookies here - just return false and let the server handle it
      logger.info('No refresh token available in JS (may be HttpOnly)');
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
        logger.warn('Token refresh failed', response.status);
        this.clearTokens();
        return false;
      }

      const result = (await response.json()) as {
        success: boolean;
        access_token?: string;
        refresh_token?: string;
        expires_in?: number;
      };

      if (result.success && result.access_token !== undefined && result.access_token !== '') {
        this.accessToken = result.access_token;
        if (result.refresh_token !== undefined && result.refresh_token !== '') {
          this.refreshToken = result.refresh_token;
        }
        this.tokenExpiry = Date.now() + (result.expires_in ?? 3600) * 1000;
        this.notifyListeners();
        return true;
      }

      this.clearTokens();
      return false;
    } catch (error) {
      logger.error('Token refresh request failed', {
        error_message: error instanceof Error ? error.message : String(error),
        error_type: error instanceof Error ? error.constructor.name : typeof error,
        backend_url: this.backendUrl
      });
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
    this.challengeCache.clear();
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
      logger.warn('Authentication failed (401) and refresh failed. Request:', endpoint);

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
      const data = await response.json() as unknown;

      // Backend returns unified response format
      return data as UnifiedApiResponse<T>;
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
    if (user?.access !== undefined && user.access !== '') {
      this.accessToken = user.access;
      // If we don't have an expiry yet, assume it's valid for at least a bit (hydration scenario)
      this.tokenExpiry ??= Date.now() + 3600 * 1000; // 1 hour safety buffer
    }

    this.notifyListeners();
  }

  /**
   * Update tokens manually (e.g. after a server action refreshed them)
   */
  updateTokens(accessToken: string, expiresIn?: number): void {
    this.accessToken = accessToken;
    this.tokenExpiry = Date.now() + (expiresIn ?? 3600) * 1000;
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
      this.notifyListeners();
      return user;
    } catch (error) {
      logger.warn('[AUTH] Failed to load current user', {
        error_message: error instanceof Error ? error.message : String(error),
        error_type: error instanceof Error ? error.constructor.name : typeof error,
        backend_url: this.backendUrl
      });
      // Don't auto-logout on network error, but do if 401
      if (error instanceof Error && error.message.includes('401')) {
        this.clearTokens();
      }
      return null;
    }
  }

  async fetchCurrentUser(): Promise<UserInfoResponse | null> {
    if (this.user !== null) { return this.user; }
    if (this.accessToken !== null && this.accessToken !== '') {
      return this.decodeUserFromToken(this.accessToken);
    }
    return null;
  }

  logout(): void {
    this.clearTokens();
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
    return this.user?.wallet_address ?? null;
  }

  getUserTier(): string {
    return this.user?.tier_level ?? 'free';
  }

  getUserPermissions(): string[] {
    return this.user?.permissions ?? [];
  }

  // ============================================================================
  // OPENID CONNECT DISCOVERY (Compatibility)
  // ============================================================================

  async getDiscoveryDocument(): Promise<Record<string, unknown>> {
    const url = `${this.backendUrl}/.well-known/openid-configuration`;
    const response = await fetch(url);
    return (await response.json()) as Record<string, unknown>;
  }

  async getJwks(): Promise<Record<string, unknown>> {
    const url = `${this.backendUrl}/.well-known/jwks.json`;
    const response = await fetch(url);
    return (await response.json()) as Record<string, unknown>;
  }
}

// Factory functions for creating client instances
export function createFrontendClient(): SharedWeb3AuthClient {
  return new SharedWeb3AuthClient(
    'epsx-frontend',
    process.env['NEXT_PUBLIC_BACKEND_URL'] ?? 'http://127.0.0.1:8080'
  );
}

export function createAdminClient(): SharedWeb3AuthClient {
  return new SharedWeb3AuthClient(
    'epsx-admin',
    process.env['NEXT_PUBLIC_BACKEND_URL'] ?? 'http://127.0.0.1:8080'
  );
}
