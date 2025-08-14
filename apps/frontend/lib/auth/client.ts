/**
 * Frontend OAuth 2.0 Client for EPSX Backend
 * Custom implementation replacing Auth.js for compatibility with Rust backend
 */

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export const OIDC_CONFIG = {
  client_id: process.env.OIDC_CLIENT_ID || 'epsx-frontend',
  client_secret: process.env.OIDC_CLIENT_SECRET || 'sk-frontend-2024-secure-random-secret-key-87a3f8b2c1d9e4f5',
  redirect_uri: `${process.env.NEXTAUTH_URL || 'http://localhost:3000'}/api/auth/callback/epsx-backend`,
  scope: 'openid profile email',
  response_type: 'code',
} as const;

export const OAUTH_ENDPOINTS = {
  authorization_endpoint: `${BACKEND_URL}/oauth/authorize`,
  token_endpoint: `${BACKEND_URL}/oauth/token`,
  userinfo_endpoint: `${BACKEND_URL}/oauth/userinfo`,
  jwks_endpoint: `${BACKEND_URL}/oauth/jwks`,
} as const;

// PKCE Helper Functions
function generateRandomString(length: number): string {
  const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += charset.charAt(Math.floor(Math.random() * charset.length));
  }
  return result;
}

function base64URLEncode(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let result = '';
  for (let i = 0; i < bytes.byteLength; i++) {
    result += String.fromCharCode(bytes[i]);
  }
  return btoa(result)
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

async function sha256(buffer: string): Promise<ArrayBuffer> {
  const encoder = new TextEncoder();
  const data = encoder.encode(buffer);
  return await crypto.subtle.digest('SHA-256', data);
}

export function generateCodeVerifier(): string {
  return generateRandomString(128);
}

export async function generateCodeChallenge(codeVerifier: string): Promise<string> {
  const hashed = await sha256(codeVerifier);
  return base64URLEncode(hashed);
}

export function generateState(): string {
  return generateRandomString(32);
}

// OAuth 2.0 Flow Functions
export async function getAuthorizationUrl(): Promise<{
  url: string;
  codeVerifier: string; 
  state: string;
}> {
  const codeVerifier = generateCodeVerifier();
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  const state = generateState();
  
  const params = new URLSearchParams({
    response_type: OIDC_CONFIG.response_type,
    client_id: OIDC_CONFIG.client_id,
    redirect_uri: OIDC_CONFIG.redirect_uri,
    scope: OIDC_CONFIG.scope,
    state,
    code_challenge: codeChallenge,
    code_challenge_method: 'S256',
  });
  
  return {
    url: `${OAUTH_ENDPOINTS.authorization_endpoint}?${params.toString()}`,
    codeVerifier,
    state,
  };
}

export async function exchangeCodeForTokens(
  code: string, 
  codeVerifier: string,
  state: string
): Promise<{
  accessToken: string;
  idToken: string;
  refreshToken?: string;
}> {
  console.log('🔄 Frontend: Exchanging authorization code for tokens');

  const tokenParams = new URLSearchParams({
    grant_type: 'authorization_code',
    client_id: OIDC_CONFIG.client_id,
    client_secret: OIDC_CONFIG.client_secret,
    code,
    redirect_uri: OIDC_CONFIG.redirect_uri,
    code_verifier: codeVerifier,
    state,
  });

  const response = await fetch(OAUTH_ENDPOINTS.token_endpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      'Accept': 'application/json',
    },
    body: tokenParams.toString(),
  });

  if (!response.ok) {
    const errorText = await response.text();
    console.error('❌ Frontend: Token exchange failed:', {
      status: response.status,
      statusText: response.statusText,
      error: errorText,
    });
    throw new Error(`Token exchange failed: ${response.status} ${response.statusText}`);
  }

  const tokens = await response.json();
  
  console.log('✅ Frontend: Successfully received tokens from backend');
  
  return {
    accessToken: tokens.access_token,
    idToken: tokens.id_token,
    refreshToken: tokens.refresh_token,
  };
}

export async function getUserInfo(accessToken: string): Promise<{
  sub: string;
  email: string;
  name: string;
  role: string;
  permissions: string[];
  package_tier: string;
  firebase_uid: string;
}> {
  console.log('🔄 Frontend: Fetching user information from backend');

  const response = await fetch(OAUTH_ENDPOINTS.userinfo_endpoint, {
    headers: {
      'Authorization': `Bearer ${accessToken}`,
      'Accept': 'application/json',
    },
  });

  if (!response.ok) {
    const errorText = await response.text();
    console.error('❌ Frontend: User info request failed:', {
      status: response.status,
      statusText: response.statusText,
      error: errorText,
    });
    throw new Error(`User info request failed: ${response.status} ${response.statusText}`);
  }

  const userinfo = await response.json();
  
  console.log('✅ Frontend: Successfully received user info from backend');
  
  return {
    sub: userinfo.sub,
    email: userinfo.email,
    name: userinfo.name || userinfo.email.split('@')[0],
    role: userinfo.role || 'user',
    permissions: userinfo.permissions || ['user:read'],
    package_tier: userinfo.package_tier || 'FREE',
    firebase_uid: userinfo.sub,
  };
}