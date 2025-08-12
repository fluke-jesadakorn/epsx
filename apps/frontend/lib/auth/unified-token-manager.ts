// Unified Token Manager
// Manages authentication tokens across multiple providers with automatic refresh and validation

import { getFirebaseAuth, getFirebaseTokenValidator } from '@epsx/firebase-analytics';
import ProviderDetector, { type AuthProvider } from './provider-detector';

/**
 * Unified JWT token structure
 */
export interface UnifiedToken {
  access_token: string;
  token_type: string;
  expires_at: string;
  expires_in: number;
  session_id: string;
  jti: string;
  refresh_token?: string;
}

/**
 * Token validation result
 */
export interface TokenValidationResult {
  isValid: boolean;
  needsRefresh: boolean;
  expiresIn: number; // seconds until expiry
  claims?: any;
}

/**
 * Token management events
 */
export interface TokenManagerEvents {
  onTokenRefreshed: (token: UnifiedToken) => void;
  onTokenExpired: () => void;
  onAuthenticationFailed: (error: Error) => void;
  onProviderSwitched: (newProvider: AuthProvider) => void;
}

/**
 * Unified token manager configuration
 */
export interface TokenManagerConfig {
  // Auto-refresh settings
  autoRefresh: boolean;
  refreshThresholdMinutes: number; // Refresh when token expires within this time
  maxRetryAttempts: number;
  
  // Storage settings
  useSessionStorage: boolean;
  storagePrefix: string;
  
  // Backend settings
  backendUrl: string;
  clientId: string;
  
  // Event handlers
  events?: Partial<TokenManagerEvents>;
}

/**
 * Default configuration
 */
const DEFAULT_CONFIG: TokenManagerConfig = {
  autoRefresh: true,
  refreshThresholdMinutes: 5,
  maxRetryAttempts: 3,
  useSessionStorage: false,
  storagePrefix: 'unified_auth_',
  backendUrl: process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080',
  clientId: 'frontend-client'
};

/**
 * Unified token manager class
 * Handles token operations across Firebase and OIDC providers
 */
export class UnifiedTokenManager {
  private config: TokenManagerConfig;
  private currentToken: UnifiedToken | null = null;
  private refreshPromise: Promise<UnifiedToken | null> | null = null;
  private refreshTimer: NodeJS.Timeout | null = null;
  private firebaseAuth = getFirebaseAuth();
  private firebaseValidator = getFirebaseTokenValidator();

  constructor(config?: Partial<TokenManagerConfig>) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.initialize();
  }

  /**
   * Initialize the token manager
   */
  private initialize(): void {
    // Load existing token from storage
    this.loadTokenFromStorage();
    
    // Set up auto-refresh if enabled
    if (this.config.autoRefresh) {
      this.setupAutoRefresh();
    }
  }

  /**
   * Get current access token
   * Automatically refreshes if needed
   */
  async getCurrentToken(): Promise<string | null> {
    const token = await this.getValidToken();
    return token?.access_token || null;
  }

  /**
   * Get valid unified token (refreshes if needed)
   */
  async getValidToken(): Promise<UnifiedToken | null> {
    // Return cached token if it's still valid
    if (this.currentToken && await this.isTokenValid(this.currentToken)) {
      return this.currentToken;
    }

    // Token is invalid or missing, try to get a fresh one
    return this.refreshToken();
  }

  /**
   * Refresh the authentication token
   */
  async refreshToken(): Promise<UnifiedToken | null> {
    // Prevent concurrent refresh attempts
    if (this.refreshPromise) {
      return this.refreshPromise;
    }

    this.refreshPromise = this.performTokenRefresh();
    
    try {
      const newToken = await this.refreshPromise;
      return newToken;
    } finally {
      this.refreshPromise = null;
    }
  }

  /**
   * Perform the actual token refresh
   */
  private async performTokenRefresh(): Promise<UnifiedToken | null> {
    const provider = ProviderDetector.detectProvider();
    
    try {
      let newToken: UnifiedToken | null = null;

      switch (provider.provider) {
        case 'firebase':
          newToken = await this.refreshFirebaseToken();
          break;
          
        case 'oidc':
          newToken = await this.refreshOIDCToken();
          break;
          
        default:
          // Try both providers in priority order
          const providers = ProviderDetector.getProviderPriority();
          
          for (const providerType of providers) {
            try {
              if (providerType === 'firebase') {
                newToken = await this.refreshFirebaseToken();
              } else {
                newToken = await this.refreshOIDCToken();
              }
              
              if (newToken) {
                this.config.events?.onProviderSwitched?.(providerType);
                break;
              }
            } catch (error) {
              console.debug(`Failed to refresh with ${providerType}:`, error);
              continue;
            }
          }
      }

      if (newToken) {
        this.setCurrentToken(newToken);
        this.config.events?.onTokenRefreshed?.(newToken);
        return newToken;
      }

      // All refresh attempts failed
      this.handleAuthenticationFailure(new Error('Token refresh failed for all providers'));
      return null;

    } catch (error) {
      this.handleAuthenticationFailure(error as Error);
      return null;
    }
  }

  /**
   * Refresh Firebase token
   */
  private async refreshFirebaseToken(): Promise<UnifiedToken | null> {
    if (!this.firebaseAuth.isAuthenticated()) {
      throw new Error('Firebase user not authenticated');
    }

    // Get fresh Firebase token
    const firebaseToken = await this.firebaseAuth.getIdToken(true);
    if (!firebaseToken) {
      throw new Error('Failed to get Firebase token');
    }

    // Exchange for unified token
    const unifiedToken = await this.firebaseValidator.validateAndExchange(firebaseToken);
    return unifiedToken;
  }

  /**
   * Refresh OIDC token
   */
  private async refreshOIDCToken(): Promise<UnifiedToken | null> {
    // Try to use refresh token if available
    if (this.currentToken?.refresh_token) {
      return this.exchangeRefreshToken(this.currentToken.refresh_token);
    }

    // TODO: Implement OIDC refresh flow without refresh token
    throw new Error('OIDC token refresh not implemented');
  }

  /**
   * Exchange refresh token for new access token
   */
  private async exchangeRefreshToken(refreshToken: string): Promise<UnifiedToken | null> {
    const response = await fetch(`${this.config.backendUrl}/api/auth/token/refresh`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        refresh_token: refreshToken,
        client_id: this.config.clientId
      })
    });

    if (!response.ok) {
      throw new Error(`Token refresh failed: ${response.status}`);
    }

    const result = await response.json();
    
    if (!result.success || !result.data) {
      throw new Error(result.error?.message || 'Token refresh failed');
    }

    return result.data;
  }

  /**
   * Validate if a token is still valid
   */
  async isTokenValid(token: UnifiedToken): Promise<boolean> {
    if (!token) return false;

    try {
      const expiresAt = new Date(token.expires_at);
      const now = new Date();
      const thresholdMs = this.config.refreshThresholdMinutes * 60 * 1000;

      // Token is invalid if it expires within the threshold
      return (expiresAt.getTime() - now.getTime()) > thresholdMs;
    } catch (error) {
      console.debug('Token validation error:', error);
      return false;
    }
  }

  /**
   * Get detailed token validation result
   */
  async validateToken(token: UnifiedToken): Promise<TokenValidationResult> {
    if (!token) {
      return { isValid: false, needsRefresh: true, expiresIn: 0 };
    }

    try {
      const expiresAt = new Date(token.expires_at);
      const now = new Date();
      const expiresInMs = expiresAt.getTime() - now.getTime();
      const expiresInSeconds = Math.floor(expiresInMs / 1000);
      const thresholdMs = this.config.refreshThresholdMinutes * 60 * 1000;

      const isValid = expiresInMs > 0;
      const needsRefresh = expiresInMs <= thresholdMs;

      // Parse claims if possible
      let claims: any = undefined;
      try {
        const payload = JSON.parse(atob(token.access_token.split('.')[1]));
        claims = payload;
      } catch (e) {
        // Ignore parsing errors
      }

      return {
        isValid,
        needsRefresh,
        expiresIn: Math.max(0, expiresInSeconds),
        claims
      };
    } catch (error) {
      console.debug('Token validation error:', error);
      return { isValid: false, needsRefresh: true, expiresIn: 0 };
    }
  }

  /**
   * Clear authentication tokens
   */
  clearTokens(): void {
    this.currentToken = null;
    this.clearTokenFromStorage();
    this.clearAutoRefresh();
    
    // Clear provider-specific tokens too
    if (typeof window !== 'undefined') {
      localStorage.removeItem('firebase_token');
      localStorage.removeItem('oidc_token');
      sessionStorage.removeItem('access_token');
    }
  }

  /**
   * Set current token and persist to storage
   */
  private setCurrentToken(token: UnifiedToken): void {
    this.currentToken = token;
    this.saveTokenToStorage(token);
    this.setupAutoRefresh();
  }

  /**
   * Save token to storage
   */
  private saveTokenToStorage(token: UnifiedToken): void {
    if (typeof window === 'undefined') return;

    const storage = this.config.useSessionStorage ? sessionStorage : localStorage;
    const key = `${this.config.storagePrefix}token`;
    
    try {
      storage.setItem(key, JSON.stringify(token));
    } catch (error) {
      console.warn('Failed to save token to storage:', error);
    }
  }

  /**
   * Load token from storage
   */
  private loadTokenFromStorage(): void {
    if (typeof window === 'undefined') return;

    const storage = this.config.useSessionStorage ? sessionStorage : localStorage;
    const key = `${this.config.storagePrefix}token`;
    
    try {
      const stored = storage.getItem(key);
      if (stored) {
        const token = JSON.parse(stored) as UnifiedToken;
        // Only load if token is not expired
        if (new Date(token.expires_at) > new Date()) {
          this.currentToken = token;
        } else {
          // Remove expired token
          storage.removeItem(key);
        }
      }
    } catch (error) {
      console.debug('Failed to load token from storage:', error);
    }
  }

  /**
   * Clear token from storage
   */
  private clearTokenFromStorage(): void {
    if (typeof window === 'undefined') return;

    const storage = this.config.useSessionStorage ? sessionStorage : localStorage;
    const key = `${this.config.storagePrefix}token`;
    storage.removeItem(key);
  }

  /**
   * Set up automatic token refresh
   */
  private setupAutoRefresh(): void {
    this.clearAutoRefresh();

    if (!this.config.autoRefresh || !this.currentToken) {
      return;
    }

    try {
      const expiresAt = new Date(this.currentToken.expires_at);
      const now = new Date();
      const refreshTime = expiresAt.getTime() - this.config.refreshThresholdMinutes * 60 * 1000;
      const delay = Math.max(0, refreshTime - now.getTime());

      this.refreshTimer = setTimeout(() => {
        this.refreshToken().catch(error => {
          console.error('Auto-refresh failed:', error);
          this.config.events?.onAuthenticationFailed?.(error);
        });
      }, delay);

    } catch (error) {
      console.debug('Auto-refresh setup failed:', error);
    }
  }

  /**
   * Clear auto-refresh timer
   */
  private clearAutoRefresh(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }
  }

  /**
   * Handle authentication failure
   */
  private handleAuthenticationFailure(error: Error): void {
    console.error('Authentication failed:', error);
    this.clearTokens();
    this.config.events?.onAuthenticationFailed?.(error);
  }
}

/**
 * Singleton instance
 */
let tokenManagerInstance: UnifiedTokenManager | null = null;

/**
 * Get unified token manager instance
 */
export function getUnifiedTokenManager(config?: Partial<TokenManagerConfig>): UnifiedTokenManager {
  if (!tokenManagerInstance) {
    tokenManagerInstance = new UnifiedTokenManager(config);
  }
  return tokenManagerInstance;
}

/**
 * Initialize token manager with configuration
 */
export function initializeTokenManager(config: Partial<TokenManagerConfig>): UnifiedTokenManager {
  tokenManagerInstance = new UnifiedTokenManager(config);
  return tokenManagerInstance;
}

export default UnifiedTokenManager;