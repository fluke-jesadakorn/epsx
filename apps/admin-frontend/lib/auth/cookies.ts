/**
 * Centralized JWT Cookie Management for EPSX Applications
 * Provides secure cookie handling with refresh token support
 */
import { serialize } from 'cookie';
import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';

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
  
  /**
   *
   * @param appName
   */
  constructor(appName: 'frontend' | 'admin' | 'shared') {
    this.config = this.createCookieConfig(appName);
  }

  /**
   * Create cookie configuration based on application type
   * @param appName
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
   * @param response
   * @param token
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
   * @param response
   * @param token
   */
  async setAccessToken(response: NextResponse, token: string): Promise<NextResponse> {
    return this.setAccessTokenCookie(response, token);
  }

  /**
   * Set refresh token cookie on response
   * @param response
   * @param token
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
   * @param response
   * @param accessToken
   * @param refreshToken
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
   * @param response
   */
  clearAccessTokenCookie(response: NextResponse): NextResponse {
    response.cookies.delete(this.config.accessToken.name);
    return response;
  }

  /**
   * Clear refresh token cookie
   * @param response
   */
  clearRefreshTokenCookie(response: NextResponse): NextResponse {
    response.cookies.delete(this.config.refreshToken.name);
    return response;
  }

  /**
   * Clear all authentication cookies
   * @param response
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
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Error reading access token cookie:', _error);
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
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Error reading refresh token cookie:', _error);
      return null;
    }
  }

  /**
   * Check if access token exists and is not expired
   */
  async hasValidAccessToken(): Promise<boolean> {
    const token = await this.getAccessToken();
    if (!token) {return false;}

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
    if (!token) {return 0;}

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
 * @param appName
 */
export function createCookieManager(appName: 'frontend' | 'admin' | 'shared'): JWTCookieManager {
  return new JWTCookieManager(appName);
}

/**
 * Utility function to extract JWT claims without verification
 * @param token
 */
export function extractJWTClaims(token: string): Record<string, unknown> {
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    return payload as Record<string, unknown>;
  } catch (_error) {
    throw new Error('Invalid JWT token format');
  }
}