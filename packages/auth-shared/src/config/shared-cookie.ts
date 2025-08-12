// Shared cookie configuration for cross-app authentication
// Supports both localhost (different ports) and production (subdomains)

export interface SharedCookieConfig {
  domain?: string;
  secure: boolean;
  sameSite: 'strict' | 'lax' | 'none';
  httpOnly: boolean;
  maxAge: number;
  path: string;
}

export interface CrossAppAuthConfig {
  frontend: {
    url: string;
    port: number;
  };
  adminFrontend: {
    url: string;
    port: number;
  };
  backend: {
    url: string;
    port: number;
  };
  cookieName: string;
  sharedDomain?: string;
  useLocalStorageSync: boolean;
}

/**
 * Get shared cookie configuration based on environment
 */
export function getSharedCookieConfig(isDevelopment = false): SharedCookieConfig {
  if (isDevelopment) {
    // Development: Use localhost with relaxed settings
    return {
      domain: 'localhost',
      secure: false,
      sameSite: 'lax',
      httpOnly: false, // Allow JS access for cross-port sync
      maxAge: 60 * 60 * 24, // 24 hours
      path: '/',
    };
  }

  // Production: Use secure settings with shared domain
  return {
    domain: process.env.NEXT_PUBLIC_COOKIE_DOMAIN || '.epsx.com',
    secure: true,
    sameSite: 'strict',
    httpOnly: true,
    maxAge: 60 * 60 * 8, // 8 hours in production
    path: '/',
  };
}

/**
 * Cross-app authentication configuration
 */
export function getCrossAppAuthConfig(): CrossAppAuthConfig {
  const isDevelopment = process.env.NODE_ENV === 'development';
  
  if (isDevelopment) {
    return {
      frontend: {
        url: process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000',
        port: 3000,
      },
      adminFrontend: {
        url: process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001',
        port: 3001,
      },
      backend: {
        url: process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080',
        port: 8080,
      },
      cookieName: 'epsx_auth_token',
      useLocalStorageSync: true, // Use localStorage sync for localhost
    };
  }

  // Production configuration
  return {
    frontend: {
      url: process.env.NEXT_PUBLIC_FRONTEND_URL || 'https://app.epsx.com',
      port: 443,
    },
    adminFrontend: {
      url: process.env.NEXT_PUBLIC_ADMIN_URL || 'https://admin.epsx.com',
      port: 443,
    },
    backend: {
      url: process.env.NEXT_PUBLIC_BACKEND_URL || 'https://api.epsx.com',
      port: 443,
    },
    cookieName: 'epsx_auth_token',
    sharedDomain: '.epsx.com',
    useLocalStorageSync: false, // Use proper cookies in production
  };
}

/**
 * Cookie utility functions
 */
export class SharedCookieManager {
  private config: CrossAppAuthConfig;
  private cookieConfig: SharedCookieConfig;

  constructor() {
    this.config = getCrossAppAuthConfig();
    this.cookieConfig = getSharedCookieConfig(process.env.NODE_ENV === 'development');
  }

  /**
   * Set authentication token across both apps
   */
  async setAuthToken(token: string, userId: string): Promise<void> {
    if (this.config.useLocalStorageSync) {
      // Development: Use localStorage + message passing
      await this.setTokenWithLocalStorage(token, userId);
    } else {
      // Production: Use shared domain cookies
      await this.setTokenWithSharedCookie(token, userId);
    }
  }

  /**
   * Get authentication token
   */
  async getAuthToken(): Promise<string | null> {
    if (this.config.useLocalStorageSync) {
      return this.getTokenFromLocalStorage();
    } else {
      return this.getTokenFromSharedCookie();
    }
  }

  /**
   * Clear authentication token from both apps
   */
  async clearAuthToken(): Promise<void> {
    if (this.config.useLocalStorageSync) {
      await this.clearTokenFromLocalStorage();
    } else {
      await this.clearTokenFromSharedCookie();
    }
  }

  /**
   * Development: Use localStorage with cross-origin message passing
   */
  private async setTokenWithLocalStorage(token: string, userId: string): Promise<void> {
    const authData = {
      token,
      userId,
      timestamp: Date.now(),
      expiresAt: Date.now() + (this.cookieConfig.maxAge * 1000),
    };

    // Store in localStorage
    localStorage.setItem(this.config.cookieName, JSON.stringify(authData));
    localStorage.setItem(`${this.config.cookieName}_sync`, Date.now().toString());

    // Notify other tabs/windows
    window.dispatchEvent(new CustomEvent('auth-sync', {
      detail: { type: 'login', data: authData }
    }));

    // Also try to sync with other app via postMessage (if available)
    try {
      // This requires special handling - we'd need iframe or window.open communication
      // For now, rely on localStorage events and cross-app sync
    } catch (error) {
      console.warn('Cross-app sync failed:', error);
    }
  }

  /**
   * Production: Use shared domain cookies
   */
  private async setTokenWithSharedCookie(token: string, userId: string): Promise<void> {
    const authData = {
      token,
      userId,
      timestamp: Date.now(),
      expiresAt: Date.now() + (this.cookieConfig.maxAge * 1000),
    };

    // Set cookie for shared domain
    document.cookie = `${this.config.cookieName}=${encodeURIComponent(JSON.stringify(authData))}; ` +
      `domain=${this.cookieConfig.domain}; ` +
      `path=${this.cookieConfig.path}; ` +
      `max-age=${this.cookieConfig.maxAge}; ` +
      `${this.cookieConfig.secure ? 'secure; ' : ''}` +
      `samesite=${this.cookieConfig.sameSite}`;
  }

  private getTokenFromLocalStorage(): string | null {
    try {
      const stored = localStorage.getItem(this.config.cookieName);
      if (!stored) return null;

      const authData = JSON.parse(stored);
      
      // Check expiration
      if (authData.expiresAt && Date.now() > authData.expiresAt) {
        localStorage.removeItem(this.config.cookieName);
        return null;
      }

      return authData.token;
    } catch {
      return null;
    }
  }

  private getTokenFromSharedCookie(): string | null {
    try {
      const cookies = document.cookie.split(';');
      const authCookie = cookies.find(cookie => 
        cookie.trim().startsWith(`${this.config.cookieName}=`)
      );

      if (!authCookie) return null;

      const value = authCookie.split('=')[1];
      const authData = JSON.parse(decodeURIComponent(value));
      
      // Check expiration
      if (authData.expiresAt && Date.now() > authData.expiresAt) {
        this.clearTokenFromSharedCookie();
        return null;
      }

      return authData.token;
    } catch {
      return null;
    }
  }

  private async clearTokenFromLocalStorage(): Promise<void> {
    localStorage.removeItem(this.config.cookieName);
    localStorage.removeItem(`${this.config.cookieName}_sync`);

    // Notify other tabs/windows
    window.dispatchEvent(new CustomEvent('auth-sync', {
      detail: { type: 'logout' }
    }));
  }

  private async clearTokenFromSharedCookie(): Promise<void> {
    document.cookie = `${this.config.cookieName}=; ` +
      `domain=${this.cookieConfig.domain}; ` +
      `path=${this.cookieConfig.path}; ` +
      `expires=Thu, 01 Jan 1970 00:00:00 GMT`;
  }

  /**
   * Setup cross-app sync listeners
   */
  setupSyncListeners(onAuthChange: (isLoggedIn: boolean) => void): void {
    if (this.config.useLocalStorageSync) {
      // Listen for localStorage changes from other tabs
      window.addEventListener('storage', (e) => {
        if (e.key === `${this.config.cookieName}_sync`) {
          const token = this.getTokenFromLocalStorage();
          onAuthChange(!!token);
        }
      });

      // Listen for custom auth sync events
      window.addEventListener('auth-sync', (e: any) => {
        const { type } = e.detail;
        onAuthChange(type === 'login');
      });
    }

    // Also listen for visibility changes to sync state
    document.addEventListener('visibilitychange', () => {
      if (!document.hidden) {
        const token = this.getTokenFromLocalStorage();
        onAuthChange(!!token);
      }
    });
  }
}

/**
 * Singleton instance
 */
export const sharedCookieManager = new SharedCookieManager();