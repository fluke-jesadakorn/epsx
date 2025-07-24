import { cookies } from 'next/headers';
import { NextRequest, NextResponse } from 'next/server';

// Cookie configuration constants
export const COOKIE_NAMES = {
  SESSION: '__session',
  CSRF: '__csrf_token',
  REFRESH: '__refresh_token',
  THEME: '__theme',
  THEME_MODE: '__theme_mode',
} as const;

export const COOKIE_CONFIG = {
  SESSION: {
    maxAge: 60 * 60 * 24 * 5, // 5 days
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
    path: '/',
  },
  CSRF: {
    maxAge: 60 * 60 * 24, // 1 day
    httpOnly: false, // CSRF token needs to be accessible to client
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict' as const,
    path: '/',
  },
  REFRESH: {
    maxAge: 60 * 60 * 24 * 30, // 30 days
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict' as const,
    path: '/api/v1/authentication',
  },
  THEME: {
    maxAge: 60 * 60 * 24 * 365, // 1 year
    httpOnly: false, // Client needs access for theme switching
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
    path: '/',
  },
  THEME_MODE: {
    maxAge: 60 * 60 * 24 * 365, // 1 year
    httpOnly: false, // Client needs access for theme switching
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
    path: '/',
  },
} as const;

// Server-side cookie utilities (for App Router server components and API routes)
export class ServerCookies {
  /**
   * Set a secure cookie on the server side
   */
  static async set(
    name: keyof typeof COOKIE_NAMES,
    value: string,
    options?: Partial<typeof COOKIE_CONFIG.SESSION>
  ): Promise<void> {
    const cookieStore = await cookies();
    const cookieName = COOKIE_NAMES[name];
    const config = { ...COOKIE_CONFIG[name], ...options };
    
    cookieStore.set(cookieName, value, config);
  }

  /**
   * Get a cookie value on the server side
   */
  static async get(name: keyof typeof COOKIE_NAMES): Promise<string | undefined> {
    try {
      const cookieStore = await cookies();
      const cookieName = COOKIE_NAMES[name];
      return cookieStore.get(cookieName)?.value;
    } catch (error) {
      console.error(`Error getting cookie ${name}:`, error);
      return undefined;
    }
  }

  /**
   * Delete a cookie on the server side
   */
  static async delete(name: keyof typeof COOKIE_NAMES): Promise<void> {
    try {
      const cookieStore = await cookies();
      const cookieName = COOKIE_NAMES[name];
      cookieStore.delete(cookieName);
    } catch (error) {
      console.error(`Error deleting cookie ${name}:`, error);
    }
  }

  /**
   * Check if a cookie exists on the server side
   */
  static async has(name: keyof typeof COOKIE_NAMES): Promise<boolean> {
    try {
      const cookieStore = await cookies();
      const cookieName = COOKIE_NAMES[name];
      return cookieStore.has(cookieName);
    } catch (error) {
      console.error(`Error checking cookie ${name}:`, error);
      return false;
    }
  }

  /**
   * Get all authentication-related cookies
   */
  static async getAuthCookies(): Promise<{
    session?: string;
    csrf?: string;
    refresh?: string;
  }> {
    const [session, csrf, refresh] = await Promise.all([
      this.get('SESSION'),
      this.get('CSRF'),
      this.get('REFRESH'),
    ]);

    return { session, csrf, refresh };
  }

  /**
   * Clear all authentication cookies
   */
  static async clearAuthCookies(): Promise<void> {
    await Promise.all([
      this.delete('SESSION'),
      this.delete('CSRF'),
      this.delete('REFRESH'),
    ]);
  }
}

// API Route cookie utilities (for NextResponse manipulation)
export class ApiCookies {
  /**
   * Set a cookie in an API route response
   */
  static set(
    response: NextResponse,
    name: keyof typeof COOKIE_NAMES,
    value: string,
    options?: Partial<typeof COOKIE_CONFIG.SESSION>
  ): NextResponse {
    const cookieName = COOKIE_NAMES[name];
    const config = { ...COOKIE_CONFIG[name], ...options };
    
    // Build cookie string manually for better control
    const cookieString = this.buildCookieString(cookieName, value, config);
    
    // Handle multiple cookies by appending to existing Set-Cookie headers
    const existingCookies = response.headers.get('Set-Cookie') || '';
    const newCookies = existingCookies 
      ? `${existingCookies}, ${cookieString}`
      : cookieString;
    
    response.headers.set('Set-Cookie', newCookies);
    return response;
  }

  /**
   * Get a cookie from API route request
   */
  static get(request: NextRequest, name: keyof typeof COOKIE_NAMES): string | undefined {
    const cookieName = COOKIE_NAMES[name];
    return request.cookies.get(cookieName)?.value;
  }

  /**
   * Delete a cookie in an API route response
   */
  static delete(
    response: NextResponse,
    name: keyof typeof COOKIE_NAMES
  ): NextResponse {
    const cookieName = COOKIE_NAMES[name];
    const config = { ...COOKIE_CONFIG[name], maxAge: 0, expires: new Date(0) };
    
    const cookieString = this.buildCookieString(cookieName, '', config);
    
    const existingCookies = response.headers.get('Set-Cookie') || '';
    const newCookies = existingCookies 
      ? `${existingCookies}, ${cookieString}`
      : cookieString;
    
    response.headers.set('Set-Cookie', newCookies);
    return response;
  }

  /**
   * Set multiple authentication cookies at once
   */
  static setAuthCookies(
    response: NextResponse,
    cookies: {
      session?: string;
      csrf?: string;
      refresh?: string;
    }
  ): NextResponse {
    let updatedResponse = response;
    
    if (cookies.session) {
      updatedResponse = this.set(updatedResponse, 'SESSION', cookies.session);
    }
    if (cookies.csrf) {
      updatedResponse = this.set(updatedResponse, 'CSRF', cookies.csrf);
    }
    if (cookies.refresh) {
      updatedResponse = this.set(updatedResponse, 'REFRESH', cookies.refresh);
    }
    
    return updatedResponse;
  }

  /**
   * Clear all authentication cookies in response
   */
  static clearAuthCookies(response: NextResponse): NextResponse {
    let updatedResponse = response;
    updatedResponse = this.delete(updatedResponse, 'SESSION');
    updatedResponse = this.delete(updatedResponse, 'CSRF');
    updatedResponse = this.delete(updatedResponse, 'REFRESH');
    return updatedResponse;
  }

  /**
   * Build a proper cookie string
   */
  private static buildCookieString(
    name: string,
    value: string,
    options: any
  ): string {
    let cookieString = `${name}=${encodeURIComponent(value)}`;
    
    if (options.maxAge) {
      cookieString += `; Max-Age=${options.maxAge}`;
    }
    if (options.expires) {
      cookieString += `; Expires=${options.expires.toUTCString()}`;
    }
    if (options.path) {
      cookieString += `; Path=${options.path}`;
    }
    if (options.domain) {
      cookieString += `; Domain=${options.domain}`;
    }
    if (options.secure) {
      cookieString += '; Secure';
    }
    if (options.httpOnly) {
      cookieString += '; HttpOnly';
    }
    if (options.sameSite) {
      cookieString += `; SameSite=${options.sameSite}`;
    }
    
    return cookieString;
  }
}

// Client-side cookie utilities (for browser/client components)
export class ClientCookies {
  /**
   * Get a cookie value on the client side
   */
  static get(name: keyof typeof COOKIE_NAMES): string | undefined {
    if (typeof document === 'undefined') return undefined;
    
    const cookieName = COOKIE_NAMES[name];
    const cookies = document.cookie.split(';');
    
    for (const cookie of cookies) {
      const [key, value] = cookie.trim().split('=');
      if (key === cookieName) {
        return decodeURIComponent(value);
      }
    }
    
    return undefined;
  }

  /**
   * Set a cookie on the client side (limited use - prefer server-side for security)
   */
  static set(
    name: keyof typeof COOKIE_NAMES,
    value: string,
    options?: {
      maxAge?: number;
      path?: string;
      secure?: boolean;
      sameSite?: 'strict' | 'lax' | 'none';
    }
  ): void {
    if (typeof document === 'undefined') return;
    
    const cookieName = COOKIE_NAMES[name];
    let cookieString = `${cookieName}=${encodeURIComponent(value)}`;
    
    if (options?.maxAge) {
      cookieString += `; Max-Age=${options.maxAge}`;
    }
    if (options?.path) {
      cookieString += `; Path=${options.path}`;
    }
    if (options?.secure) {
      cookieString += '; Secure';
    }
    if (options?.sameSite) {
      cookieString += `; SameSite=${options.sameSite}`;
    }
    
    document.cookie = cookieString;
  }

  /**
   * Delete a cookie on the client side
   */
  static delete(name: keyof typeof COOKIE_NAMES): void {
    if (typeof document === 'undefined') return;
    
    const cookieName = COOKIE_NAMES[name];
    document.cookie = `${cookieName}=; Max-Age=0; Path=/`;
  }

  /**
   * Check if a cookie exists on the client side
   */
  static has(name: keyof typeof COOKIE_NAMES): boolean {
    return this.get(name) !== undefined;
  }
}

// Utility functions for cookie validation and security
export class CookieValidator {
  /**
   * Validate cookie value format and security
   */
  static isValidCookieValue(value: string): boolean {
    // Check for null bytes, control characters, and other security issues
    return !/[\x00-\x08\x0A-\x1F\x7F]/.test(value);
  }

  /**
   * Sanitize cookie value
   */
  static sanitizeCookieValue(value: string): string {
    return value.replace(/[^\w\-\.=]/g, '');
  }

  /**
   * Generate a secure CSRF token
   */
  static generateCSRFToken(): string {
    const array = new Uint8Array(32);
    if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
      crypto.getRandomValues(array);
    } else {
      // Fallback for Node.js
      const crypto = require('crypto');
      return crypto.randomBytes(32).toString('hex');
    }
    return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
  }

  /**
   * Validate CSRF token
   */
  static validateCSRFToken(token: string, expectedToken: string): boolean {
    if (!token || !expectedToken) return false;
    return token === expectedToken;
  }
}