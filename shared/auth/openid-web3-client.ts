// ============================================================================
// SHARED OPENID + WEB3 AUTHENTICATION CLIENT
// Unified authentication system for both frontend and admin-frontend
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Same authentication standard for frontend and admin-frontend
 * - Web3 wallet signing triggers OpenID token issuance
 * - No permission logic in frontends - backend decides everything
 * - Bearer token authentication for all API calls
 * - Display exactly what backend tells us to display
 */

// OpenID Connect Token Response (from backend)
export interface OpenIDTokenResponse {
  access_token: string;      // JWT Bearer token for API access
  token_type: string;        // Always "Bearer"
  expires_in: number;        // Seconds until expiration
  refresh_token: string;     // For token renewal
  id_token: string;          // OpenID identity token
  scope: string;             // "openid profile permissions"
}

// Web3 Authentication Request
export interface Web3AuthRequest {
  wallet_address: string;
  signature: string;
  message: string;
  nonce: string;
  client_id: string;        // "epsx-frontend" or "epsx-admin"
}

// Web3 Challenge Response
export interface Web3ChallengeResponse {
  nonce: string;
  message: string;
  wallet_address: string;
}

// User Info Response (from /api/v1/auth/userinfo)
export interface UserInfoResponse {
  sub: string;                    // Wallet address
  wallet_address: string;         // Web3 wallet address
  tier_level: string;            // User tier
  auth_method: string;           // "web3_siwe"
  permissions: string[];         // Backend-determined permissions (display only)
  email?: string;                // Optional email (for compatibility)
  packageTier?: string;          // Alias for tier_level (for compatibility)
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
 * Shared OpenID + Web3 Authentication Client
 * Used by both frontend and admin-frontend applications
 */
export class SharedOpenIDWeb3Client {
  private accessToken: string | null = null;
  private refreshToken: string | null = null;
  private tokenExpiry: number | null = null;
  private user: UserInfoResponse | null = null;
  private listeners: Set<(user: UserInfoResponse | null) => void> = new Set();
  private clientId: string;
  private backendUrl: string;

  constructor(clientId: string, backendUrl: string) {
    this.clientId = clientId;
    this.backendUrl = backendUrl;
    this.loadTokensFromStorage();
  }

  // ============================================================================
  // AUTHENTICATION STATE MANAGEMENT
  // ============================================================================

  isAuthenticated(): boolean {
    return !!(this.accessToken && this.tokenExpiry && this.tokenExpiry > Date.now());
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
  // TOKEN STORAGE (localStorage)
  // ============================================================================

  private loadTokensFromStorage(): void {
    if (typeof window === 'undefined') return;
    
    try {
      this.accessToken = localStorage.getItem(`${this.clientId}_access_token`);
      this.refreshToken = localStorage.getItem(`${this.clientId}_refresh_token`);
      const expiry = localStorage.getItem(`${this.clientId}_token_expiry`);
      this.tokenExpiry = expiry ? parseInt(expiry, 10) : null;
    } catch (error) {
      console.warn('Failed to load tokens from storage', { error });
    }
  }

  private saveTokensToStorage(): void {
    if (typeof window === 'undefined') return;
    
    try {
      if (this.accessToken) {
        localStorage.setItem(`${this.clientId}_access_token`, this.accessToken);
      }
      if (this.refreshToken) {
        localStorage.setItem(`${this.clientId}_refresh_token`, this.refreshToken);
      }
      if (this.tokenExpiry) {
        localStorage.setItem(`${this.clientId}_token_expiry`, this.tokenExpiry.toString());
      }
    } catch (error) {
      console.warn('Failed to save tokens to storage', { error });
    }
  }

  private clearTokensFromStorage(): void {
    if (typeof window === 'undefined') return;
    
    try {
      localStorage.removeItem(`${this.clientId}_access_token`);
      localStorage.removeItem(`${this.clientId}_refresh_token`);
      localStorage.removeItem(`${this.clientId}_token_expiry`);
    } catch (error) {
      console.warn('Failed to clear tokens from storage', { error });
    }
  }

  // ============================================================================
  // WEB3 AUTHENTICATION FLOW
  // ============================================================================

  async requestChallenge(walletAddress: string): Promise<Web3ChallengeResponse> {
    const response = await fetch(`${this.backendUrl}/api/auth/web3/challenge`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        wallet_address: walletAddress,
        client_id: this.clientId
      }),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(`Challenge request failed: ${errorData.message || 'Unknown error'}`);
    }

    return response.json();
  }

  async authenticateWithSignature(request: {
    wallet_address: string;
    signature: string;
    message: string;
    nonce: string;
  }): Promise<{ success: boolean; user?: UserInfoResponse; error?: string }> {
    try {
      // Get OpenID tokens from backend
      const tokenResponse = await this.getOpenIDTokens({
        ...request,
        client_id: this.clientId
      });

      // Store tokens
      this.accessToken = tokenResponse.access_token;
      this.refreshToken = tokenResponse.refresh_token;
      this.tokenExpiry = Date.now() + (tokenResponse.expires_in * 1000);
      this.saveTokensToStorage();

      // Get user information
      const user = await this.fetchCurrentUser();
      
      if (!user) {
        throw new Error('Failed to fetch user information');
      }

      this.user = user;
      this.notifyListeners();

      return { success: true, user };

    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Authentication failed'
      };
    }
  }

  private async getOpenIDTokens(request: Web3AuthRequest): Promise<OpenIDTokenResponse> {
    const response = await fetch(`${this.backendUrl}/api/v1/auth/web3/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(`Token request failed: ${errorData.error_description || 'Unknown error'}`);
    }

    return response.json();
  }

  // ============================================================================
  // TOKEN MANAGEMENT
  // ============================================================================

  private needsRefresh(): boolean {
    if (!this.tokenExpiry) return false;
    const fiveMinutesFromNow = Date.now() + (5 * 60 * 1000);
    return this.tokenExpiry <= fiveMinutesFromNow;
  }

  private async refreshTokens(): Promise<boolean> {
    if (!this.refreshToken) return false;

    try {
      const response = await fetch(`${this.backendUrl}/api/v1/auth/token/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          refresh_token: this.refreshToken,
          client_id: this.clientId,
        }),
      });

      if (!response.ok) {
        this.clearTokens();
        return false;
      }

      const tokenResponse: OpenIDTokenResponse = await response.json();
      
      this.accessToken = tokenResponse.access_token;
      this.refreshToken = tokenResponse.refresh_token;
      this.tokenExpiry = Date.now() + (tokenResponse.expires_in * 1000);
      this.saveTokensToStorage();

      return true;

    } catch (error) {
      this.clearTokens();
      return false;
    }
  }

  private clearTokens(): void {
    this.accessToken = null;
    this.refreshToken = null;
    this.tokenExpiry = null;
    this.clearTokensFromStorage();
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
      if (this.refreshToken && await this.refreshTokens()) {
        // Token refreshed successfully, continue
      } else {
        return {
          success: false,
          error: {
            code: 401,
            message: 'Authentication required',
            reason: 'No valid authentication tokens'
          }
        };
      }
    }

    // Refresh token if it expires soon
    if (this.needsRefresh() && this.refreshToken) {
      await this.refreshTokens();
    }

    const url = endpoint.startsWith('http') 
      ? endpoint 
      : `${this.backendUrl}${endpoint}`;

    const response = await fetch(url, {
      ...options,
      headers: {
        ...options.headers,
        'Authorization': `Bearer ${this.accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    // Handle 401 with token refresh retry
    if (response.status === 401) {
      if (this.refreshToken && await this.refreshTokens()) {
        // Retry with new token
        const retryResponse = await fetch(url, {
          ...options,
          headers: {
            ...options.headers,
            'Authorization': `Bearer ${this.accessToken}`,
            'Content-Type': 'application/json',
          },
        });

        return this.parseApiResponse(retryResponse);
      } else {
        this.clearTokens();
        this.user = null;
        this.notifyListeners();
        
        return {
          success: false,
          error: {
            code: 401,
            message: 'Authentication expired',
            reason: 'Please sign in again'
          }
        };
      }
    }

    return this.parseApiResponse(response);
  }

  private async parseApiResponse<T>(response: Response): Promise<UnifiedApiResponse<T>> {
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
          reason: `Failed to parse response: ${error}`
        }
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
    const response = await this.makeAuthenticatedRequest<UserInfoResponse>('/api/v1/auth/userinfo');
    return response.success ? response.data || null : null;
  }

  async logout(): Promise<void> {
    try {
      // Revoke tokens on backend
      if (this.refreshToken) {
        await fetch(`${this.backendUrl}/api/v1/auth/token/revoke`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            refresh_token: this.refreshToken,
            client_id: this.clientId,
          }),
        });
      }
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
export function createFrontendClient(): SharedOpenIDWeb3Client {
  return new SharedOpenIDWeb3Client(
    'epsx-frontend',
    process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
  );
}

export function createAdminClient(): SharedOpenIDWeb3Client {
  return new SharedOpenIDWeb3Client(
    'epsx-admin',
    process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
  );
}