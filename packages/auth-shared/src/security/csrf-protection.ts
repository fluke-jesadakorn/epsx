// Enhanced CSRF protection for shared cookie authentication
// Provides protection against cross-site request forgery attacks

import { getCrossAppAuthConfig } from '../config/shared-cookie';

export interface CSRFToken {
  token: string;
  timestamp: number;
  expiresAt: number;
  origin: string;
}

export interface CSRFConfig {
  tokenLength: number;
  tokenExpiry: number; // in milliseconds
  cookieName: string;
  headerName: string;
  allowedOrigins: string[];
  rotationInterval: number; // in milliseconds
}

/**
 * CSRF protection manager for cross-app authentication
 */
export class CSRFProtection {
  private config: CSRFConfig;
  private currentToken: CSRFToken | null = null;
  private rotationInterval?: NodeJS.Timeout;

  constructor(config?: Partial<CSRFConfig>) {
    const appConfig = getCrossAppAuthConfig();
    
    this.config = {
      tokenLength: 32,
      tokenExpiry: 60 * 60 * 1000, // 1 hour
      cookieName: 'epsx_csrf_token',
      headerName: 'X-CSRF-Token',
      allowedOrigins: [appConfig.frontend.url, appConfig.adminFrontend.url],
      rotationInterval: 30 * 60 * 1000, // 30 minutes
      ...config
    };

    this.startTokenRotation();
  }

  /**
   * Generate a new CSRF token
   */
  generateToken(): CSRFToken {
    const token = this.generateRandomToken(this.config.tokenLength);
    const timestamp = Date.now();
    const expiresAt = timestamp + this.config.tokenExpiry;
    const origin = window.location.origin;

    const csrfToken: CSRFToken = {
      token,
      timestamp,
      expiresAt,
      origin
    };

    this.currentToken = csrfToken;
    this.storeToken(csrfToken);
    
    return csrfToken;
  }

  /**
   * Get current CSRF token (generate if none exists)
   */
  getToken(): string {
    if (!this.currentToken || this.isTokenExpired(this.currentToken)) {
      this.generateToken();
    }
    
    return this.currentToken!.token;
  }

  /**
   * Validate CSRF token
   */
  validateToken(token: string, origin?: string): boolean {
    if (!this.currentToken) {
      return false;
    }

    // Check token match
    if (this.currentToken.token !== token) {
      return false;
    }

    // Check expiration
    if (this.isTokenExpired(this.currentToken)) {
      return false;
    }

    // Check origin if provided
    if (origin && !this.isOriginAllowed(origin)) {
      return false;
    }

    return true;
  }

  /**
   * Add CSRF token to request headers
   */
  addTokenToHeaders(headers: Record<string, string> = {}): Record<string, string> {
    return {
      ...headers,
      [this.config.headerName]: this.getToken()
    };
  }

  /**
   * Add CSRF token to form data
   */
  addTokenToFormData(formData: FormData): FormData {
    formData.append('csrf_token', this.getToken());
    return formData;
  }

  /**
   * Create CSRF-protected fetch function
   */
  createProtectedFetch() {
    return async (url: string, options: RequestInit = {}): Promise<Response> => {
      const headers = this.addTokenToHeaders(options.headers as Record<string, string>);
      
      return fetch(url, {
        ...options,
        headers
      });
    };
  }

  /**
   * Validate request origin against allowed origins
   */
  isOriginAllowed(origin: string): boolean {
    return this.config.allowedOrigins.includes(origin);
  }

  /**
   * Check if token is expired
   */
  private isTokenExpired(token: CSRFToken): boolean {
    return Date.now() > token.expiresAt;
  }

  /**
   * Generate cryptographically secure random token
   */
  private generateRandomToken(length: number): string {
    if (typeof window !== 'undefined' && window.crypto && window.crypto.getRandomValues) {
      // Browser environment
      const array = new Uint8Array(length);
      window.crypto.getRandomValues(array);
      return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
    } else {
      // Fallback for environments without crypto API
      const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
      let result = '';
      for (let i = 0; i < length; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
      }
      return result;
    }
  }

  /**
   * Store token in localStorage with expiration
   */
  private storeToken(token: CSRFToken): void {
    try {
      localStorage.setItem(`${this.config.cookieName}_data`, JSON.stringify(token));
    } catch (error) {
      console.warn('Failed to store CSRF token:', error);
    }
  }

  /**
   * Load token from localStorage
   */
  private loadToken(): CSRFToken | null {
    try {
      const stored = localStorage.getItem(`${this.config.cookieName}_data`);
      if (stored) {
        const token = JSON.parse(stored) as CSRFToken;
        if (!this.isTokenExpired(token)) {
          return token;
        } else {
          // Clean up expired token
          localStorage.removeItem(`${this.config.cookieName}_data`);
        }
      }
    } catch (error) {
      console.warn('Failed to load CSRF token:', error);
    }
    return null;
  }

  /**
   * Start automatic token rotation
   */
  private startTokenRotation(): void {
    // Load existing token
    this.currentToken = this.loadToken();

    // Set up rotation interval
    this.rotationInterval = setInterval(() => {
      if (this.currentToken) {
        this.generateToken();
        
        // Dispatch event for components to update
        window.dispatchEvent(new CustomEvent('csrf-token-rotated', {
          detail: { token: this.currentToken.token }
        }));
      }
    }, this.config.rotationInterval);
  }

  /**
   * Clean up resources
   */
  destroy(): void {
    if (this.rotationInterval) {
      clearInterval(this.rotationInterval);
    }
    
    try {
      localStorage.removeItem(`${this.config.cookieName}_data`);
    } catch (error) {
      console.warn('Failed to clean up CSRF token:', error);
    }
  }
}

/**
 * CSRF-protected request wrapper
 */
export class CSRFProtectedRequest {
  private csrfProtection: CSRFProtection;

  constructor(config?: Partial<CSRFConfig>) {
    this.csrfProtection = new CSRFProtection(config);
  }

  /**
   * Make a CSRF-protected GET request
   */
  async get(url: string, options: RequestInit = {}): Promise<Response> {
    const headers = this.csrfProtection.addTokenToHeaders(options.headers as Record<string, string>);
    
    return fetch(url, {
      ...options,
      method: 'GET',
      headers
    });
  }

  /**
   * Make a CSRF-protected POST request
   */
  async post(url: string, data?: any, options: RequestInit = {}): Promise<Response> {
    const headers = this.csrfProtection.addTokenToHeaders({
      'Content-Type': 'application/json',
      ...options.headers as Record<string, string>
    });

    return fetch(url, {
      ...options,
      method: 'POST',
      headers,
      body: data ? JSON.stringify(data) : options.body
    });
  }

  /**
   * Make a CSRF-protected PUT request
   */
  async put(url: string, data?: any, options: RequestInit = {}): Promise<Response> {
    const headers = this.csrfProtection.addTokenToHeaders({
      'Content-Type': 'application/json',
      ...options.headers as Record<string, string>
    });

    return fetch(url, {
      ...options,
      method: 'PUT',
      headers,
      body: data ? JSON.stringify(data) : options.body
    });
  }

  /**
   * Make a CSRF-protected DELETE request
   */
  async delete(url: string, options: RequestInit = {}): Promise<Response> {
    const headers = this.csrfProtection.addTokenToHeaders(options.headers as Record<string, string>);

    return fetch(url, {
      ...options,
      method: 'DELETE',
      headers
    });
  }

  /**
   * Get current CSRF token
   */
  getToken(): string {
    return this.csrfProtection.getToken();
  }

  /**
   * Validate CSRF token
   */
  validateToken(token: string, origin?: string): boolean {
    return this.csrfProtection.validateToken(token, origin);
  }

  /**
   * Clean up resources
   */
  destroy(): void {
    this.csrfProtection.destroy();
  }
}

/**
 * Global CSRF protection instance
 */
let globalCSRFProtection: CSRFProtection | null = null;

export function getCSRFProtection(): CSRFProtection {
  if (!globalCSRFProtection) {
    globalCSRFProtection = new CSRFProtection();
  }
  return globalCSRFProtection;
}

/**
 * Global CSRF-protected request instance
 */
let globalCSRFRequest: CSRFProtectedRequest | null = null;

export function getCSRFProtectedRequest(): CSRFProtectedRequest {
  if (!globalCSRFRequest) {
    globalCSRFRequest = new CSRFProtectedRequest();
  }
  return globalCSRFRequest;
}