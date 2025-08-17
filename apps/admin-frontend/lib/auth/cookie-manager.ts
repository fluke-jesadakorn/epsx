/**
 * Centralized JWT Cookie Management for EPSX Applications
 * Provides secure cookie handling with refresh token support
 */
import { NextResponse } from 'next/server';
import { serialize } from 'cookie';
import { cookies } from 'next/headers';

export interface CookieConfig {
  name: string;
  domain?: string;
  maxAge: number;
  secure?: boolean;
  httpOnly?: boolean;
  sameSite?: 'strict' | 'lax' | 'none';
  path?: string;
}

export interface TokenCookies {
  accessToken: CookieConfig;
  refreshToken: CookieConfig;
}

/**
 * JWT Cookie Manager with production-ready security features
 */
export class JWTCookieManager {
  private config: TokenCookies;
  
  constructor(appName: 'frontend' | 'admin' | 'shared') {
    this.config = this.createCookieConfig(appName);
  }

  /**
   * Create cookie configuration based on application type
   */
  private createCookieConfig(appName: 'frontend' | 'admin' | 'shared'): TokenCookies {
    const isProduction = process.env.NODE_ENV === 'production';
    const domain = process.env.COOKIE_DOMAIN;
    
    return {
      accessToken: {
        name: `epsx_${appName}_jwt`,
        maxAge: 15 * 60, // 15 minutes for better security
        httpOnly: true,
        secure: isProduction,
        sameSite: 'lax',
        path: '/',
        domain: domain,
      },
      refreshToken: {
        name: `epsx_${appName}_refresh`,
        maxAge: 7 * 24 * 60 * 60, // 7 days
        httpOnly: true,
        secure: isProduction,
        sameSite: 'strict', // More restrictive for refresh tokens
        path: '/',
        domain: domain,
      },
    };
  }

  /**
   * Set JWT access token cookie on response
   */
  setAccessTokenCookie(response: NextResponse, token: string): NextResponse {
    const config = this.config.accessToken;
    
    response.headers.append('Set-Cookie', serialize(config.name, token, {
      httpOnly: config.httpOnly,
      secure: config.secure,
      sameSite: config.sameSite,
      maxAge: config.maxAge,
      path: config.path,
      ...(config.domain && { domain: config.domain }),
    }));
    
    return response;
  }

  /**
   * Alias for setAccessTokenCookie for compatibility
   */
  async setAccessToken(response: NextResponse, token: string): Promise<NextResponse> {
    return this.setAccessTokenCookie(response, token);
  }

  /**
   * Set refresh token cookie on response
   */
  setRefreshTokenCookie(response: NextResponse, token: string): NextResponse {
    const config = this.config.refreshToken;
    
    response.headers.append('Set-Cookie', serialize(config.name, token, {
      httpOnly: config.httpOnly,
      secure: config.secure,
      sameSite: config.sameSite,
      maxAge: config.maxAge,
      path: config.path,
      ...(config.domain && { domain: config.domain }),
    }));
    
    return response;
  }

  /**
   * Set both access and refresh token cookies
   */
  setTokenCookies(
    response: NextResponse, 
    accessToken: string, 
    refreshToken: string
  ): NextResponse {
    this.setAccessTokenCookie(response, accessToken);
    this.setRefreshTokenCookie(response, refreshToken);
    return response;
  }

  /**
   * Clear access token cookie
   */
  clearAccessTokenCookie(response: NextResponse): NextResponse {
    response.cookies.delete(this.config.accessToken.name);
    return response;
  }

  /**
   * Clear refresh token cookie
   */
  clearRefreshTokenCookie(response: NextResponse): NextResponse {
    response.cookies.delete(this.config.refreshToken.name);
    return response;
  }

  /**
   * Clear all authentication cookies
   */
  clearAllCookies(response: NextResponse): NextResponse {
    this.clearAccessTokenCookie(response);
    this.clearRefreshTokenCookie(response);
    return response;
  }

  /**
   * Get access token from request cookies
   */
  async getAccessToken(): Promise<string | null> {
    try {
      const cookieStore = await cookies();
      const tokenCookie = cookieStore.get(this.config.accessToken.name);
      return tokenCookie?.value || null;
    } catch (error) {
      console.error('Error reading access token cookie:', error);
      return null;
    }
  }

  /**
   * Get refresh token from request cookies
   */
  async getRefreshToken(): Promise<string | null> {
    try {
      const cookieStore = await cookies();
      const tokenCookie = cookieStore.get(this.config.refreshToken.name);
      return tokenCookie?.value || null;
    } catch (error) {
      console.error('Error reading refresh token cookie:', error);
      return null;
    }
  }

  /**
   * Check if access token exists and is not expired
   */
  async hasValidAccessToken(): Promise<boolean> {
    const token = await this.getAccessToken();
    if (!token) return false;

    try {
      // Basic JWT expiration check (without full verification)
      const payload = JSON.parse(atob(token.split('.')[1]));
      const now = Math.floor(Date.now() / 1000);
      return payload.exp > now;
    } catch {
      return false;
    }
  }

  /**
   * Get time until token expiration in seconds
   */
  async getTokenTimeToExpiry(): Promise<number> {
    const token = await this.getAccessToken();
    if (!token) return 0;

    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      const now = Math.floor(Date.now() / 1000);
      return Math.max(0, payload.exp - now);
    } catch {
      return 0;
    }
  }

  /**
   * Check if token needs refresh (expires within 5 minutes)
   */
  async needsRefresh(): Promise<boolean> {
    const timeToExpiry = await this.getTokenTimeToExpiry();
    return timeToExpiry > 0 && timeToExpiry < 300; // 5 minutes
  }

  /**
   * Get cookie configuration
   */
  getConfig(): TokenCookies {
    return this.config;
  }
}

/**
 * Factory function to create cookie manager for specific app
 */
export function createCookieManager(appName: 'frontend' | 'admin' | 'shared'): JWTCookieManager {
  return new JWTCookieManager(appName);
}

/**
 * Utility function to extract JWT claims without verification
 */
export function extractJWTClaims(token: string): any {
  try {
    return JSON.parse(atob(token.split('.')[1]));
  } catch (error) {
    throw new Error('Invalid JWT token format');
  }
}