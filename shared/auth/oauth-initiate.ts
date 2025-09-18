/**
 * Shared OAuth Initiation Utilities
 * Consolidates OAuth authorization flow initiation logic
 * Used by both frontend and admin-frontend applications
 */

import { NextResponse } from 'next/server';
import { generateCodeVerifier, generateCodeChallenge, generateState } from './pkce';
import { setPKCECookies } from './cookies';
import { getBackendUrl, URL, URLContext, OIDCEndpoint } from '../utils/url-resolver';

export interface OAuthConfig {
  clientId: string;
  scope: string;
  redirectUri: string;
  appType: 'frontend' | 'admin';
}

export interface OAuthInitiateRequest {
  redirectTo?: string;
}

export interface OAuthInitiateResponse {
  success: boolean;
  authorizationUrl?: string;
  message: string;
  error?: string;
}

export async function createOAuthInitiation(
  config: OAuthConfig,
  request: OAuthInitiateRequest
): Promise<NextResponse> {
  try {
    const { redirectTo } = request;
    
    // Generate PKCE parameters
    const codeVerifier = generateCodeVerifier();
    const codeChallenge = await generateCodeChallenge(codeVerifier);
    const state = generateState();
    
    console.log(`🔄 ${config.appType}: Generating PKCE parameters for OAuth authorization...`);
    console.log(`✅ ${config.appType}: Code verifier generated successfully`);
    console.log(`✅ ${config.appType}: Code challenge generated successfully`);
    console.log(`✅ ${config.appType}: State parameter generated successfully`);
    
    // Build authorization URL
    const authorizationEndpoint = config.appType === 'admin' 
      ? URL.oidc(OIDCEndpoint.AUTHORIZE, URLContext.SERVER)
      : `${getBackendUrl('server')}/oauth/authorize`;
    
    console.log(`🔧 ${config.appType}: OAuth configuration:`, {
      authorizationEndpoint,
      clientId: config.clientId,
      redirectUri: config.redirectUri,
      scope: config.scope
    });
    
    const params = new URLSearchParams({
      response_type: 'code',
      client_id: config.clientId,
      redirect_uri: config.redirectUri,
      scope: config.scope,
      state: state,
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
    });
    
    const authorizationUrl = `${authorizationEndpoint}?${params.toString()}`;
    
    console.log(`✅ ${config.appType}: Authorization URL generated successfully:`, authorizationUrl);
    
    // Create response with authorization URL
    const response = NextResponse.json({
      success: true,
      authorizationUrl,
      message: 'PKCE parameters generated successfully'
    } as OAuthInitiateResponse);
    
    // Set PKCE cookies for callback verification
    setPKCECookies(response, {
      codeVerifier,
      state,
      redirectTo
    });
    
    return response;
    
  } catch (error) {
    console.error(`❌ ${config.appType}: OAuth initiation failed:`, error);
    return NextResponse.json({
      success: false,
      message: 'OAuth initiation failed',
      error: error instanceof Error ? error.message : 'Unknown error'
    } as OAuthInitiateResponse, { status: 500 });
  }
}

export function createFrontendOAuthConfig(redirectUri: string): OAuthConfig {
  return {
    clientId: 'epsx-frontend',
    scope: 'openid profile email',
    redirectUri,
    appType: 'frontend'
  };
}

export function createAdminOAuthConfig(redirectUri: string): OAuthConfig {
  return {
    clientId: 'epsx-admin',
    scope: 'openid profile email permissions',
    redirectUri,
    appType: 'admin'
  };
}