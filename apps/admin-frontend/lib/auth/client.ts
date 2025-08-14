/**
 * OAuth 2.0 Client Configuration
 * Simple OAuth implementation using fetch instead of openid-client
 */
import { randomBytes, createHash } from 'crypto';

// OIDC Configuration for our backend
const OIDC_CONFIG = {
  issuer: process.env.BACKEND_URL || 'http://localhost:8080',
  client_id: process.env.OIDC_CLIENT_ID || 'epsx-admin',
  client_secret: process.env.OIDC_CLIENT_SECRET || 'sk-admin-2024-secure-random-secret-key-9b5e8f2a7c4d1f6e',
  redirect_uri: `${process.env.NEXTAUTH_URL || 'http://localhost:3001'}/api/auth/callback/epsx-backend`,
  scope: 'openid profile email admin',
};

// Backend OAuth endpoints
const OAUTH_ENDPOINTS = {
  authorization_endpoint: `${OIDC_CONFIG.issuer}/oauth/authorize`,
  token_endpoint: `${OIDC_CONFIG.issuer}/oauth/token`,
  userinfo_endpoint: `${OIDC_CONFIG.issuer}/oauth/userinfo`,
};

/**
 * Generate OAuth authorization URL with PKCE
 */
export async function getAuthorizationUrl(): Promise<{
  url: string;
  codeVerifier: string;
  state: string;
}> {
  // Generate PKCE and state for security
  const codeVerifier = generateCodeVerifier();
  const codeChallenge = generateCodeChallenge(codeVerifier);
  const state = generateState();

  // Build authorization URL manually
  const params = new URLSearchParams({
    response_type: 'code',
    client_id: OIDC_CONFIG.client_id,
    redirect_uri: OIDC_CONFIG.redirect_uri,
    scope: OIDC_CONFIG.scope,
    state,
    code_challenge: codeChallenge,
    code_challenge_method: 'S256',
  });

  const authUrl = `${OAUTH_ENDPOINTS.authorization_endpoint}?${params.toString()}`;

  console.log('🔧 Generated authorization URL:', {
    url: authUrl,
    state,
    codeChallenge,
  });

  return {
    url: authUrl,
    codeVerifier,
    state,
  };
}

/**
 * Exchange authorization code for tokens
 */
export async function exchangeCodeForTokens(
  code: string,
  codeVerifier: string,
  state: string
): Promise<{
  accessToken: string;
  idToken?: string;
  refreshToken?: string;
}> {
  console.log('🔧 Exchanging code for tokens:', {
    code: code.substring(0, 10) + '...',
    codeVerifier: codeVerifier.substring(0, 10) + '...',
    state,
  });

  // Prepare token request
  const tokenRequest = new URLSearchParams({
    grant_type: 'authorization_code',
    code,
    redirect_uri: OIDC_CONFIG.redirect_uri,
    client_id: OIDC_CONFIG.client_id,
    client_secret: OIDC_CONFIG.client_secret,
    code_verifier: codeVerifier,
  });

  // Exchange authorization code for tokens via fetch
  const response = await fetch(OAUTH_ENDPOINTS.token_endpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body: tokenRequest,
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Token exchange failed: ${response.status} ${errorText}`);
  }

  const tokenSet = await response.json();

  console.log('🔧 Received tokens:', {
    access_token: tokenSet.access_token ? 'present' : 'missing',
    id_token: tokenSet.id_token ? 'present' : 'missing',
    refresh_token: tokenSet.refresh_token ? 'present' : 'missing',
  });

  return {
    accessToken: tokenSet.access_token,
    idToken: tokenSet.id_token,
    refreshToken: tokenSet.refresh_token,
  };
}

/**
 * Get user info using access token
 */
export async function getUserInfo(accessToken: string): Promise<any> {
  console.log('🔧 Fetching user info with access token:', accessToken.substring(0, 10) + '...');

  const response = await fetch(OAUTH_ENDPOINTS.userinfo_endpoint, {
    headers: {
      'Authorization': `Bearer ${accessToken}`,
    },
  });

  console.log('🔧 UserInfo response status:', response.status);

  if (!response.ok) {
    const errorText = await response.text();
    console.error('🔧 UserInfo error response:', errorText);
    throw new Error(`UserInfo request failed: ${response.status} ${errorText}`);
  }

  const userinfo = await response.json();
  
  console.log('🔧 Received user info:', userinfo);

  return userinfo;
}

// Helper functions to replace openid-client generators
function generateCodeVerifier(): string {
  return randomBytes(32).toString('base64url');
}

function generateCodeChallenge(verifier: string): string {
  return createHash('sha256').update(verifier).digest('base64url');
}

function generateState(): string {
  return randomBytes(16).toString('base64url');
}

export { OIDC_CONFIG };