/**
 * Shared OIDC Cookie Management Utilities
 * Standardizes cookie handling across frontend and admin-frontend applications
 */

import { NextResponse } from 'next/server';

export interface OIDCTokens {
  accessToken: string;
  idToken: string;
  refreshToken: string;
  expiresIn?: number;
}

export interface PKCECookies {
  codeVerifier: string;
  state: string;
  redirectTo?: string;
}

const getCookieOptions = (maxAge?: number) => ({
  httpOnly: true,
  secure: process.env.NODE_ENV === 'production',
  sameSite: 'lax' as const,
  path: '/',
  ...(maxAge && { maxAge }),
});

// OIDC Token Cookie Names
export const OIDC_COOKIES = {
  ACCESS_TOKEN: 'access_token',
  ID_TOKEN: 'id_token',
  REFRESH_TOKEN: 'refresh_token',
} as const;

// PKCE Cookie Names  
export const PKCE_COOKIES = {
  CODE_VERIFIER: 'oauth_code_verifier',
  STATE: 'oauth_state',
  REDIRECT_TO: 'oauth_redirect_to',
  CALLBACK_URL: 'oauth_callback_url',
} as const;

// Legacy Cookie Names (for cleanup)
export const LEGACY_COOKIES = {
  FRONTEND_JWT: 'epsx_frontend_jwt',
  ADMIN_JWT: 'epsx_admin_jwt',
} as const;

export function setOIDCTokens(response: NextResponse, tokens: OIDCTokens): void {
  const accessTokenAge = tokens.expiresIn || 3600; // 1 hour default
  const refreshTokenAge = 30 * 24 * 60 * 60; // 30 days

  response.cookies.set(OIDC_COOKIES.ACCESS_TOKEN, tokens.accessToken, 
    getCookieOptions(accessTokenAge));
    
  response.cookies.set(OIDC_COOKIES.ID_TOKEN, tokens.idToken,
    getCookieOptions(accessTokenAge));
    
  response.cookies.set(OIDC_COOKIES.REFRESH_TOKEN, tokens.refreshToken,
    getCookieOptions(refreshTokenAge));
}

export function setPKCECookies(response: NextResponse, pkce: PKCECookies): void {
  const pkceAge = 900; // 15 minutes

  response.cookies.set(PKCE_COOKIES.CODE_VERIFIER, pkce.codeVerifier,
    getCookieOptions(pkceAge));
    
  response.cookies.set(PKCE_COOKIES.STATE, pkce.state,
    getCookieOptions(pkceAge));

  if (pkce.redirectTo) {
    response.cookies.set(PKCE_COOKIES.REDIRECT_TO, pkce.redirectTo,
      getCookieOptions(pkceAge));
  }
}

export function clearOIDCTokens(response: NextResponse): void {
  response.cookies.delete(OIDC_COOKIES.ACCESS_TOKEN);
  response.cookies.delete(OIDC_COOKIES.ID_TOKEN);
  response.cookies.delete(OIDC_COOKIES.REFRESH_TOKEN);
}

export function clearPKCECookies(response: NextResponse): void {
  response.cookies.delete(PKCE_COOKIES.CODE_VERIFIER);
  response.cookies.delete(PKCE_COOKIES.STATE);
  response.cookies.delete(PKCE_COOKIES.REDIRECT_TO);
  response.cookies.delete(PKCE_COOKIES.CALLBACK_URL);
}

export function clearLegacyCookies(response: NextResponse): void {
  response.cookies.delete(LEGACY_COOKIES.FRONTEND_JWT);
  response.cookies.delete(LEGACY_COOKIES.ADMIN_JWT);
}

export function clearAllAuthCookies(response: NextResponse): void {
  clearOIDCTokens(response);
  clearPKCECookies(response);
  clearLegacyCookies(response);
}